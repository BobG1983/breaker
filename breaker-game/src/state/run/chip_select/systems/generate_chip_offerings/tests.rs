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
    shared::rng::ChipRng,
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
        .insert_resource(ChipRng::from_seed(42))
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
        .insert_resource(ChipRng::from_seed(42))
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
        .insert_resource(ChipRng::from_seed(42))
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
        .insert_resource(ChipRng::from_seed(42))
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

// ════════════════════════════════════════════════════════════════════════════
// Wave 2C Group C — ChipRng migration tests (Behaviors 12b, 13, 14)
// ════════════════════════════════════════════════════════════════════════════

/// Behavior 12b: `generate_chip_offerings` reads `ChipRng`, NOT `GameRng`.
///
/// At RED: `generate_chip_offerings` reads `GameRng`. App1's `GameRng::from_seed(99)`
/// vs App2's `GameRng::from_seed(42)` produce different sequences → names differ →
/// "names are IDENTICAL" assertion FAILS. After GREEN migration to `ChipRng`, both
/// apps draw from `ChipRng::from_seed(42)` → names match.
#[test]
fn generate_chip_offerings_reads_chip_rng_not_game_rng() {
    use rand::Rng;

    // App1: ChipRng=42, GameRng=99 (mismatched)
    let names1: Vec<String> = {
        let mut app = TestAppBuilder::new()
            .insert_resource(make_mixed_registry())
            .with_resource::<ChipInventory>()
            .insert_resource(ChipSelectConfig::default())
            .insert_resource(ChipRng::from_seed(42))
            .insert_resource(GameRng::from_seed(99))
            .with_system(Update, generate_chip_offerings)
            .build();
        app.update();
        app.world()
            .resource::<ChipOffers>()
            .0
            .iter()
            .map(|o| o.name().to_owned())
            .collect()
    };

    // App2: ChipRng=42, GameRng=42 (GameRng matches ChipRng seed)
    let names2: Vec<String> = {
        let mut app = TestAppBuilder::new()
            .insert_resource(make_mixed_registry())
            .with_resource::<ChipInventory>()
            .insert_resource(ChipSelectConfig::default())
            .insert_resource(ChipRng::from_seed(42))
            .insert_resource(GameRng::from_seed(42))
            .with_system(Update, generate_chip_offerings)
            .build();
        app.update();
        app.world()
            .resource::<ChipOffers>()
            .0
            .iter()
            .map(|o| o.name().to_owned())
            .collect()
    };

    assert_eq!(
        names1, names2,
        "generate_chip_offerings must draw from ChipRng (same seed=42) in both apps, \
         not from GameRng (different seeds 99 vs 42); offerings must match"
    );
}

/// Behavior 13 primary: same `ChipRng` seed produces identical offerings (determinism).
/// At RED: both apps use `GameRng::from_seed(42)` → names match trivially (passes at RED).
/// Edge-case assertion (App3 differs) fails at RED because `ChipRng::from_seed(43)` is ignored
/// and `GameRng::from_seed(42)` is used for all three apps.
#[test]
fn generate_chip_offerings_same_chip_rng_seed_produces_identical_offerings() {
    let names_seed42_a: Vec<String> = {
        let mut app = TestAppBuilder::new()
            .insert_resource(make_mixed_registry())
            .with_resource::<ChipInventory>()
            .insert_resource(ChipSelectConfig::default())
            .insert_resource(ChipRng::from_seed(42))
            .insert_resource(GameRng::from_seed(42))
            .with_system(Update, generate_chip_offerings)
            .build();
        app.update();
        app.world()
            .resource::<ChipOffers>()
            .0
            .iter()
            .map(|o| o.name().to_owned())
            .collect()
    };

    let names_seed42_b: Vec<String> = {
        let mut app = TestAppBuilder::new()
            .insert_resource(make_mixed_registry())
            .with_resource::<ChipInventory>()
            .insert_resource(ChipSelectConfig::default())
            .insert_resource(ChipRng::from_seed(42))
            .insert_resource(GameRng::from_seed(42))
            .with_system(Update, generate_chip_offerings)
            .build();
        app.update();
        app.world()
            .resource::<ChipOffers>()
            .0
            .iter()
            .map(|o| o.name().to_owned())
            .collect()
    };

    assert_eq!(
        names_seed42_a, names_seed42_b,
        "same ChipRng seed must produce identical offerings across independent apps"
    );
}

