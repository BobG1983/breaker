//! Tests for `generate_chip_offerings` weighted chip offering generation.

use bevy::prelude::*;

use super::*;
use crate::{
    chips::{
        ChipDefinition, Recipe,
        definition::{EvolutionIngredient, Rarity},
    },
    effect_v3::{
        effects::PiercingConfig,
        types::{EffectType, RootNode, StampTarget, Tree},
    },
    state::run::node::{ActiveNodeLayout, definition::NodePool},
};

/// Build a registry with `count` Common chips named `Chip_0`, `Chip_1`, etc.
fn make_registry(count: usize) -> ChipCatalog {
    let mut registry = ChipCatalog::default();
    for i in 0..count {
        registry.insert(ChipDefinition::test(
            &format!("Chip_{i}"),
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
            3,
        ));
    }
    registry
}

/// Build a registry with specific rarities for testing weighted selection.
fn make_mixed_registry() -> ChipCatalog {
    let mut registry = ChipCatalog::default();
    for i in 0..3 {
        registry.insert(ChipDefinition {
            rarity: Rarity::Common,
            ..ChipDefinition::test(
                &format!("Common_{i}"),
                Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
                3,
            )
        });
    }
    registry.insert(ChipDefinition {
        rarity: Rarity::Rare,
        ..ChipDefinition::test(
            "Rare_0",
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
            3,
        )
    });
    registry
}

fn test_app_with_registry(registry: ChipCatalog) -> App {
    TestAppBuilder::new()
        .insert_resource(registry)
        .with_resource::<ChipInventory>()
        .insert_resource(ChipSelectConfig::default())
        .insert_resource(GameRng::from_seed(42))
        .with_system(Update, generate_chip_offerings)
        .build()
}

#[test]
fn generate_inserts_chip_offers_resource() {
    let mut app = test_app_with_registry(make_registry(5));
    app.update();

    // The system should insert ChipOffers after running.
    // This will panic if the resource does not exist.
    let offers = app.world().resource::<ChipOffers>();
    assert!(
        !offers.0.is_empty(),
        "expected ChipOffers to be non-empty after generation with 5 chips"
    );
}

#[test]
fn generate_offers_correct_count() {
    let mut app = test_app_with_registry(make_registry(5));
    app.update();

    let offers = app.world().resource::<ChipOffers>();
    assert_eq!(
        offers.0.len(),
        3,
        "expected 3 offers (default offers_per_node), got {}",
        offers.0.len()
    );
}

#[test]
fn generate_does_not_apply_decay() {
    let mut app = test_app_with_registry(make_registry(5));
    app.update();

    let offers = app.world().resource::<ChipOffers>();
    let inventory = app.world().resource::<ChipInventory>();

    // Generation should NOT apply decay -- decay is deferred to confirmation
    // or timer expiry. All offered chips must have weight_decay == 1.0.
    for offering in &offers.0 {
        let decay = inventory.weight_decay(offering.name());
        assert!(
            (decay - 1.0).abs() < f32::EPSILON,
            "expected offered chip '{}' to have no decay (1.0), got {decay}",
            offering.name()
        );
    }
}

#[test]
fn generate_excludes_maxed_chips() {
    let mut registry = ChipCatalog::default();
    let chip_a = ChipDefinition::test(
        "MaxedChip",
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
        1, // max_stacks = 1
    );
    let chip_b = ChipDefinition::test(
        "AvailableChip_0",
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
        3,
    );
    let chip_c = ChipDefinition::test(
        "AvailableChip_1",
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
        3,
    );
    registry.insert(chip_a.clone());
    registry.insert(chip_b);
    registry.insert(chip_c);

    let mut inventory = ChipInventory::default();
    // Max out chip_a (1/1 stacks)
    let _ = inventory.add_chip("MaxedChip", &chip_a);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(registry)
        .insert_resource(inventory)
        .insert_resource(ChipSelectConfig::default())
        .insert_resource(GameRng::from_seed(42))
        .add_systems(Update, generate_chip_offerings);
    app.update();

    let offers = app.world().resource::<ChipOffers>();
    let names: Vec<&str> = offers.0.iter().map(ChipOffering::name).collect();
    assert!(
        !names.contains(&"MaxedChip"),
        "maxed chip should not appear in offerings, got: {names:?}"
    );
}

