//! `ChipTemplateRegistry` -- `SeedableRegistry` tests.

use bevy::prelude::*;
use rantzsoft_defaults::prelude::SeedableRegistry;

use crate::{
    chips::{
        definition::{ChipTemplate, RaritySlot},
        resources::ChipTemplateRegistry,
    },
    effect_v3::{
        effects::PiercingConfig,
        types::{EffectType, RootNode, StampTarget, Tree},
    },
};

fn make_chip_template(name: &str, max_taken: u32, prefix: &str) -> ChipTemplate {
    ChipTemplate {
        name: name.to_owned(),
        max_taken,
        common: Some(RaritySlot {
            prefix:  prefix.to_owned(),
            effects: vec![RootNode::Stamp(
                StampTarget::Bolt,
                Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
            )],
        }),
        uncommon: None,
        rare: None,
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

// ── Behavior 1: seed() populates from 2 templates ───────────────

#[test]
fn chip_template_registry_seed_populates_from_templates() {
    let pairs = template_asset_pairs(vec![
        make_chip_template("Piercing", 3, "Basic"),
        make_chip_template("Surge", 2, "Quick"),
    ]);

    let mut registry = ChipTemplateRegistry::default();
    registry.seed(&pairs);

    assert_eq!(
        registry.len(),
        2,
        "registry should contain 2 templates after seed"
    );
    assert!(
        registry.get("Piercing").is_some(),
        "registry should contain 'Piercing'"
    );
    assert!(
        registry.get("Surge").is_some(),
        "registry should contain 'Surge'"
    );
}

// ── Behavior 2: seed() clears existing entries ──────────────────

#[test]
fn chip_template_registry_seed_clears_existing() {
    let old_pairs = template_asset_pairs(vec![make_chip_template("Old", 1, "Stale")]);
    let new_pairs = template_asset_pairs(vec![make_chip_template("New", 2, "Fresh")]);

    let mut registry = ChipTemplateRegistry::default();
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

// ── Behavior 3: update_single() upserts by name ────────────────

#[test]
fn chip_template_registry_update_single_upserts_by_name() {
    let pairs = template_asset_pairs(vec![make_chip_template("Piercing", 3, "Basic")]);

    let mut registry = ChipTemplateRegistry::default();
    registry.seed(&pairs);

    let (original_id, _) = &pairs[0];
    let updated = make_chip_template("Piercing", 5, "Basic");
    registry.update_single(*original_id, &updated);

    let (_, template) = registry.get("Piercing").expect("'Piercing' should exist");
    assert_eq!(
        template.max_taken, 5,
        "max_taken should be updated to 5 after update_single"
    );
}

// ── Behavior 4: update_single() inserts new ────────────────────

#[test]
fn chip_template_registry_update_single_inserts_new() {
    let pairs = template_asset_pairs(vec![make_chip_template("Piercing", 3, "Basic")]);
    let new_pairs = template_asset_pairs(vec![make_chip_template("Surge", 2, "Quick")]);

    let mut registry = ChipTemplateRegistry::default();
    registry.seed(&pairs);
    assert_eq!(registry.len(), 1);

    let (new_id, _) = &new_pairs[0];
    let new_template = make_chip_template("Surge", 2, "Quick");
    registry.update_single(*new_id, &new_template);

    assert_eq!(
        registry.len(),
        2,
        "registry should contain 2 templates after inserting new"
    );
    assert!(
        registry.get("Surge").is_some(),
        "'Surge' should be present after update_single"
    );
}

// ── Behavior 5: asset_dir() returns correct path ────────────────

#[test]
fn chip_template_registry_asset_dir() {
    assert_eq!(
        ChipTemplateRegistry::asset_dir(),
        "chips/standard",
        "asset_dir() should return \"chips/standard\""
    );
}

// ── Behavior 6: extensions() returns correct extension ──────────

#[test]
fn chip_template_registry_extensions() {
    assert_eq!(
        ChipTemplateRegistry::extensions(),
        &["chip.ron"],
        "extensions() should return [\"chip.ron\"]"
    );
}