/// Behavior 13 edge case: different `ChipRng` seed must produce different offerings.
/// At RED: App3 uses `ChipRng::from_seed(43)` which is ignored — `GameRng::from_seed(42)`
/// drives all three → names match baseline → this assertion FAILS (primary RED trigger
/// for Behavior 13).
#[test]
fn generate_chip_offerings_different_chip_rng_seed_produces_different_offerings() {
    let names_seed42: Vec<String> = {
        let mut app = TestAppBuilder::new()
            .insert_resource(make_mixed_registry())
            .with_resource::<ChipInventory>()
            .insert_resource(ChipSelectConfig::default())
            .insert_resource(ChipRng::from_seed(42))
            .insert_resource(GameRng::from_seed(42))
            .with_system(Update, generate_chip_offerings)
            .build();
        app.update();
        app.world()
            .resource::<ChipOffers>()
            .0
            .iter()
            .map(|o| o.name().to_owned())
            .collect()
    };

    let names_seed43: Vec<String> = {
        // App3: ChipRng=43 (different seed), GameRng=42 (same as baseline)
        let mut app = TestAppBuilder::new()
            .insert_resource(make_mixed_registry())
            .with_resource::<ChipInventory>()
            .insert_resource(ChipSelectConfig::default())
            .insert_resource(ChipRng::from_seed(43))
            .insert_resource(GameRng::from_seed(42))
            .with_system(Update, generate_chip_offerings)
            .build();
        app.update();
        app.world()
            .resource::<ChipOffers>()
            .0
            .iter()
            .map(|o| o.name().to_owned())
            .collect()
    };

    assert_ne!(
        names_seed42, names_seed43,
        "different ChipRng seeds (42 vs 43) must produce different offerings; \
         at least one offering name must change"
    );
}

/// Behavior 14: Greed boost still applied after migration to `ChipRng`.
///
/// Note: existing `greed_changes_offer_sequence_end_to_end` covers this with
/// `GameRng`. This test mirrors it with `ChipRng` inserted alongside `GameRng`
/// (required at RED so the unmigrated signature resolves). After GREEN migration,
/// the system draws from `ChipRng`, and the Greed boost still shifts rarity
/// weights → offerings differ from baseline.
#[test]
fn generate_chip_offerings_greed_boost_still_applied_after_chip_rng_migration() {
    use crate::mutators::protocols::greed::{GreedConfig, GreedStacks};

    let baseline_names: Vec<String> = {
        let mut app = TestAppBuilder::new()
            .insert_resource(make_mixed_registry())
            .with_resource::<ChipInventory>()
            .insert_resource(ChipSelectConfig::default())
            .insert_resource(GameRng::from_seed(42))
            .insert_resource(ChipRng::from_seed(42))
            .with_system(Update, generate_chip_offerings)
            .build();
        app.update();
        app.world()
            .resource::<ChipOffers>()
            .0
            .iter()
            .map(|o| o.name().to_owned())
            .collect()
    };

    let greed_names: Vec<String> = {
        let mut app = TestAppBuilder::new()
            .insert_resource(make_mixed_registry())
            .with_resource::<ChipInventory>()
            .insert_resource(ChipSelectConfig::default())
            .insert_resource(GameRng::from_seed(42))
            .insert_resource(ChipRng::from_seed(42))
            .insert_resource(GreedStacks { skips: 10 })
            .insert_resource(GreedConfig {
                rarity_boost_per_skip: 5.0,
            })
            .with_system(Update, generate_chip_offerings)
            .build();
        app.update();
        app.world()
            .resource::<ChipOffers>()
            .0
            .iter()
            .map(|o| o.name().to_owned())
            .collect()
    };

    assert_ne!(
        greed_names, baseline_names,
        "Greed boost (10 skips × 5.0) must shift rarity weights enough to change \
         the chip offering sequence even after ChipRng migration"
    );
}

