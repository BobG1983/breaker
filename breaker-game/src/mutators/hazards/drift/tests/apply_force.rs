//! Group A — `drift_apply_force` emitter system (Wave 2 rewrite).
//!
//! After the Wave 2 migration, `drift_apply_force` emits `ApplyBoltForce`
//! messages instead of mutating `Velocity2D` directly. Every test here wires
//! ONLY `drift_apply_force` via `wire_apply_force_only(&mut app)` or
//! `attach_message_capture` — the consumer (`apply_bolt_forces`) is NOT wired.
//! Therefore `Velocity2D` MUST remain bitwise-unchanged across all Group A
//! tests; assertions target the message channel only.
//!
//! Group B (Behavior 12) is the sole place that wires both the emitter AND the
//! consumer and asserts on `Velocity2D`.
//!
//! Drift scenario coverage note: a smoke scenario exists at
//! `breaker-scenario-runner/scenarios/mechanic/drift_stack_3_smoke.scenario.ron`
//! which exercises the observable drift behavior end-to-end. No dedicated
//! Drift chaos scenario was found under `scenarios/chaos/` or
//! `scenarios/hazards/` at the time of writing — this is a coverage gap that
//! the orchestrator may want to address in a future wave.
//!
//! RNG is not required — `drift_apply_force` does not read it.

use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::{
    super::system::{DriftConfig, DriftWind, drift_apply_force},
    helpers::{
        add_drift_stacks, canonical_config, install_drift_config, install_drift_wind, spawn_bolt,
        test_app_playing, tick_with_dt, wire_apply_force_only,
    },
};
use crate::{
    bolt::{messages::ApplyBoltForce, systems::apply_bolt_forces},
    mutators::hazards::{definition::HazardKind, resources::ActiveHazards},
    shared::test_utils::collector::{MessageCollector, attach_message_capture},
};

// ── Harness helpers ───────────────────────────────────────────────────────────

/// Returns the collected `ApplyBoltForce` messages from this tick.
fn captured_forces(app: &App) -> Vec<ApplyBoltForce> {
    app.world()
        .resource::<MessageCollector<ApplyBoltForce>>()
        .0
        .clone()
}

/// Builds a test app with the playing state, `ActiveHazards`, and
/// `ApplyBoltForce` message capture already installed.
fn test_app_playing_with_force_capture() -> App {
    let mut app = test_app_playing();
    attach_message_capture::<ApplyBoltForce>(&mut app);
    app
}

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

// ── Group A — Emitter behaviors (Behaviors 1–11) ──────────────────────────────

// ── Behavior 1 — stack 1, direction (1,0), force 100 → one message per bolt ──

/// The emitter emits exactly one `ApplyBoltForce { bolt, force: (100, 0) }`
/// per active bolt at stack 1. The bolt's `Velocity2D` must remain bitwise-ZERO.
#[test]
fn emitter_writes_one_message_per_bolt_at_stack_1() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
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

    tick_with_dt(&mut app, Duration::from_secs(1));

    let forces = captured_forces(&app);
    assert_eq!(
        forces.len(),
        1,
        "exactly one message expected, got {}",
        forces.len()
    );
    assert_eq!(forces[0].bolt, bolt_a, "message must address bolt_a");
    assert!(
        (forces[0].force.x - 100.0).abs() < 1e-5,
        "force.x should be 100.0, got {}",
        forces[0].force.x
    );
    assert!(
        forces[0].force.y.abs() < 1e-5,
        "force.y should be 0.0, got {}",
        forces[0].force.y
    );
    let vel = app.world().get::<Velocity2D>(bolt_a).unwrap();
    assert_eq!(
        vel.0.x.to_bits(),
        0.0_f32.to_bits(),
        "emitter must NOT mutate velocity.x (bitwise-zero expected)"
    );
    assert_eq!(
        vel.0.y.to_bits(),
        0.0_f32.to_bits(),
        "emitter must NOT mutate velocity.y (bitwise-zero expected)"
    );
}

