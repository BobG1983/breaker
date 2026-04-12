//! `ActiveCells` target dispatch tests — behaviors 1-2.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    chips::{definition::ChipDefinition, systems::dispatch_chip_effects::tests::helpers::*},
    effect_v3::{
        effects::{DamageBoostConfig, ShieldConfig, SpeedBoostConfig},
        stacking::EffectStack,
        storage::BoundEffects,
        types::{EffectType, EntityKind, RootNode, StampTarget, Tree, Trigger},
    },
};

// ── Behavior 1: ActiveCells stamps tree directly to Breaker's BoundEffects ──

#[test]
fn active_cells_target_stamps_tree_to_breaker() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Cell Fortify".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootNode::Stamp(
            StampTarget::ActiveCells,
            Tree::When(
                Trigger::Impacted(EntityKind::Bolt),
                Box::new(Tree::Fire(EffectType::Shield(ShieldConfig {
                    duration: OrderedFloat(5.0),
                    reflection_cost: OrderedFloat(0.0),
                }))),
            ),
        )],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Cell Fortify");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 stamped entry for ActiveCells"
    );

    let (chip_name, tree) = &bound.0[0];
    assert_eq!(chip_name, "Cell Fortify");
    assert!(
        matches!(tree, Tree::When(Trigger::Impacted(EntityKind::Bolt), _)),
        "Tree should be stamped directly, got {tree:?}"
    );
}

// ── Behavior 1 edge case: Zero Breaker entities ──

#[test]
fn active_cells_target_with_zero_breakers_no_panic() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Cell Shield",
        StampTarget::ActiveCells,
        Tree::When(
            Trigger::Impacted(EntityKind::Bolt),
            Box::new(Tree::Fire(EffectType::Shield(ShieldConfig {
                duration: OrderedFloat(5.0),
                reflection_cost: OrderedFloat(0.0),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

    // No breaker spawned
    select_chip(&mut app, "Cell Shield");

    // Should not panic
    app.update();
}

// ── Behavior 1 edge case: ActiveCells with bare Fire stamps to breaker ──

#[test]
fn active_cells_with_fire_stamps_to_breaker() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Cell Damage".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootNode::Stamp(
            StampTarget::ActiveCells,
            Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(1.5),
            })),
        )],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Cell Damage");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 stamped entry for ActiveCells Fire"
    );
}

// ── Behavior 2: Multiple stamps to same target all stamped ──

#[test]
fn multiple_active_cells_stamps_all_stamped() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Dual Cell".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![
            RootNode::Stamp(
                StampTarget::ActiveCells,
                Tree::When(
                    Trigger::Impacted(EntityKind::Bolt),
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.2),
                    }))),
                ),
            ),
            RootNode::Stamp(
                StampTarget::ActiveCells,
                Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                    multiplier: OrderedFloat(1.1),
                })),
            ),
        ],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Dual Cell");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "Breaker should have 2 stamped entries for ActiveCells"
    );
}

// ── Behavior 2: Breaker direct Fire still fires immediately ──

#[test]
fn breaker_fire_still_fires_immediately_alongside_cells() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Mixed Target".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![
            RootNode::Stamp(
                StampTarget::Breaker,
                Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                })),
            ),
            RootNode::Stamp(
                StampTarget::ActiveCells,
                Tree::When(
                    Trigger::Impacted(EntityKind::Bolt),
                    Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                        multiplier: OrderedFloat(1.2),
                    }))),
                ),
            ),
        ],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Mixed Target");

    app.update();

    // Breaker Fire fired immediately
    let speed = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(
        speed.len(),
        1,
        "SpeedBoost should fire immediately on breaker"
    );

    // ActiveCells stamped
    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Only the ActiveCells tree should be in BoundEffects"
    );
}
