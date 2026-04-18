//! Group F — `register` integration: full chain, ordering, gating.
//!
//! These tests exercise the production wiring (`register(&mut app)`) so
//! the run conditions `hazard_active(GravitySurge)` and
//! `in_state(NodeState::Playing)` are active and the chain ordering
//! `(spawn_gravity_wells, gravity_well_pull, despawn_expired_gravity_wells)
//! .chain()` is enforced.

use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::{
    super::system::{GravityWell, register},
    helpers::{
        add_gravity_surge_stacks, canonical_config, install_gravity_surge_config, spawn_bolt,
        spawn_well, test_app_not_playing, test_app_playing, tick_with_dt, write_cell_destroyed,
    },
};
use crate::hazard::{definition::HazardKind, resources::ActiveHazards};

// ── Behavior 42 — full chain in one tick ─────────────────────────────

#[test]
fn full_chain_spawns_applies_force_and_preserves_active_well() {
    let mut app = test_app_playing();
    register(&mut app);
    install_gravity_surge_config(&mut app, canonical_config());
    add_gravity_surge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::ZERO);
    write_cell_destroyed(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    // Exactly 1 well: spawned this tick, pulled (ticked), survived despawn.
    let mut query = app.world_mut().query::<(&GravityWell, &Position2D)>();
    let wells: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(wells.len(), 1);
    let (well, pos) = wells[0];
    assert!(pos.0.length() < f32::EPSILON);
    assert!((well.remaining - (2.0 - 0.1)).abs() < 1e-5);

    // Pull: 500/100 = 5 u/s² × 0.1 = 0.5 in -X.
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!((vel.0.x - (-0.5)).abs() < 1e-3);
    assert!(vel.0.y.abs() < 1e-5);
}

#[test]
fn full_chain_second_tick_no_new_message_accumulates_velocity() {
    // Edge: tick 2 with no new Destroyed<Cell> → still 1 well (remaining
    // ≈ 1.8), bolt velocity accumulates another -0.5.
    let mut app = test_app_playing();
    register(&mut app);
    install_gravity_surge_config(&mut app, canonical_config());
    add_gravity_surge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::ZERO);
    write_cell_destroyed(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let mut query = app.world_mut().query::<&GravityWell>();
    let wells: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(wells.len(), 1);
    assert!((wells[0].remaining - 1.8).abs() < 1e-5);

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!((vel.0.x - (-1.0)).abs() < 1e-3);
}

// ── Behavior 43 — same-tick: new cell arrives while old well expires ──

