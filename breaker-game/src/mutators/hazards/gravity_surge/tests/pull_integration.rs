//! Group I — Integration: `gravity_well_pull` + `apply_bolt_forces` → `Velocity2D`.
//! Group J — Wire ordering: `gravity_well_pull` runs `.before(BoltSystems::ApplyForces)`.
//!
//! These tests require the consumer (`apply_bolt_forces`) to be wired alongside
//! the producer (`gravity_well_pull`) and assert on `Velocity2D` outcomes and
//! `ApplyBoltForce` message captures.

use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::helpers::{
    add_gravity_surge_stacks, canonical_config, install_gravity_surge_config, spawn_bolt,
    spawn_well, test_app_playing, tick_with_dt, wire_pull_only_with_consumer,
    wire_with_force_consumer,
};
use crate::{bolt::messages::ApplyBoltForce, shared::test_utils::collector::MessageCollector};

// ── Group I — Integration: pull → apply_bolt_forces → Velocity2D ─────────────

// ── Behavior 16 — producer + consumer in sequence updates Velocity2D ─────────

#[test]
fn integration_producer_and_consumer_update_velocity() {
    // wire_pull_only_with_consumer wires (gravity_well_pull, apply_bolt_forces)
    // in FixedUpdate via .chain(). Tests end-to-end: accel = 500/100 = 5,
    // dt = 1.0 → vel = -5 in X.
    let mut app = test_app_playing();
    wire_pull_only_with_consumer(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (vel.0.x - (-5.0)).abs() < 1e-3,
        "expected vel.x ≈ -5.0 (accel=5, dt=1.0), got {}",
        vel.0.x
    );
    assert!(
        vel.0.y.abs() < 1e-5,
        "expected vel.y ≈ 0.0, got {}",
        vel.0.y
    );

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected exactly one ApplyBoltForce message, got {}",
        collector.0.len()
    );
    assert_eq!(
        collector.0[0].bolt, bolt,
        "message must target the bolt entity"
    );
}

#[test]
fn integration_producer_and_consumer_update_velocity_short_dt() {
    // Edge: same setup with dt = 0.1 → vel.x ≈ -0.5 (consumer applies dt).
    // This proves the producer emits acceleration and the consumer owns dt.
    let mut app = test_app_playing();
    wire_pull_only_with_consumer(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (vel.0.x - (-0.5)).abs() < 1e-3,
        "expected vel.x ≈ -0.5 (accel=5, dt=0.1), got {}",
        vel.0.x
    );

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected exactly one ApplyBoltForce message, got {}",
        collector.0.len()
    );
    assert_eq!(
        collector.0[0].bolt, bolt,
        "message must target the bolt entity"
    );
}

// ── Behavior 17 — pre-existing velocity preserved (consumer adds, not overwrites)

#[test]
fn integration_pre_existing_velocity_preserved_after_consumer() {
    // Migrated equivalent of the old `pull_impulse_adds_to_existing_velocity`.
    // bolt starts at (3.0, 4.0); accel = -5 in X, dt = 1.0.
    // Result: (3 + -5*1, 4 + 0*1) = (-2.0, 4.0).
    let mut app = test_app_playing();
    wire_pull_only_with_consumer(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::new(3.0, 4.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (vel.0 - Vec2::new(-2.0, 4.0)).length() < 1e-3,
        "expected vel ≈ (-2.0, 4.0) (consumer adds to existing velocity), got {:?}",
        vel.0
    );

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected exactly one ApplyBoltForce message, got {}",
        collector.0.len()
    );
    assert_eq!(
        collector.0[0].bolt, bolt,
        "message must target the bolt entity"
    );
}

// ── Group J — Wire ordering: gravity_well_pull.before(BoltSystems::ApplyForces)

// ── Behavior 18 — ordering regression ────────────────────────────────────────

#[test]
fn gravity_well_pull_runs_before_bolt_systems_apply_forces() {
    // Regression test: if a future change drops the .before(BoltSystems::ApplyForces)
    // ordering, the consumer reads zero messages this tick (producer hasn't
    // written yet), the bolt's velocity remains Vec2::ZERO after the tick,
    // and this assertion fails.
    //
    // Uses wire_with_force_consumer which calls the production gravity_surge::wire
    // (with .before(BoltSystems::ApplyForces)) and manually registers
    // apply_bolt_forces in BoltSystems::ApplyForces.
    let mut app = test_app_playing();
    wire_with_force_consumer(&mut app);
    install_gravity_surge_config(&mut app, canonical_config());
    add_gravity_surge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::ZERO);
    // Pre-seed a well directly (bypass spawn_gravity_wells chain).
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    // accel = -500/100 = -5, dt = 0.1 → vel.x = -5 * 0.1 = -0.5
    assert!(
        (vel.0.x - (-0.5)).abs() < 1e-3,
        "expected vel.x ≈ -0.5 (producer-before-consumer ordering broken? got {})",
        vel.0.x
    );
    assert!(
        vel.0.y.abs() < 1e-4,
        "expected vel.y ≈ 0.0, got {}",
        vel.0.y
    );

    // Also assert on the captured message: pins that producer emitted in the
    // same tick with the correct per-bolt force.
    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected exactly 1 ApplyBoltForce message emitted this tick"
    );
    assert_eq!(
        collector.0[0].bolt, bolt,
        "message.bolt must match the spawned bolt entity"
    );
    assert!(
        (collector.0[0].force.x - (-5.0)).abs() < 1e-3,
        "expected force.x ≈ -5.0 (acceleration, not velocity), got {}",
        collector.0[0].force.x
    );
}
