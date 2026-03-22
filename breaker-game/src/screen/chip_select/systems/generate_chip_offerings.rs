//! System to generate weighted random chip offerings before the selection screen.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    chips::{
        ChipRegistry,
        definition::Rarity,
        inventory::ChipInventory,
        offering::{OfferingConfig, generate_offerings},
    },
    screen::chip_select::{ChipSelectConfig, resources::ChipOffers},
    shared::GameRng,
};

/// Generates chip offerings using weighted random selection and inserts `ChipOffers`.
///
/// Runs `OnEnter(GameState::ChipSelect)`, before `spawn_chip_select`.
pub(crate) fn generate_chip_offerings(
    mut commands: Commands,
    registry: Res<ChipRegistry>,
    mut inventory: ResMut<ChipInventory>,
    config: Res<ChipSelectConfig>,
    mut rng: ResMut<GameRng>,
) {
    // Build `OfferingConfig` from `ChipSelectConfig` fields
    let rarity_weights = HashMap::from([
        (Rarity::Common, config.rarity_weight_common),
        (Rarity::Uncommon, config.rarity_weight_uncommon),
        (Rarity::Rare, config.rarity_weight_rare),
        (Rarity::Legendary, config.rarity_weight_legendary),
    ]);

    let offering_config = OfferingConfig {
        rarity_weights,
        seen_decay_factor: config.seen_decay_factor,
        offers_per_node: config.offers_per_node,
    };

    // Generate offerings
    let offers = generate_offerings(&registry, &inventory, &offering_config, &mut rng.0);

    // Record each offered chip in inventory for decay tracking
    for chip in &offers {
        inventory.record_offered(&chip.name, config.seen_decay_factor);
    }

    // Insert offers resource
    commands.insert_resource(ChipOffers(offers));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chips::{
            ChipDefinition,
            definition::{AmpEffect, ChipEffect, Rarity},
        },
        screen::chip_select::resources::ChipOffers,
    };

    /// Build a registry with `count` Common chips named `Chip_0`, `Chip_1`, etc.
    fn make_registry(count: usize) -> ChipRegistry {
        let mut registry = ChipRegistry::default();
        for i in 0..count {
            registry.insert(ChipDefinition::test(
                &format!("Chip_{i}"),
                ChipEffect::Amp(AmpEffect::Piercing(1)),
                3,
            ));
        }
        registry
    }

    /// Build a registry with specific rarities for testing weighted selection.
    fn make_mixed_registry() -> ChipRegistry {
        let mut registry = ChipRegistry::default();
        for i in 0..3 {
            registry.insert(ChipDefinition {
                rarity: Rarity::Common,
                ..ChipDefinition::test(
                    &format!("Common_{i}"),
                    ChipEffect::Amp(AmpEffect::Piercing(1)),
                    3,
                )
            });
        }
        registry.insert(ChipDefinition {
            rarity: Rarity::Legendary,
            ..ChipDefinition::test("Legendary_0", ChipEffect::Amp(AmpEffect::Piercing(1)), 3)
        });
        registry
    }

    fn test_app_with_registry(registry: ChipRegistry) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(registry)
            .init_resource::<ChipInventory>()
            .insert_resource(ChipSelectConfig::default())
            .insert_resource(GameRng::from_seed(42))
            .add_systems(Update, generate_chip_offerings);
        app
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
    fn generate_records_offered_chips_in_inventory() {
        let mut app = test_app_with_registry(make_registry(5));
        app.update();

        let offers = app.world().resource::<ChipOffers>();
        let inventory = app.world().resource::<ChipInventory>();

        // Each offered chip should have decay recorded (weight_decay < 1.0).
        for chip in &offers.0 {
            let decay = inventory.weight_decay(&chip.name);
            assert!(
                decay < 1.0,
                "expected offered chip '{}' to have decay < 1.0, got {decay}",
                chip.name
            );
        }
    }

    #[test]
    fn generate_excludes_maxed_chips() {
        let mut registry = ChipRegistry::default();
        let chip_a = ChipDefinition::test(
            "MaxedChip",
            ChipEffect::Amp(AmpEffect::Piercing(1)),
            1, // max_stacks = 1
        );
        let chip_b = ChipDefinition::test(
            "AvailableChip_0",
            ChipEffect::Amp(AmpEffect::Piercing(1)),
            3,
        );
        let chip_c = ChipDefinition::test(
            "AvailableChip_1",
            ChipEffect::Amp(AmpEffect::Piercing(1)),
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
        let names: Vec<&str> = offers.0.iter().map(|d| d.name.as_str()).collect();
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
        let names1: Vec<String> = offers1.0.iter().map(|d| d.name.clone()).collect();

        // Second app with same seed 42
        let mut app2 = test_app_with_registry(make_mixed_registry());
        app2.update();
        let offers2 = app2.world().resource::<ChipOffers>();
        let names2: Vec<String> = offers2.0.iter().map(|d| d.name.clone()).collect();

        assert_eq!(
            names1, names2,
            "same seed should produce identical offerings"
        );
    }
}