/// Edge case for Behavior 1 — re-run the same setup twice in a row on the
/// same app. Both ticks must each emit exactly one message carrying
/// `Vec2::new(100.0, 0.0)`, and `Velocity2D` must remain bitwise-ZERO
/// after both ticks. This confirms the emitter is truly idempotent on
/// velocity across consecutive emissions.
#[test]
fn emitter_is_idempotent_on_velocity_across_consecutive_ticks() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
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

    // First tick
    tick_with_dt(&mut app, Duration::from_secs(1));
    {
        let forces = captured_forces(&app);
        assert_eq!(forces.len(), 1, "tick 1: exactly one message expected");
        assert!((forces[0].force.x - 100.0).abs() < 1e-5);
    }
    {
        let vel = app.world().get::<Velocity2D>(bolt_a).unwrap();
        assert_eq!(
            vel.0.x.to_bits(),
            0.0_f32.to_bits(),
            "tick 1: velocity.x must be bitwise-zero"
        );
        assert_eq!(
            vel.0.y.to_bits(),
            0.0_f32.to_bits(),
            "tick 1: velocity.y must be bitwise-zero"
        );
    }

    // Second tick (same dt)
    tick_with_dt(&mut app, Duration::from_secs(1));
    {
        let forces = captured_forces(&app);
        assert_eq!(forces.len(), 1, "tick 2: exactly one message expected");
        assert!(
            (forces[0].force.x - 100.0).abs() < 1e-5,
            "tick 2: force.x must be 100.0"
        );
    }
    {
        let vel = app.world().get::<Velocity2D>(bolt_a).unwrap();
        assert_eq!(
            vel.0.x.to_bits(),
            0.0_f32.to_bits(),
            "tick 2: velocity.x must remain bitwise-zero"
        );
        assert_eq!(
            vel.0.y.to_bits(),
            0.0_f32.to_bits(),
            "tick 2: velocity.y must remain bitwise-zero"
        );
    }
}

// ── Behavior 2 — emitted force does not depend on dt ─────────────────────────

/// With dt = 1ms, the emitted force must still be exactly `Vec2::new(100.0, 0.0)`.
/// The emitter MUST NOT pre-multiply by dt (that is the consumer's job).
#[test]
fn emitted_force_is_independent_of_dt_1ms() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
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
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let _bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_millis(1));

    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 1, "one message expected");
    assert!(
        (forces[0].force.x - 100.0).abs() < 1e-5,
        "force.x must be 100.0 regardless of dt=1ms (not 0.1), got {}",
        forces[0].force.x
    );
    assert!(forces[0].force.y.abs() < 1e-5);
}

/// Edge case for Behavior 2 — fresh app with dt = 10s. Force must still be
/// exactly `Vec2::new(100.0, 0.0)`. Uses a SEPARATE app to avoid the
/// mid-run `set_timestep` pathology documented at `apply_force.rs:511–520`.
#[test]
fn emitted_force_is_independent_of_dt_10s() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
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
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let _bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs(10));

    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 1, "one message expected");
    assert!(
        (forces[0].force.x - 100.0).abs() < 1e-5,
        "force.x must be 100.0 regardless of dt=10s, got {}",
        forces[0].force.x
    );
    assert!(forces[0].force.y.abs() < 1e-5);
}

// ── Behavior 3 — stack 3, downward direction → force = (0, -166) ─────────────

/// At 3 Drift stacks and direction (0, -1), the emitted force must be
/// `Vec2::new(0.0, -166.0)` (100 + 33*2 = 166 in the -Y direction).
#[test]
fn emitter_scales_force_with_stack_count() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.0,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::new(0.0, -1.0),
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 3);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs(1));

    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 1, "one message expected");
    assert!(
        forces[0].force.x.abs() < 1e-3,
        "force.x should be 0.0, got {}",
        forces[0].force.x
    );
    assert!(
        (forces[0].force.y - -166.0).abs() < 1e-3,
        "force.y should be -166.0 (100 + 33*2), got {}",
        forces[0].force.y
    );
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(
        vel.0.x.to_bits(),
        0.0_f32.to_bits(),
        "velocity.x must be bitwise-zero"
    );
    assert_eq!(
        vel.0.y.to_bits(),
        0.0_f32.to_bits(),
        "velocity.y must be bitwise-zero"
    );
}

