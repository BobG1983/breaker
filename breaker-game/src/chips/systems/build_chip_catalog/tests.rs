use bevy::prelude::*;
use rantzsoft_defaults::prelude::{RegistryHandles, SeedableRegistry};

use super::system::build_chip_catalog;
use crate::{
    chips::{
        definition::{ChipTemplate, EvolutionIngredient, EvolutionTemplate, RaritySlot},
        resources::{ChipCatalog, ChipTemplateRegistry, EvolutionTemplateRegistry},
    },
    effect_v3::{
        effects::PiercingConfig,
        types::{EffectType, RootNode, StampTarget, Tree},
    },
    prelude::*,
};

// ── Test helpers ────────────────────────────────────────────────────

fn test_app() -> App {
    TestAppBuilder::new()
        .with_resource::<ChipTemplateRegistry>()
        .with_resource::<EvolutionTemplateRegistry>()
        // RegistryHandles are inserted manually per test
        .with_system(Update, build_chip_catalog.map(drop))
        .build()
}

fn insert_loaded_handles(app: &mut App) {
    let mut template_handles = RegistryHandles::<ChipTemplate>::new(Handle::default());
    template_handles.loaded = true;
    app.insert_resource(template_handles);

    let mut evolution_handles = RegistryHandles::<EvolutionTemplate>::new(Handle::default());
    evolution_handles.loaded = true;
    app.insert_resource(evolution_handles);
}

fn make_template(
    name: &str,
    max_taken: u32,
    common: Option<(&str, Vec<RootNode>)>,
    uncommon: Option<(&str, Vec<RootNode>)>,
) -> ChipTemplate {
    ChipTemplate {
        name: name.to_owned(),
        max_taken,
        common: common.map(|(prefix, effects)| RaritySlot {
            prefix: prefix.to_owned(),
            effects,
        }),
        uncommon: uncommon.map(|(prefix, effects)| RaritySlot {
            prefix: prefix.to_owned(),
            effects,
        }),
        rare: None,
    }
}

fn piercing_effects(count: u32) -> Vec<RootNode> {
    vec![RootNode::Stamp(
        StampTarget::Bolt,
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: count })),
    )]
}

fn make_evolution(name: &str, ingredients: Vec<EvolutionIngredient>) -> EvolutionTemplate {
    EvolutionTemplate {
        name: name.to_owned(),
        description: String::new(),
        max_stacks: 1,
        effects: vec![RootNode::Stamp(
            StampTarget::Bolt,
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 5 })),
        )],
        ingredients,
    }
}

/// Creates `AssetId` values by adding assets to an `Assets<ChipTemplate>` store.
fn template_asset_pairs(
    templates: Vec<ChipTemplate>,
) -> Vec<(AssetId<ChipTemplate>, ChipTemplate)> {
    let mut assets = Assets::<ChipTemplate>::default();
    templates
        .into_iter()
        .map(|t| {
            let handle = assets.add(t.clone());
            (handle.id(), t)
        })
        .collect()
}

/// Creates `AssetId` values by adding assets to an `Assets<EvolutionTemplate>` store.
fn evolution_asset_pairs(
    templates: Vec<EvolutionTemplate>,
) -> Vec<(AssetId<EvolutionTemplate>, EvolutionTemplate)> {
    let mut assets = Assets::<EvolutionTemplate>::default();
    templates
        .into_iter()
        .map(|t| {
            let handle = assets.add(t.clone());
            (handle.id(), t)
        })
        .collect()
}

// ── Test 1: build_chip_catalog expands templates into catalog ────

#[test]
fn build_chip_catalog_expands_templates_into_catalog() {
    let mut app = test_app();
    insert_loaded_handles(&mut app);

    // Seed ChipTemplateRegistry with 1 template having common and uncommon slots
    let template = make_template(
        "Piercing",
        3,
        Some(("Basic", piercing_effects(1))),
        Some(("Keen", piercing_effects(2))),
    );
    let pairs = template_asset_pairs(vec![template]);
    app.world_mut()
        .resource_mut::<ChipTemplateRegistry>()
        .seed(&pairs);

    app.update();

    let catalog = app
        .world()
        .get_resource::<ChipCatalog>()
        .expect("ChipCatalog should be inserted after build_chip_catalog runs");
    assert!(
        catalog.get("Basic Piercing").is_some(),
        "catalog should contain 'Basic Piercing'"
    );
    assert!(
        catalog.get("Keen Piercing").is_some(),
        "catalog should contain 'Keen Piercing'"
    );
}

// ── Test 2: build_chip_catalog inserts evolution definitions ─────

#[test]
fn build_chip_catalog_inserts_evolution_definitions() {
    let mut app = test_app();
    insert_loaded_handles(&mut app);

    // Seed EvolutionTemplateRegistry with 1 evolution
    let evolution = make_evolution(
        "Barrage",
        vec![EvolutionIngredient {
            chip_name:       "Piercing Shot".to_owned(),
            stacks_required: 2,
        }],
    );
    let pairs = evolution_asset_pairs(vec![evolution]);
    app.world_mut()
        .resource_mut::<EvolutionTemplateRegistry>()
        .seed(&pairs);

    app.update();

    let catalog = app
        .world()
        .get_resource::<ChipCatalog>()
        .expect("ChipCatalog should be inserted after build_chip_catalog runs");
    assert!(
        catalog.get("Barrage").is_some(),
        "catalog should contain evolution chip 'Barrage'"
    );
}

// ── Test 3: build_chip_catalog extracts recipes from evolution ───