#[test]
fn generate_deterministic_with_same_seed() {
    // First app with seed 42
    let mut app1 = test_app_with_registry(make_mixed_registry());
    app1.update();
    let offers1 = app1.world().resource::<ChipOffers>();
    let names1: Vec<String> = offers1.0.iter().map(|o| o.name().to_owned()).collect();

    // Second app with same seed 42
    let mut app2 = test_app_with_registry(make_mixed_registry());
    app2.update();
    let offers2 = app2.world().resource::<ChipOffers>();
    let names2: Vec<String> = offers2.0.iter().map(|o| o.name().to_owned()).collect();

    assert_eq!(
        names1, names2,
        "same seed should produce identical offerings"
    );
}

// --- B12d: Evolution offering generation tests using ChipCatalog ---

fn make_test_layout(pool: NodePool) -> ActiveNodeLayout {
    ActiveNodeLayout(NodeLayout {
        name: "test_layout".to_owned(),
        timer_secs: 60.0,
        cols: 1,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec!["S".to_owned()]],
        pool,
        entity_scale: 1.0,
        locks: None,
        sequences: None,
    })
}

/// Test app for evolution offering tests using unified `ChipCatalog`.
///
/// The `ChipCatalog` contains 5 normal chips plus the "Barrage" evolution
/// chip definition and a recipe requiring "Piercing Shot" x2.
/// The `ActiveNodeLayout` pool controls whether evolutions are offered.
fn test_app_for_evolution(pool: NodePool, evolution_eligible: bool) -> App {
    let ps_def = ChipDefinition::test(
        "Piercing Shot",
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
        5,
    )
    .with_template("Piercing Shot");
    let mut inventory = ChipInventory::default();
    if evolution_eligible {
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
    }

    // Build unified ChipCatalog with 5 normal chips + Barrage evolution + recipe
    let mut registry = make_registry(5);
    registry.insert(ChipDefinition {
        name:          "Barrage".into(),
        description:   "Combined piercing power".into(),
        rarity:        Rarity::Evolution,
        max_stacks:    1,
        effects:       vec![RootNode::Stamp(
            StampTarget::Bolt,
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 5 })),
        )],
        ingredients:   Some(vec![EvolutionIngredient {
            chip_name:       "Piercing Shot".into(),
            stacks_required: 2,
        }]),
        template_name: None,
    });
    registry.insert_recipe(Recipe {
        ingredients: vec![EvolutionIngredient {
            chip_name:       "Piercing Shot".into(),
            stacks_required: 2,
        }],
        result_name: "Barrage".to_owned(),
    });

    TestAppBuilder::new()
        .insert_resource(registry)
        .insert_resource(inventory)
        .insert_resource(ChipSelectConfig::default())
        .insert_resource(GameRng::from_seed(42))
        .insert_resource(make_test_layout(pool))
        .with_system(Update, generate_chip_offerings)
        .build()
}

// --- Behavior 13: generate_chip_offerings on boss node with eligible recipe ---

#[test]
fn boss_node_eligible_evolution_appears_in_offers() {
    let mut app = test_app_for_evolution(NodePool::Boss, true);
    app.update();

    let offers = app.world().resource::<ChipOffers>();
    let has_evolution = offers
        .0
        .iter()
        .any(|o| matches!(o, ChipOffering::Evolution { .. }));
    assert!(
        has_evolution,
        "boss node with eligible evolution should include at least one Evolution offering, got: {:?}",
        offers.0.iter().map(ChipOffering::name).collect::<Vec<_>>()
    );

    let evo_names: Vec<&str> = offers
        .0
        .iter()
        .filter_map(|o| match o {
            ChipOffering::Evolution { result, .. } => Some(result.name.as_str()),
            ChipOffering::Normal(_) => None,
        })
        .collect();
    assert!(
        evo_names.contains(&"Barrage"),
        "evolution offering should have result name 'Barrage', got: {evo_names:?}"
    );
}