/// Edge case for Behavior 3 — `per_level_force: 0.0` at 3 stacks.
/// Only the base force (100.0) applies.
#[test]
fn emitter_force_uses_only_base_when_per_level_force_is_zero() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 0.0,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::new(0.0, -1.0),
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 3);
    let _bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs(1));

    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 1);
    assert!(forces[0].force.x.abs() < 1e-3);
    assert!(
        (forces[0].force.y - -100.0).abs() < 1e-3,
        "with per_level_force=0.0 only base applies: expected -100.0, got {}",
        forces[0].force.y
    );
}

// ── Behavior 4 — multi-bolt fan-out ──────────────────────────────────────────

/// With 3 bolts in the world, the emitter emits exactly 3 messages — one per
/// bolt — each carrying the same `force: Vec2::X * 100.0`. All three bolts'
/// `Velocity2D` values must remain bitwise-unchanged.
#[test]
fn emitter_writes_per_bolt_messages_for_all_active_bolts() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt_a = spawn_bolt(&mut app, Vec2::ZERO);
    let bolt_b = spawn_bolt(&mut app, Vec2::new(50.0, 0.0));
    let bolt_c = spawn_bolt(&mut app, Vec2::new(-50.0, 25.0));

    tick_with_dt(&mut app, Duration::from_secs(1));

    let forces = captured_forces(&app);
    assert_eq!(
        forces.len(),
        3,
        "exactly 3 messages expected for 3 bolts, got {}",
        forces.len()
    );

    // Each message carries the same force vector
    for msg in &forces {
        assert!(
            (msg.force.x - 100.0).abs() < 1e-5,
            "force.x must be 100.0 for every message, got {}",
            msg.force.x
        );
        assert!(
            msg.force.y.abs() < 1e-5,
            "force.y must be 0.0, got {}",
            msg.force.y
        );
    }

    // The set of bolt fields is exactly {bolt_a, bolt_b, bolt_c} — no duplicates
    let addressed: std::collections::HashSet<Entity> = forces.iter().map(|m| m.bolt).collect();
    assert!(addressed.contains(&bolt_a), "message missing for bolt_a");
    assert!(addressed.contains(&bolt_b), "message missing for bolt_b");
    assert!(addressed.contains(&bolt_c), "message missing for bolt_c");
    assert_eq!(addressed.len(), 3, "no duplicate messages");

    // Velocities remain bitwise-unchanged from spawn values
    let vel_a = app.world().get::<Velocity2D>(bolt_a).unwrap();
    assert_eq!(vel_a.0.x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(vel_a.0.y.to_bits(), 0.0_f32.to_bits());

    let vel_b = app.world().get::<Velocity2D>(bolt_b).unwrap();
    assert_eq!(vel_b.0.x.to_bits(), 50.0_f32.to_bits());
    assert_eq!(vel_b.0.y.to_bits(), 0.0_f32.to_bits());

    let vel_c = app.world().get::<Velocity2D>(bolt_c).unwrap();
    assert_eq!(vel_c.0.x.to_bits(), (-50.0_f32).to_bits());
    assert_eq!(vel_c.0.y.to_bits(), 25.0_f32.to_bits());
}

/// Edge case for Behavior 4 — 0 bolts → 0 messages, no panic.
#[test]
fn emitter_emits_no_messages_when_no_bolts_in_world() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    // No bolts spawned.

    tick_with_dt(&mut app, Duration::from_secs(1));

    let forces = captured_forces(&app);
    assert_eq!(
        forces.len(),
        0,
        "0 messages expected when no bolts in world"
    );
}

// ── Behavior 5 — zero stacks → no messages ───────────────────────────────────

/// With zero Drift stacks, the emitter emits no messages.
/// The bolt's `Velocity2D` must be bitwise-equal to its spawn value.
#[test]
fn emitter_is_noop_at_zero_stacks() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    // No add_drift_stacks → stacks == 0
    let bolt = spawn_bolt(&mut app, Vec2::new(5.0, 7.0));

    tick_with_dt(&mut app, Duration::from_secs(1));

    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 0, "zero messages at zero stacks");
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(
        vel.0.x.to_bits(),
        5.0_f32.to_bits(),
        "velocity.x must remain bitwise 5.0"
    );
    assert_eq!(
        vel.0.y.to_bits(),
        7.0_f32.to_bits(),
        "velocity.y must remain bitwise 7.0"
    );
}