/// Behavior 14 edge case: `GreedStacks { skips: 0 }` with `GreedConfig` present
/// must yield identical offerings to the no-Greed baseline.
#[test]
fn generate_chip_offerings_zero_greed_stacks_with_chip_rng_matches_baseline() {
    use crate::mutators::protocols::greed::{GreedConfig, GreedStacks};

    let baseline_names: Vec<String> = {
        let mut app = TestAppBuilder::new()
            .insert_resource(make_mixed_registry())
            .with_resource::<ChipInventory>()
            .insert_resource(ChipSelectConfig::default())
            .insert_resource(GameRng::from_seed(42))
            .insert_resource(ChipRng::from_seed(42))
            .with_system(Update, generate_chip_offerings)
            .build();
        app.update();
        app.world()
            .resource::<ChipOffers>()
            .0
            .iter()
            .map(|o| o.name().to_owned())
            .collect()
    };

    let zero_greed_names: Vec<String> = {
        let mut app = TestAppBuilder::new()
            .insert_resource(make_mixed_registry())
            .with_resource::<ChipInventory>()
            .insert_resource(ChipSelectConfig::default())
            .insert_resource(GameRng::from_seed(42))
            .insert_resource(ChipRng::from_seed(42))
            .insert_resource(GreedStacks { skips: 0 })
            .insert_resource(GreedConfig {
                rarity_boost_per_skip: 5.0,
            })
            .with_system(Update, generate_chip_offerings)
            .build();
        app.update();
        app.world()
            .resource::<ChipOffers>()
            .0
            .iter()
            .map(|o| o.name().to_owned())
            .collect()
    };

    assert_eq!(
        zero_greed_names, baseline_names,
        "GreedStacks {{ skips: 0 }} with GreedConfig must produce identical offerings \
         to the no-Greed baseline (same seed)"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Group D — Greed rarity-boost integration (Behaviors 19–26)
// ════════════════════════════════════════════════════════════════════════════

mod greed_rarity_boost {
    use std::collections::HashMap;

    use bevy::prelude::*;

    use super::{
        super::generate_chip_offerings, make_mixed_registry, make_registry, test_app_with_registry,
    };
    use crate::{
        chips::{ChipCatalog, definition::Rarity, inventory::ChipInventory},
        mutators::protocols::greed::{GreedConfig, GreedStacks, apply_greed_boost},
        prelude::*,
        shared::rng::ChipRng,
        state::run::chip_select::{ChipOffers, ChipSelectConfig},
    };

    fn default_rarity_weights() -> HashMap<Rarity, f32> {
        HashMap::from([
            (Rarity::Common, 100.0),
            (Rarity::Uncommon, 50.0),
            (Rarity::Rare, 15.0),
        ])
    }

    fn canonical_config() -> GreedConfig {
        GreedConfig {
            rarity_boost_per_skip: 5.0,
        }
    }

    /// Builds the identical baseline app that `test_app_with_registry` builds,
    /// so "no-Greed" offerings can be compared against Greed-app offerings.
    fn baseline_names(registry: ChipCatalog) -> Vec<String> {
        let mut app = test_app_with_registry(registry);
        app.update();
        app.world()
            .resource::<ChipOffers>()
            .0
            .iter()
            .map(|o| o.name().to_owned())
            .collect()
    }

    /// Builds a test app with Greed resources installed (plus the baseline
    /// `ChipCatalog`, `ChipInventory`, `ChipSelectConfig`, `GameRng`, `ChipRng`).
    fn test_app_with_greed(
        registry: ChipCatalog,
        stacks: GreedStacks,
        config: Option<GreedConfig>,
    ) -> App {
        let mut builder = TestAppBuilder::new()
            .insert_resource(registry)
            .with_resource::<ChipInventory>()
            .insert_resource(ChipSelectConfig::default())
            .insert_resource(GameRng::from_seed(42))
            .insert_resource(ChipRng::from_seed(42))
            .insert_resource(stacks)
            .with_system(Update, generate_chip_offerings);
        if let Some(cfg) = config {
            builder = builder.insert_resource(cfg);
        }
        builder.build()
    }

    // ── Behavior 19 — zero GreedStacks leaves rarity weights unchanged ──────

    #[test]
    fn zero_greed_stacks_produces_same_offers_as_no_greed_baseline() {
        let baseline = baseline_names(make_registry(5));
        let mut app = test_app_with_greed(
            make_registry(5),
            GreedStacks { skips: 0 },
            Some(canonical_config()),
        );
        app.update();
        let names: Vec<String> = app
            .world()
            .resource::<ChipOffers>()
            .0
            .iter()
            .map(|o| o.name().to_owned())
            .collect();
        assert_eq!(
            names, baseline,
            "zero skips with Greed resources installed must produce identical \
             offers to the no-Greed baseline (same seed 42)"
        );
    }

    #[test]
    fn greed_stacks_present_but_config_missing_matches_baseline() {
        // Edge case: `GreedStacks` present but `GreedConfig` absent → no boost.
        let baseline = baseline_names(make_registry(5));
        let mut app = test_app_with_greed(make_registry(5), GreedStacks { skips: 5 }, None);
        app.update();
        let names: Vec<String> = app
            .world()
            .resource::<ChipOffers>()
            .0
            .iter()
            .map(|o| o.name().to_owned())
            .collect();
        assert_eq!(
            names, baseline,
            "stacks present but config absent must not apply boost; expected \
             baseline offers"
        );
    }

    // ── Behavior 20 — one skip × 5.0 shifts 5 from Common → 2.5 each ────────

    #[test]
    fn apply_greed_boost_with_one_skip_five_per_skip_shifts_weights_correctly() {
        let mut weights = default_rarity_weights();
        let stacks = GreedStacks { skips: 1 };
        let config = canonical_config();

        apply_greed_boost(&mut weights, stacks, config);

        assert!(
            (weights[&Rarity::Common] - 95.0).abs() < f32::EPSILON,
            "Common expected 95.0 (100 - 5), got {}",
            weights[&Rarity::Common]
        );
        assert!(
            (weights[&Rarity::Uncommon] - 52.5).abs() < f32::EPSILON,
            "Uncommon expected 52.5 (50 + 2.5), got {}",
            weights[&Rarity::Uncommon]
        );
        assert!(
            (weights[&Rarity::Rare] - 17.5).abs() < f32::EPSILON,
            "Rare expected 17.5 (15 + 2.5), got {}",
            weights[&Rarity::Rare]
        );
    }

    #[test]
    fn apply_greed_boost_with_zero_skips_is_noop() {
        let mut weights = default_rarity_weights();
        let stacks = GreedStacks { skips: 0 };
        let config = canonical_config();

        apply_greed_boost(&mut weights, stacks, config);

        assert!((weights[&Rarity::Common] - 100.0).abs() < f32::EPSILON);
        assert!((weights[&Rarity::Uncommon] - 50.0).abs() < f32::EPSILON);
        assert!((weights[&Rarity::Rare] - 15.0).abs() < f32::EPSILON);
    }

    // ── Behavior 21 — sum is preserved while not clamped ────────────────────

    #[test]
    fn apply_greed_boost_preserves_weight_sum_for_one_skip() {
        let mut weights = default_rarity_weights();
        let stacks = GreedStacks { skips: 1 };
        let config = canonical_config();

        apply_greed_boost(&mut weights, stacks, config);

        let sum = weights[&Rarity::Common] + weights[&Rarity::Uncommon] + weights[&Rarity::Rare];
        assert!(
            (sum - 165.0).abs() < 1e-4,
            "sum of weights must be preserved at 165.0, got {sum}"
        );
    }

    #[test]
    fn apply_greed_boost_preserves_weight_sum_for_five_skips() {
        let mut weights = default_rarity_weights();
        let stacks = GreedStacks { skips: 5 };
        let config = canonical_config();

        apply_greed_boost(&mut weights, stacks, config);

        let sum = weights[&Rarity::Common] + weights[&Rarity::Uncommon] + weights[&Rarity::Rare];
        // boost = 25.0, Common = 75.0 (above floor 10.0), sum preserved.
        assert!(
            (sum - 165.0).abs() < 1e-4,
            "5-skip sum must still equal 165.0, got {sum}"
        );
    }

    // ── Behavior 22 — floor clamp: Common never drops below original × 0.10 ──

    #[test]
    fn apply_greed_boost_clamps_common_to_floor_at_hundred_skips() {
        let mut weights = default_rarity_weights();
        let stacks = GreedStacks { skips: 100 };
        let config = canonical_config();

        apply_greed_boost(&mut weights, stacks, config);

        assert!(
            (weights[&Rarity::Common] - 10.0).abs() < f32::EPSILON,
            "Common must be clamped to floor 10.0 (100 × 0.10), got {}",
            weights[&Rarity::Common]
        );
        // removed = 100 - 10 = 90, each of Uncommon/Rare gains 45.
        assert!(
            (weights[&Rarity::Uncommon] - 95.0).abs() < f32::EPSILON,
            "Uncommon expected 95.0 (50 + 45), got {}",
            weights[&Rarity::Uncommon]
        );
        assert!(
            (weights[&Rarity::Rare] - 60.0).abs() < f32::EPSILON,
            "Rare expected 60.0 (15 + 45), got {}",
            weights[&Rarity::Rare]
        );
    }

    #[test]
    fn apply_greed_boost_exactly_at_floor_no_truncation() {
        // skips = 18, per-skip 5.0 → boost 90.0 → Common = 10.0 exactly.
        let mut weights = default_rarity_weights();
        let stacks = GreedStacks { skips: 18 };
        let config = canonical_config();

        apply_greed_boost(&mut weights, stacks, config);

        assert!(
            (weights[&Rarity::Common] - 10.0).abs() < f32::EPSILON,
            "Common expected exactly 10.0 at boost 90, got {}",
            weights[&Rarity::Common]
        );
        assert!(
            (weights[&Rarity::Uncommon] - 95.0).abs() < f32::EPSILON,
            "Uncommon expected 95.0, got {}",
            weights[&Rarity::Uncommon]
        );
        assert!(
            (weights[&Rarity::Rare] - 60.0).abs() < f32::EPSILON,
            "Rare expected 60.0, got {}",
            weights[&Rarity::Rare]
        );
    }

    #[test]
    fn apply_greed_boost_beyond_floor_caps_removed_not_boost() {
        // skips = 91, per-skip 1.0 → desired boost 91.0, but floor clamps
        // `removed` to 90.0.
        let mut weights = default_rarity_weights();
        let stacks = GreedStacks { skips: 91 };
        let config = GreedConfig {
            rarity_boost_per_skip: 1.0,
        };

        apply_greed_boost(&mut weights, stacks, config);

        assert!(
            (weights[&Rarity::Common] - 10.0).abs() < f32::EPSILON,
            "Common must be pinned at 10.0, got {}",
            weights[&Rarity::Common]
        );
        // removed is 90.0 (not 91.0), so +45.0 each not +45.5.
        assert!(
            (weights[&Rarity::Uncommon] - 95.0).abs() < f32::EPSILON,
            "Uncommon expected 95.0 (removed is 90.0, not 91.0); got {}",
            weights[&Rarity::Uncommon]
        );
        assert!(
            (weights[&Rarity::Rare] - 60.0).abs() < f32::EPSILON,
            "Rare expected 60.0 (removed is 90.0, not 91.0); got {}",
            weights[&Rarity::Rare]
        );
    }

    // ── Behavior 23 — floor stability across additional skips once pinned ──

    #[test]
    fn apply_greed_boost_stable_once_floor_pinned() {
        let mut w20 = default_rarity_weights();
        apply_greed_boost(&mut w20, GreedStacks { skips: 20 }, canonical_config());

        let mut w30 = default_rarity_weights();
        apply_greed_boost(&mut w30, GreedStacks { skips: 30 }, canonical_config());

        assert!(
            (w20[&Rarity::Common] - w30[&Rarity::Common]).abs() < f32::EPSILON,
            "once floor is hit, Common must stay the same across additional skips"
        );
        assert!(
            (w20[&Rarity::Uncommon] - w30[&Rarity::Uncommon]).abs() < f32::EPSILON,
            "once floor is hit, Uncommon must stay the same across additional skips"
        );
        assert!(
            (w20[&Rarity::Rare] - w30[&Rarity::Rare]).abs() < f32::EPSILON,
            "once floor is hit, Rare must stay the same across additional skips"
        );
        // And the concrete values.
        assert!((w20[&Rarity::Common] - 10.0).abs() < f32::EPSILON);
        assert!((w20[&Rarity::Uncommon] - 95.0).abs() < f32::EPSILON);
        assert!((w20[&Rarity::Rare] - 60.0).abs() < f32::EPSILON);
    }

    // ── Behavior 24 — end-to-end: offers differ from baseline under boost ──

    #[test]
    fn greed_changes_offer_sequence_end_to_end() {
        let baseline = baseline_names(make_mixed_registry());
        let mut app = test_app_with_greed(
            make_mixed_registry(),
            GreedStacks { skips: 10 },
            Some(canonical_config()),
        );
        app.update();
        let names: Vec<String> = app
            .world()
            .resource::<ChipOffers>()
            .0
            .iter()
            .map(|o| o.name().to_owned())
            .collect();
        assert_ne!(
            names, baseline,
            "10 skips × 5.0/skip must shift the rarity distribution enough to \
             change the seeded offer sequence; baseline and Greed output were \
             identical"
        );
    }

    // ── Behavior 25 — generate_chip_offerings without Greed must not panic ──

    #[test]
    fn generate_chip_offerings_does_not_panic_when_greed_resources_absent() {
        let mut app = test_app_with_registry(make_registry(5));
        app.update();

        let offers = app.world().resource::<ChipOffers>();
        assert!(
            !offers.0.is_empty(),
            "offers must still be generated when no Greed resources present"
        );
    }

    // ── Behavior 26 — GreedStacks present, GreedConfig absent → baseline ──

    #[test]
    fn generate_chip_offerings_with_stacks_but_no_config_matches_baseline() {
        let baseline = baseline_names(make_registry(5));
        let mut app = test_app_with_greed(make_registry(5), GreedStacks { skips: 10 }, None);
        app.update();
        let names: Vec<String> = app
            .world()
            .resource::<ChipOffers>()
            .0
            .iter()
            .map(|o| o.name().to_owned())
            .collect();
        assert_eq!(
            names, baseline,
            "partial activation (stacks without config) must not apply a boost"
        );
    }
}