// --- Behavior 14: generate_chip_offerings on non-boss node has no evolution ---

#[test]
fn non_boss_node_has_no_evolution_offerings() {
    let mut app = test_app_for_evolution(NodePool::Active, true);
    app.update();

    let offers = app.world().resource::<ChipOffers>();
    let has_evolution = offers
        .0
        .iter()
        .any(|o| matches!(o, ChipOffering::Evolution { .. }));
    assert!(
        !has_evolution,
        "non-boss node should have no Evolution offerings, got: {:?}",
        offers.0.iter().map(ChipOffering::name).collect::<Vec<_>>()
    );

    assert!(
        offers
            .0
            .iter()
            .all(|o| matches!(o, ChipOffering::Normal(_))),
        "all offerings on non-boss node should be Normal"
    );
}

#[test]
fn boss_node_no_eligible_evolutions_all_normal() {
    let mut app = test_app_for_evolution(NodePool::Boss, false);
    app.update();

    let offers = app.world().resource::<ChipOffers>();
    assert!(
        offers
            .0
            .iter()
            .all(|o| matches!(o, ChipOffering::Normal(_))),
        "boss node with no eligible evolutions should only have Normal offerings"
    );
}

// --- Behavior 15: remaining slots filled with normal offerings ---

#[test]
fn boss_node_remaining_slots_filled_with_normal() {
    let mut app = test_app_for_evolution(NodePool::Boss, true);
    app.update();

    let offers = app.world().resource::<ChipOffers>();
    assert_eq!(
        offers.0.len(),
        3,
        "offers_per_node=3, total should be 3 (1 evolution + 2 normal), got {}",
        offers.0.len()
    );

    let evo_count = offers
        .0
        .iter()
        .filter(|o| matches!(o, ChipOffering::Evolution { .. }))
        .count();
    let normal_count = offers
        .0
        .iter()
        .filter(|o| matches!(o, ChipOffering::Normal(_)))
        .count();
    assert_eq!(
        evo_count, 1,
        "expected exactly 1 evolution offering, got {evo_count}"
    );
    assert_eq!(
        normal_count, 2,
        "expected 2 normal offerings to fill remaining slots, got {normal_count}"
    );
}

// --- Behavior: All slots filled by evolutions when eligible count >= offers_per_node ---

/// Creates an evolution chip definition with a single ingredient recipe.
fn make_evolution_def(
    name: &str,
    description: &str,
    effect: EffectType,
    ingredient_name: &str,
) -> ChipDefinition {
    ChipDefinition {
        name:          name.into(),
        description:   description.into(),
        rarity:        Rarity::Evolution,
        max_stacks:    1,
        effects:       vec![RootNode::Stamp(StampTarget::Bolt, Tree::Fire(effect))],
        ingredients:   Some(vec![EvolutionIngredient {
            chip_name:       ingredient_name.into(),
            stacks_required: 2,
        }]),
        template_name: None,
    }
}