/// Edge case for Behavior 5 — stacks raised to 1, then force-inserted back to
/// 0 before tick. Result must still be zero messages.
#[test]
fn emitter_is_noop_when_stacks_forced_to_zero_before_tick() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::new(5.0, 7.0));

    // Force stacks back to 0 before ticking
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .force_insert_entry(HazardKind::Drift, 0);

    tick_with_dt(&mut app, Duration::from_secs(1));

    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 0, "zero messages when stacks forced to 0");
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(vel.0.x.to_bits(), 5.0_f32.to_bits());
    assert_eq!(vel.0.y.to_bits(), 7.0_f32.to_bits());
}

// ── Behavior 6 — zero force magnitude → no messages ──────────────────────────

/// When `force_magnitude` computes to zero (`force: 0.0, per_level_force: 0.0`),
/// no messages must be emitted even with 5 stacks.
#[test]
fn emitter_is_noop_when_computed_force_magnitude_is_zero() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           0.0,
            period_secs:     8.0,
            per_level_force: 0.0,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 5);
    let bolt = spawn_bolt(&mut app, Vec2::new(10.0, 20.0));

    tick_with_dt(&mut app, Duration::from_secs(1));

    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 0, "zero messages when force magnitude is 0");
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(vel.0.x.to_bits(), 10.0_f32.to_bits());
    assert_eq!(vel.0.y.to_bits(), 20.0_f32.to_bits());
}

/// Edge case for Behavior 6 — negative `force` is also silenced by the
/// `<= 0.0` gate.
#[test]
fn emitter_is_noop_when_force_is_negative() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           -50.0,
            period_secs:     8.0,
            per_level_force: 0.0,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::new(10.0, 20.0));

    tick_with_dt(&mut app, Duration::from_secs(1));

    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 0, "zero messages when force is negative");
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(vel.0.x.to_bits(), 10.0_f32.to_bits());
    assert_eq!(vel.0.y.to_bits(), 20.0_f32.to_bits());
}

// ── Behavior 7 — missing DriftConfig → no messages ───────────────────────────

/// Without a `DriftConfig` resource, the emitter emits no messages.
#[test]
fn emitter_is_noop_when_drift_config_missing() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::new(10.0, 20.0));
    // No DriftConfig inserted.

    tick_with_dt(&mut app, Duration::from_secs(1));

    let forces = captured_forces(&app);
    assert_eq!(
        forces.len(),
        0,
        "zero messages when DriftConfig missing (tick 1)"
    );
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(vel.0.x.to_bits(), 10.0_f32.to_bits());
    assert_eq!(vel.0.y.to_bits(), 20.0_f32.to_bits());
}

/// Edge case for Behavior 7 — two consecutive ticks. Message channel must
/// remain empty after EACH tick individually; bolt velocity must remain
/// bitwise-unchanged after EACH tick.
#[test]
fn emitter_is_noop_when_drift_config_missing_across_two_ticks() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::new(10.0, 20.0));

    // First tick — assert immediately after
    tick_with_dt(&mut app, Duration::from_secs(1));
    {
        let forces = captured_forces(&app);
        assert_eq!(forces.len(), 0, "zero messages after tick 1");
    }
    {
        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        assert_eq!(
            vel.0.x.to_bits(),
            10.0_f32.to_bits(),
            "velocity.x bitwise-unchanged after tick 1"
        );
        assert_eq!(
            vel.0.y.to_bits(),
            20.0_f32.to_bits(),
            "velocity.y bitwise-unchanged after tick 1"
        );
    }

    // Second tick — assert immediately after
    tick_with_dt(&mut app, Duration::from_secs(1));
    {
        let forces = captured_forces(&app);
        assert_eq!(forces.len(), 0, "zero messages after tick 2");
    }
    {
        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        assert_eq!(
            vel.0.x.to_bits(),
            10.0_f32.to_bits(),
            "velocity.x bitwise-unchanged after tick 2"
        );
        assert_eq!(
            vel.0.y.to_bits(),
            20.0_f32.to_bits(),
            "velocity.y bitwise-unchanged after tick 2"
        );
    }
}

