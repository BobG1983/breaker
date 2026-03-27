//! System to generate weighted random chip offerings before the selection screen.

use std::collections::HashMap;

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    chips::{
        ChipCatalog,
        definition::Rarity,
        inventory::ChipInventory,
        offering::{OfferingConfig, generate_offerings},
    },
    run::node::{ActiveNodeLayout, definition::NodePool},
    screen::chip_select::{
        ChipSelectConfig,
        resources::{ChipOffering, ChipOffers},
    },
    shared::GameRng,
};

/// Bundled parameters for chip offering generation.
#[derive(SystemParam)]
pub(crate) struct ChipOfferingParams<'w, 's> {
    commands: Commands<'w, 's>,
    registry: Res<'w, ChipCatalog>,
    inventory: Res<'w, ChipInventory>,
    config: Res<'w, ChipSelectConfig>,
    rng: ResMut<'w, GameRng>,
    active_layout: Option<Res<'w, ActiveNodeLayout>>,
}

/// Generates chip offerings using weighted random selection and inserts `ChipOffers`.
///
/// Runs `OnEnter(GameState::ChipSelect)`, before `spawn_chip_select`.
/// On boss nodes with eligible evolutions, evolution offerings take priority
/// and remaining slots are filled with normal offerings.
pub(crate) fn generate_chip_offerings(mut params: ChipOfferingParams) {
    // Build rarity weight map from config
    let rarity_weights = HashMap::from([
        (Rarity::Common, params.config.rarity_weight_common),
        (Rarity::Uncommon, params.config.rarity_weight_uncommon),
        (Rarity::Rare, params.config.rarity_weight_rare),
        (Rarity::Legendary, params.config.rarity_weight_legendary),
    ]);

    // Check for boss node with eligible evolutions
    let mut evolution_offers: Vec<ChipOffering> = Vec::new();
    if let Some(layout) = &params.active_layout {
        if layout.0.pool == NodePool::Boss {
            let eligible = params.registry.eligible_recipes(&params.inventory);
            for recipe in eligible.iter().take(params.config.offers_per_node) {
                if let Some(result_def) = params.registry.get(&recipe.result_name) {
                    evolution_offers.push(ChipOffering::Evolution {
                        ingredients: recipe.ingredients.clone(),
                        result: result_def.clone(),
                    });
                }
            }
        }
    }

    // Fill remaining slots with normal offerings
    let remaining_slots = params
        .config
        .offers_per_node
        .saturating_sub(evolution_offers.len());
    let offering_config = OfferingConfig {
        rarity_weights,
        offers_per_node: remaining_slots,
    };
    let normal_offers = generate_offerings(
        &params.registry,
        &params.inventory,
        &offering_config,
        &mut params.rng.0,
    );

    // Combine: evolutions first, then normal
    let mut chip_offers: Vec<ChipOffering> = evolution_offers;
    chip_offers.extend(normal_offers.into_iter().map(ChipOffering::Normal));

    // Insert offers resource
    params.commands.insert_resource(ChipOffers(chip_offers));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chips::{
            ChipDefinition,
            definition::{EvolutionIngredient, Rarity},
        },
        effect::definition::{Effect, EffectNode, RootEffect, Target},
        run::node::{
            ActiveNodeLayout,
            NodeLayout,
            definition::NodePool,
        },
    };

    /// Build a registry with `count` Common chips named `Chip_0`, `Chip_1`, etc.
    fn make_registry(count: usize) -> ChipCatalog {
        let mut registry = ChipCatalog::default();
        for i in 0..count {
            registry.insert(ChipDefinition::test(
                &format!("Chip_{i}"),
                EffectNode::Do(Effect::Piercing(1)),
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
                    EffectNode::Do(Effect::Piercing(1)),
                    3,
                )
            });
        }
        registry.insert(ChipDefinition {
            rarity: Rarity::Legendary,
            ..ChipDefinition::test("Legendary_0", EffectNode::Do(Effect::Piercing(1)), 3)
        });
        registry
    }

    fn test_app_with_registry(registry: ChipCatalog) -> App {
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
    fn generate_does_not_apply_decay() {
        let mut app = test_app_with_registry(make_registry(5));
        app.update();

        let offers = app.world().resource::<ChipOffers>();
        let inventory = app.world().resource::<ChipInventory>();

        // Generation should NOT apply decay — decay is deferred to confirmation
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
            EffectNode::Do(Effect::Piercing(1)),
            1, // max_stacks = 1
        );
        let chip_b =
            ChipDefinition::test("AvailableChip_0", EffectNode::Do(Effect::Piercing(1)), 3);
        let chip_c =
            ChipDefinition::test("AvailableChip_1", EffectNode::Do(Effect::Piercing(1)), 3);
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

    use crate::chips::Recipe;

    fn make_test_layout(pool: NodePool) -> ActiveNodeLayout {
        ActiveNodeLayout(NodeLayout {
            name: "test_layout".to_owned(),
            timer_secs: 60.0,
            cols: 1,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['S']],
            pool,
            entity_scale: 1.0,
        })
    }

    /// Test app for evolution offering tests using unified `ChipCatalog`.
    ///
    /// The `ChipCatalog` contains 5 normal chips plus the "Barrage" evolution
    /// chip definition and a recipe requiring "Piercing Shot" x2.
    /// The `ActiveNodeLayout` pool controls whether evolutions are offered.
    fn test_app_for_evolution(
        pool: NodePool,
        evolution_eligible: bool,
    ) -> App {
        let mut app = App::new();

        let ps_def = ChipDefinition::test("Piercing Shot", EffectNode::Do(Effect::Piercing(1)), 5)
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
            name: "Barrage".into(),
            description: "Combined piercing power".into(),
            rarity: Rarity::Evolution,
            max_stacks: 1,
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::Do(Effect::Piercing(5))],
            }],
            ingredients: Some(vec![EvolutionIngredient {
                chip_name: "Piercing Shot".into(),
                stacks_required: 2,
            }]),
            template_name: None,
        });
        registry.insert_recipe(Recipe {
            ingredients: vec![EvolutionIngredient {
                chip_name: "Piercing Shot".into(),
                stacks_required: 2,
            }],
            result_name: "Barrage".to_owned(),
        });

        app.add_plugins(MinimalPlugins)
            .insert_resource(registry)
            .insert_resource(inventory)
            .insert_resource(ChipSelectConfig::default())
            .insert_resource(GameRng::from_seed(42))
            .insert_resource(make_test_layout(pool))
            .add_systems(Update, generate_chip_offerings);
        app
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
}