/// Builds the registry and inventory for 3 distinct eligible evolutions.
fn make_3_evolution_registry_and_inventory() -> (ChipCatalog, ChipInventory) {
    let ps_def = ChipDefinition::test(
        "Piercing Shot",
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
        5,
    )
    .with_template("Piercing Shot");
    let sb_def = ChipDefinition::test(
        "Speed Boost",
        Tree::Fire(EffectType::SpeedBoost(
            crate::effect_v3::effects::SpeedBoostConfig {
                multiplier: ordered_float::OrderedFloat(1.5),
            },
        )),
        5,
    )
    .with_template("Speed Boost");
    let db_def = ChipDefinition::test(
        "Damage Boost",
        Tree::Fire(EffectType::DamageBoost(
            crate::effect_v3::effects::DamageBoostConfig {
                multiplier: ordered_float::OrderedFloat(0.5),
            },
        )),
        5,
    )
    .with_template("Damage Boost");

    let mut inventory = ChipInventory::default();
    let _ = inventory.add_chip("Piercing Shot", &ps_def);
    let _ = inventory.add_chip("Piercing Shot", &ps_def);
    let _ = inventory.add_chip("Speed Boost", &sb_def);
    let _ = inventory.add_chip("Speed Boost", &sb_def);
    let _ = inventory.add_chip("Damage Boost", &db_def);
    let _ = inventory.add_chip("Damage Boost", &db_def);

    let mut registry = make_registry(5);
    registry.insert(make_evolution_def(
        "Barrage",
        "Combined piercing",
        EffectType::Piercing(PiercingConfig { charges: 5 }),
        "Piercing Shot",
    ));
    registry.insert(make_evolution_def(
        "Velocity",
        "Combined speed",
        EffectType::SpeedBoost(crate::effect_v3::effects::SpeedBoostConfig {
            multiplier: ordered_float::OrderedFloat(2.0),
        }),
        "Speed Boost",
    ));
    registry.insert(make_evolution_def(
        "Devastation",
        "Combined damage",
        EffectType::DamageBoost(crate::effect_v3::effects::DamageBoostConfig {
            multiplier: ordered_float::OrderedFloat(2.0),
        }),
        "Damage Boost",
    ));

    for (ingredient, result) in [
        ("Piercing Shot", "Barrage"),
        ("Speed Boost", "Velocity"),
        ("Damage Boost", "Devastation"),
    ] {
        registry.insert_recipe(Recipe {
            ingredients: vec![EvolutionIngredient {
                chip_name:       ingredient.into(),
                stacks_required: 2,
            }],
            result_name: result.to_owned(),
        });
    }

    (registry, inventory)
}

/// Setup: 3 distinct evolution recipes, all with satisfied ingredients,
/// on a Boss node with `offers_per_node`=3.
fn app_with_3_eligible_evolutions() -> App {
    let (registry, inventory) = make_3_evolution_registry_and_inventory();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(registry)
        .insert_resource(inventory)
        .insert_resource(ChipSelectConfig::default())
        .insert_resource(GameRng::from_seed(42))
        .insert_resource(make_test_layout(NodePool::Boss))
        .add_systems(Update, generate_chip_offerings);
    app
}

#[test]
fn boss_node_all_slots_filled_by_evolutions_when_3_eligible() {
    let mut app = app_with_3_eligible_evolutions();
    app.update();

    let offers = app.world().resource::<ChipOffers>();
    assert_eq!(
        offers.0.len(),
        3,
        "expected exactly 3 offers (all evolution), got {}",
        offers.0.len()
    );

    let evo_count = offers
        .0
        .iter()
        .filter(|o| matches!(o, ChipOffering::Evolution { .. }))
        .count();
    let normal_count = offers
        .0
        .iter()
        .filter(|o| matches!(o, ChipOffering::Normal(_)))
        .count();
    assert_eq!(
        evo_count, 3,
        "all 3 slots should be evolution offers, got {evo_count}"
    );
    assert_eq!(
        normal_count, 0,
        "no normal offers expected when evolutions fill all slots, got {normal_count}"
    );
}

#[test]
fn boss_node_3_eligible_evolutions_has_correct_names() {
    let mut app = app_with_3_eligible_evolutions();
    app.update();

    // Verify all 3 evolution result names are present
    let offers = app.world().resource::<ChipOffers>();
    let mut evo_names: Vec<&str> = offers
        .0
        .iter()
        .filter_map(|o| match o {
            ChipOffering::Evolution { result, .. } => Some(result.name.as_str()),
            ChipOffering::Normal(_) => None,
        })
        .collect();
    evo_names.sort_unstable();
    assert_eq!(
        evo_names,
        vec!["Barrage", "Devastation", "Velocity"],
        "expected all 3 evolution results, got {evo_names:?}"
    );
}