// ── Behavior 8 — missing DriftWind → no messages ─────────────────────────────

/// Without a `DriftWind` resource, the emitter emits no messages.
#[test]
fn emitter_is_noop_when_drift_wind_missing() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::new(10.0, 20.0));
    // No DriftWind inserted.

    tick_with_dt(&mut app, Duration::from_secs(1));

    let forces = captured_forces(&app);
    assert_eq!(
        forces.len(),
        0,
        "zero messages when DriftWind missing (tick 1)"
    );
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(vel.0.x.to_bits(), 10.0_f32.to_bits());
    assert_eq!(vel.0.y.to_bits(), 20.0_f32.to_bits());
}

/// Edge case for Behavior 8 — two consecutive ticks. Message channel must
/// remain empty after EACH tick individually; bolt velocity must remain
/// bitwise-unchanged after EACH tick.
#[test]
fn emitter_is_noop_when_drift_wind_missing_across_two_ticks() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::new(10.0, 20.0));

    // First tick — assert immediately after
    tick_with_dt(&mut app, Duration::from_secs(1));
    {
        let forces = captured_forces(&app);
        assert_eq!(forces.len(), 0, "zero messages after tick 1");
    }
    {
        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        assert_eq!(
            vel.0.x.to_bits(),
            10.0_f32.to_bits(),
            "velocity.x bitwise-unchanged after tick 1"
        );
        assert_eq!(
            vel.0.y.to_bits(),
            20.0_f32.to_bits(),
            "velocity.y bitwise-unchanged after tick 1"
        );
    }

    // Second tick — assert immediately after
    tick_with_dt(&mut app, Duration::from_secs(1));
    {
        let forces = captured_forces(&app);
        assert_eq!(forces.len(), 0, "zero messages after tick 2");
    }
    {
        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        assert_eq!(
            vel.0.x.to_bits(),
            10.0_f32.to_bits(),
            "velocity.x bitwise-unchanged after tick 2"
        );
        assert_eq!(
            vel.0.y.to_bits(),
            20.0_f32.to_bits(),
            "velocity.y bitwise-unchanged after tick 2"
        );
    }
}

// ── Behavior 9 — non-Bolt entities with Velocity2D are excluded ──────────────

/// A non-Bolt entity with `Velocity2D` must not receive an `ApplyBoltForce`
/// message. The `With<Bolt>` filter on the emitter query excludes them.
#[test]
fn emitter_does_not_address_messages_to_non_bolt_entities() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let non_bolt = app.world_mut().spawn(Velocity2D(Vec2::new(5.0, 5.0))).id();

    tick_with_dt(&mut app, Duration::from_secs(1));

    let forces = captured_forces(&app);
    assert_eq!(
        forces.len(),
        0,
        "non-Bolt entity must not receive a force message"
    );
    let vel = app.world().get::<Velocity2D>(non_bolt).unwrap();
    assert_eq!(vel.0.x.to_bits(), 5.0_f32.to_bits());
    assert_eq!(vel.0.y.to_bits(), 5.0_f32.to_bits());
}

/// Edge case for Behavior 9 — one Bolt and one non-Bolt entity with
/// `Velocity2D` simultaneously. Exactly one message is captured, addressed
/// to the Bolt entity.
#[test]
fn emitter_addresses_only_bolt_entity_when_mixed_with_non_bolt() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);
    let non_bolt = app.world_mut().spawn(Velocity2D(Vec2::new(5.0, 5.0))).id();

    tick_with_dt(&mut app, Duration::from_secs(1));

    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 1, "exactly one message — for the Bolt only");
    assert_eq!(
        forces[0].bolt, bolt,
        "message must be addressed to the Bolt entity"
    );
    assert_ne!(
        forces[0].bolt, non_bolt,
        "message must NOT be addressed to the non-Bolt entity"
    );
    let vel = app.world().get::<Velocity2D>(non_bolt).unwrap();
    assert_eq!(
        vel.0.x.to_bits(),
        5.0_f32.to_bits(),
        "non-Bolt velocity.x bitwise-unchanged"
    );
    assert_eq!(
        vel.0.y.to_bits(),
        5.0_f32.to_bits(),
        "non-Bolt velocity.y bitwise-unchanged"
    );
}

