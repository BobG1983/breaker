//! Non-Breaker target dispatch tests — behaviors 7-12.
//!
//! Non-Breaker targets (`ActiveBolts`, Bolt, `ActiveCells`, etc.) are
//! stamped directly to the Breaker's `BoundEffects` for deferred dispatch.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    chips::{definition::ChipDefinition, systems::dispatch_chip_effects::tests::helpers::*},
    effect_v3::{
        effects::{DamageBoostConfig, ShieldConfig, ShockwaveConfig, SpeedBoostConfig},
        storage::BoundEffects,
        types::{EffectType, EntityKind, StampTarget, Tree, Trigger},
    },
};

// ── Behavior 7: Target `ActiveBolts` stamps to Breaker's BoundEffects ──

#[test]
fn active_bolts_target_stamps_to_breaker_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Parry Shockwave",
        StampTarget::ActiveBolts,
        Tree::When(
            Trigger::PerfectBumped,
            Box::new(Tree::Fire(EffectType::Shockwave(ShockwaveConfig {
                base_range:      OrderedFloat(64.0),
                range_per_level: OrderedFloat(0.0),
                stacks:          1,
                speed:           OrderedFloat(500.0),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Parry Shockwave");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 BoundEffects entry for ActiveBolts stamp"
    );
    assert_eq!(bound.0[0].0, "Parry Shockwave");
    assert!(
        matches!(&bound.0[0].1, Tree::When(Trigger::PerfectBumped, _)),
        "ActiveBolts should stamp tree directly to breaker"
    );
}

// ── Behavior 7 edge case: Zero Breaker entities — no panic ──

#[test]
fn active_bolts_target_with_zero_breakers_no_panic() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Empty Target",
        StampTarget::ActiveBolts,
        Tree::When(
            Trigger::Bumped,
            Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(1.0),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

    // No breaker entities spawned — stamping has nowhere to push
    select_chip(&mut app, "Empty Target");

    // Should not panic
    app.update();
}

// ── Behavior 8: Target `Bolt` stamps to Breaker ──

#[test]
fn bolt_target_stamps_to_breaker_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test(
        "Slight Bolt Speed",
        Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.1),
        })),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Slight Bolt Speed");

    app.update();

    // Bolt target stamps to breaker for deferred dispatch
    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 BoundEffects entry for Bolt target stamp"
    );
}

// ── Behavior 9: Target `ActiveCells` stamps to Breaker's BoundEffects ──

#[test]
fn active_cells_target_stamps_to_breaker_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Cell Shield",
        StampTarget::ActiveCells,
        Tree::When(
            Trigger::Impacted(EntityKind::Bolt),
            Box::new(Tree::Fire(EffectType::Shield(ShieldConfig {
                duration:        OrderedFloat(5.0),
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
        "Breaker should have 1 BoundEffects entry for ActiveCells stamp"
    );
}

// ── Behavior 10: Zero Breaker entities for Cell-like target — no panic ──

#[test]
fn cell_target_with_zero_breakers_no_panic() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Empty Cell",
        StampTarget::ActiveCells,
        Tree::When(
            Trigger::Bumped,
            Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(1.0),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

    // No breaker entities spawned
    select_chip(&mut app, "Empty Cell");
    app.update();
}

// ── Behavior 11: Target `ActiveWalls` stamps to Breaker's BoundEffects ──

#[test]
fn active_walls_target_stamps_to_breaker_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Wall Effect",
        StampTarget::ActiveWalls,
        Tree::When(
            Trigger::Impacted(EntityKind::Bolt),
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Wall Effect");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 BoundEffects entry for ActiveWalls stamp"
    );
}

// ── Behavior 12: Zero Breaker entities for Wall target — no panic ──

#[test]
fn wall_target_with_zero_breakers_no_panic() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Empty Wall",
        StampTarget::ActiveWalls,
        Tree::When(
            Trigger::Bumped,
            Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(1.0),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

    // No breaker entities spawned
    select_chip(&mut app, "Empty Wall");
    app.update();
}
