//! `EvolutionTemplateRegistry` -- `SeedableRegistry` tests.

use bevy::prelude::*;
use rantzsoft_defaults::prelude::SeedableRegistry;

use crate::{
    chips::{
        definition::{EvolutionIngredient, EvolutionTemplate},
        resources::EvolutionTemplateRegistry,
    },
    effect_v3::{
        effects::PiercingConfig,
        types::{EffectType, RootNode, StampTarget, Tree},
    },
};

fn make_evolution_template(name: &str) -> EvolutionTemplate {
    EvolutionTemplate {
        name:        name.to_owned(),
        description: String::new(),
        max_stacks:  1,
        effects:     vec![RootNode::Stamp(
            StampTarget::Bolt,
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 5 })),
        )],
        ingredients: vec![EvolutionIngredient {
            chip_name:       "Piercing Shot".to_owned(),
            stacks_required: 2,
        }],
    }
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

// ── Behavior 1: seed() populates from 2 evolution templates ─────

#[test]
fn evolution_registry_seed_populates_from_templates() {
    let pairs = evolution_asset_pairs(vec![
        make_evolution_template("Barrage"),
        make_evolution_template("Supernova"),
    ]);

    let mut registry = EvolutionTemplateRegistry::default();
    registry.seed(&pairs);

    assert_eq!(
        registry.len(),
        2,
        "registry should contain 2 evolution templates"
    );
    assert!(
        registry.get("Barrage").is_some(),
        "'Barrage' should be present"
    );
    assert!(
        registry.get("Supernova").is_some(),
        "'Supernova' should be present"
    );
}

// ── Behavior 2: seed() clears existing ──────────────────────────

#[test]
fn evolution_registry_seed_clears_existing() {
    let old_pairs = evolution_asset_pairs(vec![make_evolution_template("Old")]);
    let new_pairs = evolution_asset_pairs(vec![make_evolution_template("New")]);

    let mut registry = EvolutionTemplateRegistry::default();
    registry.seed(&old_pairs);
    assert_eq!(registry.len(), 1);
    assert!(registry.get("Old").is_some());

    registry.seed(&new_pairs);

    assert_eq!(registry.len(), 1, "after re-seed, only 'New' should remain");
    assert!(
        registry.get("New").is_some(),
        "'New' should be present after re-seed"
    );
    assert!(
        registry.get("Old").is_none(),
        "'Old' should be gone after re-seed"
    );
}

// ── Behavior 3: update_single() upserts ─────────────────────────

#[test]
fn evolution_registry_update_single_upserts() {
    let pairs = evolution_asset_pairs(vec![make_evolution_template("Barrage")]);

    let mut registry = EvolutionTemplateRegistry::default();
    registry.seed(&pairs);

    let (original_id, _) = &pairs[0];
    let mut updated = make_evolution_template("Barrage");
    updated.max_stacks = 3;
    registry.update_single(*original_id, &updated);

    let (_, template) = registry.get("Barrage").expect("'Barrage' should exist");
    assert_eq!(
        template.max_stacks, 3,
        "max_stacks should be updated to 3 after update_single"
    );
}

// ── Behavior 4: asset_dir() returns correct path ────────────────

#[test]
fn evolution_registry_asset_dir() {
    assert_eq!(
        EvolutionTemplateRegistry::asset_dir(),
        "chips/evolutions",
        "asset_dir() should return \"chips/evolutions\""
    );
}

// ── Behavior 5: extensions() returns correct extension ──────────

#[test]
fn evolution_registry_extensions() {
    assert_eq!(
        EvolutionTemplateRegistry::extensions(),
        &["evolution.ron"],
        "extensions() should return [\"evolution.ron\"]"
    );
}