// ── Behavior 10 — 45° direction → equal X/Y components ──────────────────────

/// At 45° direction (`FRAC_1_SQRT_2, FRAC_1_SQRT_2`), the emitted force
/// must have equal X and Y components and total magnitude 100.0.
#[test]
fn emitter_at_45_degrees_produces_equal_xy_components_in_force() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::new(
                std::f32::consts::FRAC_1_SQRT_2,
                std::f32::consts::FRAC_1_SQRT_2,
            ),
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let _bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs(1));

    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 1);
    let force = forces[0].force;
    assert!(
        (force.x - force.y).abs() < 1e-5,
        "45° should produce equal X and Y components in force, got {force:?}",
    );
    assert!(
        (force.length() - 100.0).abs() < 1e-3,
        "force magnitude should equal 100.0, got {}",
        force.length()
    );
}

/// Edge case for Behavior 10 — upper-left quadrant direction
/// `(-FRAC_1_SQRT_2, FRAC_1_SQRT_2)` → force.x < 0, force.y > 0,
/// `(force.x + force.y).abs() < 1e-5`.
#[test]
fn emitter_at_upper_left_45_degrees_produces_negative_x_positive_y_in_force() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::new(
                -std::f32::consts::FRAC_1_SQRT_2,
                std::f32::consts::FRAC_1_SQRT_2,
            ),
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let _bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs(1));

    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 1);
    let force = forces[0].force;
    assert!(
        force.x < 0.0,
        "upper-left direction must have negative force.x, got {}",
        force.x
    );
    assert!(
        force.y > 0.0,
        "upper-left direction must have positive force.y, got {}",
        force.y
    );
    assert!(
        (force.x + force.y).abs() < 1e-5,
        "X and Y magnitudes should cancel to near zero, got force={force:?}",
    );
}

// ── Behavior 11 — mid-run bolt spawn picked up on next tick ──────────────────

