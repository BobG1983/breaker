//! Group E — `despawn_expired_gravity_wells` system.
//!
//! Every test wires only `despawn_expired_gravity_wells` via
//! `wire_despawn_only`. The system despawns entities where
//! `well.remaining <= 0.0` — strict inequality.

use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::{
    super::system::GravityWell,
    helpers::{spawn_well, test_app_playing, tick_with_dt, wire_despawn_only},
};

// ── Behavior 36 — well with remaining=0.0 is despawned — PRESERVED ─────

#[test]
fn expired_well_is_despawned() {
    let mut app = test_app_playing();
    wire_despawn_only(&mut app);
    let well = spawn_well(&mut app, Vec2::ZERO, 500.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(
        app.world().get_entity(well).is_err(),
        "well with remaining=0 should be despawned"
    );
}

#[test]
fn expired_well_despawn_is_position_independent() {
    // Edge: position doesn't affect despawn.
    let mut app = test_app_playing();
    wire_despawn_only(&mut app);
    let well = spawn_well(&mut app, Vec2::new(42.0, 42.0), 500.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(app.world().get_entity(well).is_err());
}

// ── Behavior 37 — well with remaining < 0 is despawned ───────────────

#[test]
fn well_with_negative_remaining_is_despawned() {
    let mut app = test_app_playing();
    wire_despawn_only(&mut app);
    let well = spawn_well(&mut app, Vec2::ZERO, 500.0, -0.5);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(app.world().get_entity(well).is_err());
}

#[test]
fn well_with_neg_infinity_remaining_is_despawned() {
    // Edge: pathological NEG_INFINITY — still despawned (<= 0.0 predicate).
    let mut app = test_app_playing();
    wire_despawn_only(&mut app);
    let well = spawn_well(&mut app, Vec2::ZERO, 500.0, f32::NEG_INFINITY);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(app.world().get_entity(well).is_err());
}

// ── Behavior 38 — well with remaining=0.0001 NOT despawned (strict) ──

#[test]
fn well_with_tiny_positive_remaining_survives() {
    let mut app = test_app_playing();
    wire_despawn_only(&mut app);
    let well = spawn_well(&mut app, Vec2::ZERO, 500.0, 0.0001);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(app.world().get_entity(well).is_ok());
    // Despawn does not tick `remaining` — unchanged.
    let remaining = app.world().get::<GravityWell>(well).unwrap().remaining;
    assert!((remaining - 0.0001).abs() < f32::EPSILON);
}

#[test]
fn well_with_epsilon_remaining_survives() {
    // Edge: f32::EPSILON is still positive → survives.
    let mut app = test_app_playing();
    wire_despawn_only(&mut app);
    let well = spawn_well(&mut app, Vec2::ZERO, 500.0, f32::EPSILON);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(app.world().get_entity(well).is_ok());
}

// ── Behavior 39 — active well (remaining=1.5) is NOT despawned — PRESERVED

#[test]
fn active_well_is_not_despawned() {
    let mut app = test_app_playing();
    wire_despawn_only(&mut app);
    let well = spawn_well(&mut app, Vec2::ZERO, 500.0, 1.5);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(app.world().get_entity(well).is_ok());
}

#[test]
fn well_with_f32_max_remaining_survives() {
    // Edge: upper bound — still alive.
    let mut app = test_app_playing();
    wire_despawn_only(&mut app);
    let well = spawn_well(&mut app, Vec2::ZERO, 500.0, f32::MAX);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(app.world().get_entity(well).is_ok());
}

// ── Behavior 40 — multiple expired wells all despawned in one tick ───

#[test]
fn multiple_expired_wells_despawned_active_ones_survive() {
    let mut app = test_app_playing();
    wire_despawn_only(&mut app);
    let w0 = spawn_well(&mut app, Vec2::ZERO, 500.0, 0.0);
    let w1 = spawn_well(&mut app, Vec2::ZERO, 500.0, -1.0);
    let w2 = spawn_well(&mut app, Vec2::ZERO, 500.0, 0.01);
    let w3 = spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let w4 = spawn_well(&mut app, Vec2::ZERO, 500.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(app.world().get_entity(w0).is_err());
    assert!(app.world().get_entity(w1).is_err());
    assert!(app.world().get_entity(w2).is_ok());
    assert!(app.world().get_entity(w3).is_ok());
    assert!(app.world().get_entity(w4).is_err());

    let mut query = app.world_mut().query::<&GravityWell>();
    assert_eq!(query.iter(app.world()).count(), 2);
}

#[test]
fn survivor_wells_stay_alive_across_second_tick() {
    // Edge: second tick does not despawn survivors.
    let mut app = test_app_playing();
    wire_despawn_only(&mut app);
    let w2 = spawn_well(&mut app, Vec2::ZERO, 500.0, 0.01);
    let w3 = spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(app.world().get_entity(w2).is_ok());
    assert!(app.world().get_entity(w3).is_ok());
}

// ── Behavior 41 — no wells in world → no panic ──────────────────────

#[test]
fn no_wells_in_world_no_panic() {
    let mut app = test_app_playing();
    wire_despawn_only(&mut app);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&GravityWell>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

#[test]
fn non_well_entity_not_affected_by_despawn() {
    // Edge: non-Well entity (just Position2D) is excluded by the query.
    let mut app = test_app_playing();
    wire_despawn_only(&mut app);
    let non_well = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(app.world().get_entity(non_well).is_ok());
}
