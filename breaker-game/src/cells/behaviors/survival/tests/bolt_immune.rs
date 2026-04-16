//! Part F — `suppress_bolt_immune_damage` system tests (behaviors 39-46).

use bevy::prelude::*;

use super::helpers::*;
use crate::prelude::*;

// ── Behavior 39: BoltImmune cell: bolt damage is suppressed ──

#[test]
fn bolt_immune_cell_bolt_damage_is_suppressed() {
    let mut app = build_bolt_immune_test_app();

    let cell = spawn_bolt_immune_cell(&mut app, 20.0);
    let bolt = spawn_test_bolt(&mut app);

    advance_to_playing(&mut app);

    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_Y, 0));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "BoltImmune cell should suppress bolt damage, got hp.current == {}",
        hp.current
    );
}

// Behavior 39 edge: f32::MAX damage is still suppressed
#[test]
fn bolt_immune_cell_suppresses_max_damage() {
    let mut app = build_bolt_immune_test_app();

    let cell = spawn_bolt_immune_cell(&mut app, 20.0);
    let bolt = spawn_test_bolt(&mut app);

    advance_to_playing(&mut app);

    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_Y, 0));
    push_damage(&mut app, damage_msg_from(cell, f32::MAX, bolt));

    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "f32::MAX damage should still be suppressed, got hp.current == {}",
        hp.current
    );
}

// ── Behavior 40: Non-BoltImmune cell: bolt damage passes through normally ──

#[test]
fn non_bolt_immune_cell_bolt_damage_passes_through() {
    let mut app = build_bolt_immune_test_app();

    let cell = spawn_plain_cell(&mut app, 20.0);
    let bolt = spawn_test_bolt(&mut app);

    advance_to_playing(&mut app);

    push_bolt_impact(&mut app, bolt_impact(bolt, cell, Vec2::NEG_Y, 0));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt));

    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "non-BoltImmune cell should take damage normally, got hp.current == {}",
        hp.current
    );
}

// ── Behavior 41: BoltImmune cell: non-bolt damage (dealerless) passes through ──

#[test]
fn bolt_immune_cell_dealerless_damage_passes_through() {
    let mut app = build_bolt_immune_test_app();

    let cell = spawn_bolt_immune_cell(&mut app, 20.0);

    advance_to_playing(&mut app);

    // No BoltImpactCell message for this cell, just dealerless damage
    push_damage(&mut app, damage_msg_dealerless(cell, 5.0));

    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "dealerless damage should pass through BoltImmune cell, got hp.current == {}",
        hp.current
    );
}

// Behavior 41 edge: dealer is some non-bolt entity with no BoltImpactCell
#[test]
fn bolt_immune_cell_non_bolt_dealer_damage_passes_through() {
    let mut app = build_bolt_immune_test_app();

    let cell = spawn_bolt_immune_cell(&mut app, 20.0);
    let non_bolt = app.world_mut().spawn_empty().id();

    advance_to_playing(&mut app);

    // No BoltImpactCell references non_bolt as bolt
    push_damage(&mut app, damage_msg_from(cell, 5.0, non_bolt));

    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "non-bolt dealer damage should pass through, got hp.current == {}",
        hp.current
    );
}

// ── Behavior 42: BoltImmune cell: damage from non-bolt dealer with no BoltImpactCell passes through ──

#[test]
fn bolt_immune_cell_damage_from_entity_without_impact_passes_through() {
    let mut app = build_bolt_immune_test_app();

    let cell = spawn_bolt_immune_cell(&mut app, 20.0);
    let some_entity = app.world_mut().spawn_empty().id();

    advance_to_playing(&mut app);

    // Dealer is some_entity, but no BoltImpactCell references it as bolt
    push_damage(&mut app, damage_msg_from(cell, 5.0, some_entity));

    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "damage from entity with no bolt impact should pass through, got hp.current == {}",
        hp.current
    );
}

// ── Behavior 43: Mixed: BoltImmune cell + non-immune cell both hit by same bolt ──

#[test]
fn mixed_bolt_immune_and_non_immune_cells_hit_by_same_bolt() {
    let mut app = build_bolt_immune_test_app();

    let cell_a = spawn_bolt_immune_cell(&mut app, 20.0);
    let cell_b = spawn_plain_cell(&mut app, 20.0);
    let bolt = spawn_test_bolt(&mut app);

    advance_to_playing(&mut app);

    push_bolt_impact(&mut app, bolt_impact(bolt, cell_a, Vec2::NEG_Y, 0));
    push_bolt_impact(&mut app, bolt_impact(bolt, cell_b, Vec2::NEG_Y, 0));
    push_damage(&mut app, damage_msg_from(cell_a, 5.0, bolt));
    push_damage(&mut app, damage_msg_from(cell_b, 5.0, bolt));

    tick(&mut app);

    let hp_a = app
        .world()
        .get::<Hp>(cell_a)
        .expect("cell A should have Hp");
    assert!(
        (hp_a.current - 20.0).abs() < f32::EPSILON,
        "BoltImmune cell A should suppress damage, got hp.current == {}",
        hp_a.current
    );
    let hp_b = app
        .world()
        .get::<Hp>(cell_b)
        .expect("cell B should have Hp");
    assert!(
        (hp_b.current - 15.0).abs() < f32::EPSILON,
        "non-immune cell B should take damage, got hp.current == {}",
        hp_b.current
    );
}

// ── Behavior 44: BoltImmune cell: multiple bolt hits in same tick, all suppressed ──

#[test]
fn bolt_immune_cell_multiple_bolt_hits_all_suppressed() {
    let mut app = build_bolt_immune_test_app();

    let cell = spawn_bolt_immune_cell(&mut app, 20.0);
    let bolt1 = spawn_test_bolt(&mut app);
    let bolt2 = spawn_test_bolt(&mut app);

    advance_to_playing(&mut app);

    push_bolt_impact(&mut app, bolt_impact(bolt1, cell, Vec2::NEG_Y, 0));
    push_bolt_impact(&mut app, bolt_impact(bolt2, cell, Vec2::NEG_Y, 0));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt1));
    push_damage(&mut app, damage_msg_from(cell, 5.0, bolt2));

    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "all bolt hits should be suppressed, got hp.current == {}",
        hp.current
    );
}

// ── Behavior 45: No BoltImpactCell messages: all damage passes through ──

#[test]
fn bolt_immune_cell_no_bolt_impacts_all_damage_passes_through() {
    let mut app = build_bolt_immune_test_app();

    let cell = spawn_bolt_immune_cell(&mut app, 20.0);
    let some_entity = app.world_mut().spawn_empty().id();

    advance_to_playing(&mut app);

    // Damage with a dealer but no BoltImpactCell messages at all
    push_damage(&mut app, damage_msg_from(cell, 5.0, some_entity));

    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "without BoltImpactCell messages, all damage should pass through, got hp.current == {}",
        hp.current
    );
}

// ── Behavior 46: Empty queues: system is a no-op ──

#[test]
fn suppress_bolt_immune_damage_empty_queues_no_crash() {
    let mut app = build_bolt_immune_test_app();

    // Spawn an immune cell but don't enqueue any messages
    let _cell = spawn_bolt_immune_cell(&mut app, 20.0);

    advance_to_playing(&mut app);
    tick(&mut app);

    // No crash, no side effects
}