#[test]
fn same_tick_new_spawn_and_old_despawn_coexist_in_chain() {
    let mut app = test_app_playing();
    register(&mut app);
    install_gravity_surge_config(&mut app, canonical_config());
    add_gravity_surge_stacks(&mut app, 1);
    // Pre-insert an already-expired well.
    let old = spawn_well(&mut app, Vec2::new(50.0, 0.0), 500.0, 0.0);
    // Fresh cell destroyed at a different position.
    write_cell_destroyed(&mut app, Vec2::new(-50.0, 0.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    // Old well despawned.
    assert!(app.world().get_entity(old).is_err());

    // Exactly 1 well — the freshly spawned one.
    let mut query = app.world_mut().query::<(&GravityWell, &Position2D)>();
    let wells: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(wells.len(), 1);
    let (well, pos) = wells[0];
    assert!((pos.0 - Vec2::new(-50.0, 0.0)).length() < f32::EPSILON);
    // Spawn → pull (ticks remaining) → despawn. New well has remaining=2.0,
    // ticked down by 0.016 to ≈ 1.984.
    assert!((well.remaining - (2.0 - 0.016)).abs() < 1e-4);
    assert!((well.strength - 500.0).abs() < 1e-4);
}

#[test]
fn same_tick_chain_completes_without_bolt() {
    // Edge: no bolt in the world — chain still runs without panic.
    let mut app = test_app_playing();
    register(&mut app);
    install_gravity_surge_config(&mut app, canonical_config());
    add_gravity_surge_stacks(&mut app, 1);
    let old = spawn_well(&mut app, Vec2::new(50.0, 0.0), 500.0, 0.0);
    write_cell_destroyed(&mut app, Vec2::new(-50.0, 0.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(app.world().get_entity(old).is_err());
    let mut query = app.world_mut().query::<&GravityWell>();
    assert_eq!(query.iter(app.world()).count(), 1);
}

// ── Behavior 44 — gate off (stacks=0) → no spawn / pull / despawn ─────

#[test]
fn gate_off_with_zero_stacks_suppresses_all_systems() {
    let mut app = test_app_playing();
    register(&mut app);
    install_gravity_surge_config(&mut app, canonical_config());
    // NO stacks added — gate closed.
    let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::new(5.0, 3.0));
    write_cell_destroyed(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let mut query = app.world_mut().query::<&GravityWell>();
    assert_eq!(query.iter(app.world()).count(), 0);

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(vel.0, Vec2::new(5.0, 3.0));
}

#[test]
fn gate_reopens_when_stack_added_mid_run() {
    // Edge: add a stack mid-way and write fresh message → gate opens,
    // spawn + pull both fire.
    //
    // Note: we deliberately do NOT write a `Destroyed<Cell>` during the
    // gated-off tick. Bevy's MessageReader retains unread messages
    // across ticks — when `spawn_gravity_wells` is gated off, any
    // pre-gate messages accumulate and would be consumed once the gate
    // opens, spawning an extra well. Here we isolate the gate toggle
    // by writing messages ONLY after the stack is added.
    let mut app = test_app_playing();
    register(&mut app);
    install_gravity_surge_config(&mut app, canonical_config());
    let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::new(5.0, 3.0));

    // Tick 1: gate closed — nothing happens, no message written.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    {
        let mut q = app.world_mut().query::<&GravityWell>();
        assert_eq!(q.iter(app.world()).count(), 0);
    }

    // Open the gate: add 1 stack, THEN write a fresh Destroyed<Cell>.
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::GravitySurge);
    write_cell_destroyed(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let mut query = app.world_mut().query::<&GravityWell>();
    assert_eq!(query.iter(app.world()).count(), 1);

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    // Bolt had velocity (5, 3); gravity pulled -0.5 in X → (4.5, 3).
    assert!((vel.0.x - 4.5).abs() < 1e-3);
    assert!((vel.0.y - 3.0).abs() < 1e-5);
}

// ── Behavior 45 — gate off (state not Playing) → no spawn / pull / despawn

#[test]
fn gate_off_outside_playing_state_suppresses_all_systems() {
    let mut app = test_app_not_playing();
    register(&mut app);
    install_gravity_surge_config(&mut app, canonical_config());
    add_gravity_surge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::new(5.0, 3.0));
    write_cell_destroyed(&mut app, Vec2::ZERO);
    // Pre-seed an already-expired well — it should NOT despawn.
    let expired = spawn_well(&mut app, Vec2::new(0.0, 0.0), 500.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    // Exactly 1 well — the expired one — still alive (despawn gated off).
    let mut query = app.world_mut().query::<&GravityWell>();
    assert_eq!(query.iter(app.world()).count(), 1);
    assert!(app.world().get_entity(expired).is_ok());

    // Bolt unchanged — pull gated off.
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(vel.0, Vec2::new(5.0, 3.0));
}

#[test]
fn same_setup_with_playing_state_fires_chain() {
    // Edge: mirror the non-Playing setup with Playing state — chain fires.
    let mut app = test_app_playing();
    register(&mut app);
    install_gravity_surge_config(&mut app, canonical_config());
    add_gravity_surge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::new(5.0, 3.0));
    write_cell_destroyed(&mut app, Vec2::ZERO);
    let expired = spawn_well(&mut app, Vec2::new(0.0, 0.0), 500.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    // The expired well was despawned.
    assert!(app.world().get_entity(expired).is_err());

    // New well spawned at (0, 0) from the Destroyed<Cell>.
    let mut query = app.world_mut().query::<(&GravityWell, &Position2D)>();
    let wells: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(wells.len(), 1);
    assert!(wells[0].1.0.length() < f32::EPSILON);

    // Bolt pulled.
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!((vel.0.x - 4.5).abs() < 1e-3);
}
