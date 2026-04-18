//! Group H — Multi-hazard synergy (light).
//!
//! Gravity Surge's sole observable side-effect on bolts is mutating
//! `Velocity2D.0` directly. Drift also mutates `Velocity2D.0` directly.
//! These tests pin that both hazards, when both chains run on the same
//! tick, compose cleanly — forces sum additively on the bolt's velocity
//! with no interference.

use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::{
    super::system::register,
    helpers::{
        add_gravity_surge_stacks, canonical_config, insert_seeded_rng,
        install_gravity_surge_config, spawn_bolt, spawn_well, test_app_playing, tick_with_dt,
    },
};
use crate::hazard::{
    definition::HazardKind,
    hazards::drift::{
        register as drift_register,
        system::{DriftConfig, DriftWind},
    },
    resources::ActiveHazards,
};

// ── Behavior 51 — GravitySurge + Drift: forces sum on same tick ────────

#[test]
fn gravity_surge_and_drift_forces_sum_on_same_tick() {
    let mut app = test_app_playing();
    // Register both hazards.
    drift_register(&mut app);
    register(&mut app);

    // Install both configs + Drift wind (+X, long timer so direction is stable).
    app.world_mut().insert_resource(DriftConfig {
        force:           100.0,
        period_secs:     8.0,
        per_level_force: 33.3,
    });
    app.world_mut().insert_resource(DriftWind {
        direction: Vec2::X,
        timer:     8.0,
    });
    install_gravity_surge_config(&mut app, canonical_config());

    // Seed RNG for Drift's wind tick (it reads GameRng).
    insert_seeded_rng(&mut app, 42);

    // Activate both hazards (stacks > 0 opens the run_if gates).
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Drift);
    add_gravity_surge_stacks(&mut app, 1);

    // Seed bolt and gravity well directly (avoid triggering Destroyed<Cell>).
    let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::ZERO);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    // Expected velocity:
    //   Drift: direction(+X) * force(100) * dt(0.1) = +10 in X
    //   Gravity: well at origin pulls bolt at (100, 0) in -X:
    //            accel = 500/100 = 5 u/s², delta = 5 * 0.1 = 0.5 in -X.
    //   Net: 10 - 0.5 = 9.5 in X.
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (vel.0.x - 9.5).abs() < 1e-2,
        "expected x ≈ 9.5, got {}",
        vel.0.x
    );
    assert!(vel.0.y.abs() < 1e-4);

    // Drift wind timer decremented by dt.
    let wind = app.world().resource::<DriftWind>();
    assert!((wind.timer - 7.9).abs() < 1e-4);
}

#[test]
fn gravity_and_drift_both_positive_x_when_bolt_at_minus_hundred() {
    // Edge: bolt at (-100, 0) — gravity pulls +X (toward origin); Drift
    // still pushes +X. Both add: 10 + 0.5 = 10.5.
    let mut app = test_app_playing();
    drift_register(&mut app);
    register(&mut app);

    app.world_mut().insert_resource(DriftConfig {
        force:           100.0,
        period_secs:     8.0,
        per_level_force: 33.3,
    });
    app.world_mut().insert_resource(DriftWind {
        direction: Vec2::X,
        timer:     8.0,
    });
    install_gravity_surge_config(&mut app, canonical_config());
    insert_seeded_rng(&mut app, 42);

    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Drift);
    add_gravity_surge_stacks(&mut app, 1);

    let bolt = spawn_bolt(&mut app, Vec2::new(-100.0, 0.0), Vec2::ZERO);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    // Drift +10, gravity +0.5 → 10.5.
    assert!(
        (vel.0.x - 10.5).abs() < 1e-2,
        "expected x ≈ 10.5, got {}",
        vel.0.x
    );
    assert!(vel.0.y.abs() < 1e-4);
}
