//! Singular target dispatch tests — behaviors 5-7 (Bolt, `ActiveCells`, `ActiveWalls`).
//!
//! In the new system, non-Breaker targets stamp trees directly to the Breaker's
//! `BoundEffects` for deferred dispatch.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    chips::{definition::ChipDefinition, systems::dispatch_chip_effects::tests::helpers::*},
    effect_v3::{
        effects::{DamageBoostConfig, ShieldConfig, SpeedBoostConfig},
        storage::BoundEffects,
        types::{EffectType, EntityKind, RootNode, StampTarget, Tree, Trigger},
    },
};

// ── Behavior 5: Bolt target stamps to Breaker ──

#[test]
fn bolt_target_stamps_to_breaker() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Bolt Speed".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootNode::Stamp(
            StampTarget::Bolt,
            Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.1),
            })),
        )],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Bolt Speed");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 stamped entry for Bolt target"
    );
}

// ── Behavior 5 edge case: Bolt When stamps full tree ──

#[test]
fn bolt_when_stamps_full_tree_to_breaker() {
    let mut app = test_app();

    let def = ChipDefinition::test(
        "Bolt Chain",
        Tree::When(
            Trigger::Bumped,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.2),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Bolt Chain");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(bound.0.len(), 1);
    assert!(
        matches!(&bound.0[0].1, Tree::When(Trigger::Bumped, _)),
        "Tree should be When(Bumped, ...), got {:?}",
        bound.0[0].1
    );
}

// ── Behavior 6: ActiveCells target stamps to Breaker ──

#[test]
fn active_cells_stamps_to_breaker() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Cell Shield",
        StampTarget::ActiveCells,
        Tree::When(
            Trigger::Impacted(EntityKind::Bolt),
            Box::new(Tree::Fire(EffectType::Shield(ShieldConfig {
                duration: OrderedFloat(3.0),
                reflection_cost: OrderedFloat(0.0),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Cell Shield");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 stamped entry for ActiveCells"
    );
}

// ── Behavior 7: ActiveWalls target stamps to Breaker ──

#[test]
fn active_walls_stamps_to_breaker() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Wall Damage",
        StampTarget::ActiveWalls,
        Tree::When(
            Trigger::Impacted(EntityKind::Bolt),
            Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(1.5),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Wall Damage");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 stamped entry for ActiveWalls"
    );
}

// ── Zero breaker edge cases ──

#[test]
fn bolt_target_zero_breakers_no_panic() {
    let mut app = test_app();

    let def = ChipDefinition::test(
        "Bolt Empty",
        Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.1),
        })),
        5,
    );
    insert_chip(&mut app, def);

    select_chip(&mut app, "Bolt Empty");
    app.update();
}

#[test]
fn active_cells_target_zero_breakers_no_panic() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Cell Empty",
        StampTarget::ActiveCells,
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(1.0),
        })),
        5,
    );
    insert_chip(&mut app, def);

    select_chip(&mut app, "Cell Empty");
    app.update();
}