#[test]
fn build_chip_catalog_extracts_recipes_from_evolution_chips() {
    let mut app = test_app();
    insert_loaded_handles(&mut app);

    let evolution = make_evolution(
        "Barrage",
        vec![EvolutionIngredient {
            chip_name:       "Piercing Shot".to_owned(),
            stacks_required: 2,
        }],
    );
    let pairs = evolution_asset_pairs(vec![evolution]);
    app.world_mut()
        .resource_mut::<EvolutionTemplateRegistry>()
        .seed(&pairs);

    app.update();

    let catalog = app
        .world()
        .get_resource::<ChipCatalog>()
        .expect("ChipCatalog should be inserted");
    let recipes = catalog.recipes();
    assert_eq!(recipes.len(), 1, "should have 1 recipe from evolution");
    assert_eq!(
        recipes[0].result_name, "Barrage",
        "recipe result_name should be 'Barrage'"
    );
    assert_eq!(
        recipes[0].ingredients.len(),
        1,
        "recipe should have 1 ingredient"
    );
    assert_eq!(
        recipes[0].ingredients[0].chip_name, "Piercing Shot",
        "ingredient chip_name should be 'Piercing Shot'"
    );
    assert_eq!(
        recipes[0].ingredients[0].stacks_required, 2,
        "ingredient stacks_required should be 2"
    );
}

// ── Test 4: build_chip_catalog processes both templates and evolutions ──

#[test]
fn build_chip_catalog_processes_both_templates_and_evolutions() {
    let mut app = test_app();
    insert_loaded_handles(&mut app);

    // 1 template with 2 slots
    let template = make_template(
        "Piercing",
        3,
        Some(("Basic", piercing_effects(1))),
        Some(("Keen", piercing_effects(2))),
    );
    let template_pairs = template_asset_pairs(vec![template]);
    app.world_mut()
        .resource_mut::<ChipTemplateRegistry>()
        .seed(&template_pairs);

    // 1 evolution
    let evolution = make_evolution(
        "Barrage",
        vec![EvolutionIngredient {
            chip_name:       "Piercing Shot".to_owned(),
            stacks_required: 2,
        }],
    );
    let evolution_pairs = evolution_asset_pairs(vec![evolution]);
    app.world_mut()
        .resource_mut::<EvolutionTemplateRegistry>()
        .seed(&evolution_pairs);

    app.update();

    let catalog = app
        .world()
        .get_resource::<ChipCatalog>()
        .expect("ChipCatalog should be inserted");

    // 2 from template + 1 from evolution = 3
    assert!(
        catalog.get("Basic Piercing").is_some(),
        "should contain 'Basic Piercing'"
    );
    assert!(
        catalog.get("Keen Piercing").is_some(),
        "should contain 'Keen Piercing'"
    );
    assert!(catalog.get("Barrage").is_some(), "should contain 'Barrage'");

    let recipes = catalog.recipes();
    assert_eq!(recipes.len(), 1, "should have 1 recipe from evolution");
}

// ── Test 5: build_chip_catalog with empty registries ────────────

#[test]
fn build_chip_catalog_empty_registries_produces_empty_catalog() {
    let mut app = test_app();
    insert_loaded_handles(&mut app);

    // Both registries are empty (default)

    app.update();

    let catalog = app
        .world()
        .get_resource::<ChipCatalog>()
        .expect("ChipCatalog should be inserted even when empty");
    assert_eq!(
        catalog.ordered_values().count(),
        0,
        "empty registries should produce empty catalog"
    );
    assert_eq!(
        catalog.recipes().len(),
        0,
        "empty registries should produce no recipes"
    );
}

// ── Test 6: build_chip_catalog is idempotent ────────────────────

#[test]
fn build_chip_catalog_is_idempotent() {
    let mut app = test_app();
    insert_loaded_handles(&mut app);

    let template = make_template("Piercing", 3, Some(("Basic", piercing_effects(1))), None);
    let pairs = template_asset_pairs(vec![template]);
    app.world_mut()
        .resource_mut::<ChipTemplateRegistry>()
        .seed(&pairs);

    // First update builds the catalog
    app.update();

    let catalog = app
        .world()
        .get_resource::<ChipCatalog>()
        .expect("ChipCatalog should be inserted");
    assert!(
        catalog.get("Basic Piercing").is_some(),
        "catalog should contain 'Basic Piercing' after first build"
    );

    // Add another template to the registry (simulating a hot-reload scenario)
    let extra_template = make_template("Surge", 2, Some(("Quick", piercing_effects(1))), None);
    let extra_pairs = template_asset_pairs(vec![extra_template]);
    app.world_mut()
        .resource_mut::<ChipTemplateRegistry>()
        .seed(&extra_pairs);

    // Second update should NOT rebuild (Local<bool> guard)
    app.update();

    let catalog = app
        .world()
        .get_resource::<ChipCatalog>()
        .expect("ChipCatalog should still exist");
    // If idempotent, "Quick Surge" should NOT appear because the catalog was already built
    assert!(
        catalog.get("Quick Surge").is_none(),
        "catalog should not contain 'Quick Surge' — second run should be a no-op"
    );
}

// ── Test 7: build_chip_catalog returns zero when handles not loaded ──

#[test]
fn build_chip_catalog_returns_zero_when_handles_not_loaded() {
    let mut app = test_app();

    // Insert handles that are NOT loaded
    let template_handles = RegistryHandles::<ChipTemplate>::new(Handle::default());
    app.insert_resource(template_handles);
    let evolution_handles = RegistryHandles::<EvolutionTemplate>::new(Handle::default());
    app.insert_resource(evolution_handles);

    app.update();

    // ChipCatalog should NOT be inserted when handles are not loaded
    assert!(
        app.world().get_resource::<ChipCatalog>().is_none(),
        "ChipCatalog should not be inserted when handles are not loaded"
    );
}