/// Bolt spawned mid-run is included in the emitter's query on the next tick.
/// `bolt_a` receives one message on tick 1; `bolt_b` receives one on tick 2.
/// Neither bolt's `Velocity2D` is mutated by the emitter.
#[test]
fn emitter_picks_up_bolt_spawned_mid_run_on_next_tick() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt_a = spawn_bolt(&mut app, Vec2::ZERO);

    // Tick 1 — only bolt_a
    tick_with_dt(&mut app, Duration::from_secs(1));
    {
        let forces = captured_forces(&app);
        assert_eq!(forces.len(), 1, "tick 1: one message for bolt_a");
        assert_eq!(forces[0].bolt, bolt_a);
        assert!((forces[0].force.x - 100.0).abs() < 1e-5);
    }

    // Spawn bolt_b, then tick 2 — both bolts
    let bolt_b = spawn_bolt(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs(1));
    {
        let forces = captured_forces(&app);
        assert_eq!(forces.len(), 2, "tick 2: two messages (bolt_a + bolt_b)");
        let addressed: std::collections::HashSet<Entity> = forces.iter().map(|m| m.bolt).collect();
        assert!(
            addressed.contains(&bolt_a),
            "tick 2: missing message for bolt_a"
        );
        assert!(
            addressed.contains(&bolt_b),
            "tick 2: missing message for bolt_b"
        );
        for msg in &forces {
            assert!((msg.force.x - 100.0).abs() < 1e-5, "force.x must be 100.0");
            assert!(msg.force.y.abs() < 1e-5, "force.y must be 0.0");
        }
    }

    // Velocities remain bitwise-ZERO (emitter-only test, no consumer wired)
    let vel_a = app.world().get::<Velocity2D>(bolt_a).unwrap();
    let vel_b = app.world().get::<Velocity2D>(bolt_b).unwrap();
    assert_eq!(vel_a.0.x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(vel_a.0.y.to_bits(), 0.0_f32.to_bits());
    assert_eq!(vel_b.0.x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(vel_b.0.y.to_bits(), 0.0_f32.to_bits());
}

/// Edge case for Behavior 11 — despawn `bolt_a` between tick 2 and tick 3,
/// spawn `bolt_c` before tick 3. Tick 3 must have exactly two messages:
/// one for `bolt_b`, one for `bolt_c`. No message for the despawned `bolt_a`.
/// All `Velocity2D` values remain bitwise-unchanged from their spawn values
/// throughout all three ticks (emitter only — no consumer wired).
#[test]
fn emitter_excludes_despawned_bolt_and_includes_newly_spawned_bolt() {
    let mut app = test_app_playing_with_force_capture();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);

    let bolt_a = spawn_bolt(&mut app, Vec2::ZERO);

    // Tick 1 — bolt_a only
    tick_with_dt(&mut app, Duration::from_secs(1));
    {
        let vel_a = app.world().get::<Velocity2D>(bolt_a).unwrap();
        assert_eq!(
            vel_a.0.x.to_bits(),
            0.0f32.to_bits(),
            "bolt_a vel.x must be bitwise-zero after tick 1"
        );
        assert_eq!(
            vel_a.0.y.to_bits(),
            0.0f32.to_bits(),
            "bolt_a vel.y must be bitwise-zero after tick 1"
        );
    }

    let bolt_b = spawn_bolt(&mut app, Vec2::ZERO);

    // Tick 2 — bolt_a and bolt_b
    tick_with_dt(&mut app, Duration::from_secs(1));
    {
        let vel_a = app.world().get::<Velocity2D>(bolt_a).unwrap();
        assert_eq!(
            vel_a.0.x.to_bits(),
            0.0f32.to_bits(),
            "bolt_a vel.x must be bitwise-zero after tick 2"
        );
        assert_eq!(
            vel_a.0.y.to_bits(),
            0.0f32.to_bits(),
            "bolt_a vel.y must be bitwise-zero after tick 2"
        );
    }

    // Between tick 2 and 3: despawn bolt_a, spawn bolt_c
    app.world_mut().despawn(bolt_a);
    let bolt_c = spawn_bolt(&mut app, Vec2::ZERO);

    // Tick 3 — bolt_b and bolt_c only
    tick_with_dt(&mut app, Duration::from_secs(1));
    {
        let forces = captured_forces(&app);
        assert_eq!(
            forces.len(),
            2,
            "tick 3: exactly 2 messages (bolt_b + bolt_c)"
        );
        let addressed: std::collections::HashSet<Entity> = forces.iter().map(|m| m.bolt).collect();
        assert!(
            !addressed.contains(&bolt_a),
            "tick 3: despawned bolt_a must NOT receive a message"
        );
        assert!(
            addressed.contains(&bolt_b),
            "tick 3: bolt_b must receive a message"
        );
        assert!(
            addressed.contains(&bolt_c),
            "tick 3: bolt_c must receive a message"
        );
        for msg in &forces {
            assert!((msg.force.x - 100.0).abs() < 1e-5, "force.x must be 100.0");
            assert!(msg.force.y.abs() < 1e-5);
        }
    }

    // bolt_b and bolt_c velocities must be bitwise-ZERO (emitter-only test)
    let vel_b = app.world().get::<Velocity2D>(bolt_b).unwrap();
    let vel_c = app.world().get::<Velocity2D>(bolt_c).unwrap();
    assert_eq!(
        vel_b.0.x.to_bits(),
        0.0_f32.to_bits(),
        "bolt_b velocity.x bitwise-zero"
    );
    assert_eq!(
        vel_b.0.y.to_bits(),
        0.0_f32.to_bits(),
        "bolt_b velocity.y bitwise-zero"
    );
    assert_eq!(
        vel_c.0.x.to_bits(),
        0.0_f32.to_bits(),
        "bolt_c velocity.x bitwise-zero"
    );
    assert_eq!(
        vel_c.0.y.to_bits(),
        0.0_f32.to_bits(),
        "bolt_c velocity.y bitwise-zero"
    );
}

// ── Group B — Integration test: emitter + consumer chain (Behavior 12) ───────

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
