//! Group B — Integration tests: emitter + consumer chain (Behavior 12).

use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::super::{
    super::system::{DriftConfig, DriftWind, drift_apply_force},
    helpers::{
        add_drift_stacks, install_drift_config, install_drift_wind, spawn_bolt, test_app_playing,
        tick_with_dt,
    },
};
use crate::bolt::{messages::ApplyBoltForce, systems::apply_bolt_forces};

/// Wires `drift_apply_force` AND the `apply_bolt_forces` consumer together for
/// integration tests (Group B). `drift_apply_force` runs before the consumer
/// so messages are drained within the same `FixedUpdate` tick.
///
/// The fixed timestep must be set to `1.0 / 64.0` s (the canonical DT) via
/// `tick_with_dt` — this helper only wires the systems.
fn wire_drift_force_pipeline(app: &mut App) {
    app.add_message::<ApplyBoltForce>();
    app.add_systems(
        FixedUpdate,
        (
            drift_apply_force,
            apply_bolt_forces.in_set(crate::bolt::sets::BoltSystems::ApplyForces),
        )
            .chain(),
    );
}

/// Fixed timestep used in Group B integration tests (matches `apply_bolt_forces`
/// canonical DT of 1/64 s).
const DT: f32 = 1.0 / 64.0;

/// Emitter + consumer chain produces the correct `Velocity2D` delta in one tick.
/// Given force = 100 * `Vec2::X` and dt = 1/64, expected delta.x ≈ 1.5625.
#[test]
fn emitter_and_consumer_chain_produces_velocity_delta() {
    let mut app = test_app_playing();
    wire_drift_force_pipeline(&mut app);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.3,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::new(1.0, 0.0),
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(DT));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    let expected_vx = 100.0 * DT;
    assert!(
        (vel.0.x - expected_vx).abs() < 1e-4,
        "expected vel.x ≈ {} (100.0 * DT), got {}",
        expected_vx,
        vel.0.x
    );
    assert_eq!(
        vel.0.y.to_bits(),
        0.0_f32.to_bits(),
        "vel.y must be exactly 0.0"
    );
}

/// Edge case 1 for Behavior 12 — multi-tick accumulation.
/// Three ticks at DT each → `Velocity2D.x ≈ 3 * 100.0 * DT = 4.6875`.
#[test]
fn emitter_and_consumer_chain_accumulates_across_three_ticks() {
    let mut app = test_app_playing();
    wire_drift_force_pipeline(&mut app);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.3,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::new(1.0, 0.0),
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(DT));
    tick_with_dt(&mut app, Duration::from_secs_f32(DT));
    tick_with_dt(&mut app, Duration::from_secs_f32(DT));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    let expected = 3.0 * 100.0 * DT;
    assert!(
        (vel.0.x - expected).abs() < 1e-4,
        "expected vel.x ≈ {} (3 * 100.0 * DT), got {}",
        expected,
        vel.0.x
    );
    assert_eq!(
        vel.0.y.to_bits(),
        0.0_f32.to_bits(),
        "vel.y must be exactly 0.0"
    );
}

/// Edge case 2 for Behavior 12 — two bolts receive independent force deltas.
/// Both `bolt_a` and `bolt_b` start at `ZERO`; after one tick both should have
/// `vel.x ≈ 100.0 * DT`. Force must NOT "fan-in" or double-count.
#[test]
fn emitter_and_consumer_chain_multi_bolt_independence() {
    let mut app = test_app_playing();
    wire_drift_force_pipeline(&mut app);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.3,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::new(1.0, 0.0),
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt_a = spawn_bolt(&mut app, Vec2::ZERO);
    let bolt_b = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(DT));

    let vel_a = app.world().get::<Velocity2D>(bolt_a).unwrap();
    let vel_b = app.world().get::<Velocity2D>(bolt_b).unwrap();
    let expected_vx = 100.0 * DT;
    assert!(
        (vel_a.0.x - expected_vx).abs() < 1e-4,
        "bolt_a vel.x expected ≈ {}, got {}",
        expected_vx,
        vel_a.0.x
    );
    assert_eq!(vel_a.0.y.to_bits(), 0.0_f32.to_bits());
    assert!(
        (vel_b.0.x - expected_vx).abs() < 1e-4,
        "bolt_b vel.x expected ≈ {}, got {}",
        expected_vx,
        vel_b.0.x
    );
    assert_eq!(vel_b.0.y.to_bits(), 0.0_f32.to_bits());
}

/// Edge case 3 for Behavior 12 — pre-existing velocity is preserved.
/// Bolt starts at `(50.0, 20.0)`; after one tick at DT, vel.x ≈ 51.5625,
/// vel.y ≈ 20.0 (force ADDS to existing velocity, does not overwrite).
#[test]
fn emitter_and_consumer_chain_preserves_pre_existing_velocity() {
    let mut app = test_app_playing();
    wire_drift_force_pipeline(&mut app);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.3,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::new(1.0, 0.0),
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::new(50.0, 20.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(DT));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    let expected_x = 100.0_f32.mul_add(DT, 50.0);
    assert!(
        (vel.0.x - expected_x).abs() < 1e-4,
        "vel.x expected ≈ {} (50.0 + 100.0*DT), got {}",
        expected_x,
        vel.0.x
    );
    assert!(
        (vel.0.y - 20.0).abs() < 1e-4,
        "vel.y must remain ≈ 20.0 (no overwrite), got {}",
        vel.0.y
    );
}
