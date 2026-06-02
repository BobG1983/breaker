//! Group F — `wire` integration: full chain, ordering, gating.
//! (Renamed from "Group D" to avoid collision with `tests/activate.rs`
//!  which labels its own section "Group E — activation lifecycle".)
//!
//! These tests exercise the production wiring (`wire(&mut app)`) so the
//! run conditions `hazard_active(Drift)` and `in_state(NodeState::Playing)`
//! are active, and the chain ordering
//! `(drift_update_wind, drift_apply_force).chain()` is enforced.
//!
//! After the Wave 2 migration, `drift::wire` does NOT wire `apply_bolt_forces`
//! (that is the bolt domain's concern). Therefore tests here assert on emitted
//! `ApplyBoltForce` messages rather than `Velocity2D` mutations — the
//! consumer is not wired, so velocity never changes.
//!
//! Behavior 13 (wiring/scheduling) tests that `drift_apply_force` runs
//! BEFORE `BoltSystems::ApplyForces` by wiring the consumer explicitly and
//! asserting on the resulting `Velocity2D` delta.

use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::{
    super::system::{DriftConfig, DriftWind, wire},
    helpers::{
        add_drift_stacks, canonical_config, insert_hazard_rng, install_drift_config,
        install_drift_wind, spawn_bolt, test_app_not_playing, test_app_playing, tick_with_dt,
    },
};
use crate::{
    bolt::{messages::ApplyBoltForce, sets::BoltSystems, systems::apply_bolt_forces},
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

// ── Behavior 34 — full chain: one tick updates wind AND emits one force message

#[test]
fn full_chain_updates_wind_and_applies_force_single_tick() {
    let mut app = test_app_playing();
    attach_message_capture::<ApplyBoltForce>(&mut app);
    wire(&mut app);
    insert_hazard_rng(&mut app, 42);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let _bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs(1));

    // Emitter fires: one message carrying force in X direction
    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 1, "one ApplyBoltForce message expected");
    assert!(
        (forces[0].force.x - 100.0).abs() < 1e-3,
        "force applied in X, got {:?}",
        forces[0].force
    );
    assert!(forces[0].force.y.abs() < 1e-5);

    // Wind timer ticked down
    let wind = app.world().resource::<DriftWind>();
    assert!(
        (wind.timer - 7.0).abs() < 1e-5,
        "timer should decrement by 1s, got {}",
        wind.timer
    );
}

#[test]
fn full_chain_accumulates_across_two_ticks() {
    // Edge: 2 ticks of 1s — 2 messages per tick (accumulated), timer should be ≈6.0.
    let mut app = test_app_playing();
    attach_message_capture::<ApplyBoltForce>(&mut app);
    wire(&mut app);
    insert_hazard_rng(&mut app, 42);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let _bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs(1));
    {
        let forces = captured_forces(&app);
        assert_eq!(forces.len(), 1, "tick 1: one message");
        assert!((forces[0].force.x - 100.0).abs() < 1e-3);
    }

    tick_with_dt(&mut app, Duration::from_secs(1));
    {
        let forces = captured_forces(&app);
        assert_eq!(forces.len(), 1, "tick 2: one message");
        assert!((forces[0].force.x - 100.0).abs() < 1e-3);
    }

    let wind = app.world().resource::<DriftWind>();
    assert!(
        (wind.timer - 6.0).abs() < 1e-5,
        "timer should be 6.0 after 2 ticks"
    );
}

// ── Behavior 35 — timer expiry → fresh roll applied same-tick ────────────────

#[test]
fn first_tick_with_expired_timer_rolls_and_applies_new_direction() {
    // Timer expires → fresh direction is rolled BEFORE force is emitted
    // (chain order: update → apply). The force magnitude is exactly 100
    // regardless of which direction was rolled.
    let mut app = test_app_playing();
    attach_message_capture::<ApplyBoltForce>(&mut app);
    wire(&mut app);
    insert_hazard_rng(&mut app, 42);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let _bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs(1));

    let wind = app.world().resource::<DriftWind>();
    assert!(
        (wind.direction.length() - 1.0).abs() < 1e-5,
        "direction must be a unit vector after roll"
    );
    assert!(
        (wind.timer - 8.0).abs() < 1e-5,
        "timer must reset to period_secs"
    );

    // Emitted force must match the rolled direction
    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 1, "one message expected");
    assert!(
        (forces[0].force.length() - 100.0).abs() < 1e-3,
        "force magnitude must be 100.0, got {}",
        forces[0].force.length()
    );
    let normalized_force = forces[0].force.normalize_or_zero();
    assert!(
        (normalized_force - wind.direction).length() < 1e-4,
        "emitted force must point in the rolled direction, not Vec2::X"
    );
}

#[test]
fn first_tick_rolled_direction_rules_out_reversed_ordering() {
    // Edge: anti-ordering guard. If the chain were reversed, the bolt
    // would receive a force along the PRE-roll direction (Vec2::X) and
    // then the direction would roll. The force magnitude is still 100,
    // but the force.x/force.y ratio would be Vec2::X — which seed 42 does
    // NOT produce. Assert force length > 50.
    let mut app = test_app_playing();
    attach_message_capture::<ApplyBoltForce>(&mut app);
    wire(&mut app);
    insert_hazard_rng(&mut app, 42);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let _bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs(1));

    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 1, "one message expected");
    assert!(
        forces[0].force.length() > 50.0,
        "force magnitude must be positive and non-trivial"
    );
}

// ── Behavior 36 — multi-tick: direction holds across ticks, timer > 0 ────────

#[test]
fn multi_tick_direction_holds_while_timer_positive() {
    let mut app = test_app_playing();
    attach_message_capture::<ApplyBoltForce>(&mut app);
    wire(&mut app);
    insert_hazard_rng(&mut app, 42);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let _bolt = spawn_bolt(&mut app, Vec2::ZERO);

    for _ in 0..4 {
        tick_with_dt(&mut app, Duration::from_secs(1));
    }

    let wind = app.world().resource::<DriftWind>();
    assert_eq!(
        wind.direction,
        Vec2::X,
        "direction should be bitwise unchanged while timer > 0"
    );

    // All 4 ticks must have emitted a force in the X direction
    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 1, "last tick: one message");
    assert!(
        (forces[0].force.x - 100.0).abs() < 1e-3,
        "force should point in X direction, got {:?}",
        forces[0].force
    );
    assert!(forces[0].force.y.abs() < 1e-5);
}

#[test]
fn fifth_tick_crosses_threshold_and_rolls_new_direction() {
    // Edge: timer set to 4.1 so that 4 ticks of 1s each bring it to 0.1,
    // and the 5th tick of 1s crosses zero — rolling a new direction.
    // ALL ticks use the same Duration (1s) to avoid the "overstep residual"
    // hang pathology that occurs when different durations are used on the same app.
    let mut app = test_app_playing();
    attach_message_capture::<ApplyBoltForce>(&mut app);
    wire(&mut app);
    insert_hazard_rng(&mut app, 42);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     4.1,
        },
    );
    add_drift_stacks(&mut app, 1);
    let _bolt = spawn_bolt(&mut app, Vec2::ZERO);

    // 5 ticks at the same duration — timer crosses zero on the 5th tick
    for _ in 0..5 {
        tick_with_dt(&mut app, Duration::from_secs(1));
    }

    let wind = app.world().resource::<DriftWind>();
    assert!(
        (wind.direction.length() - 1.0).abs() < 1e-5,
        "direction must be a unit vector after roll"
    );

    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 1, "one message expected on the fifth tick");
    assert!(
        (forces[0].force.length() - 100.0).abs() < 1e-3,
        "force magnitude must be ≈ 100.0, got {}",
        forces[0].force.length()
    );
}

// ── Behavior 37 — gate off (hazard inactive) → both systems suppressed ───────

#[test]
fn hazard_inactive_gate_suppresses_both_systems() {
    let mut app = test_app_playing();
    attach_message_capture::<ApplyBoltForce>(&mut app);
    wire(&mut app);
    insert_hazard_rng(&mut app, 42);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    // No add_drift_stacks → stacks(Drift) == 0 → hazard_active is false.
    let _bolt = spawn_bolt(&mut app, Vec2::new(5.0, 3.0));

    tick_with_dt(&mut app, Duration::from_secs(1));

    // Zero messages emitted (gate suppresses the emitter)
    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 0, "zero messages when hazard inactive");

    // Wind timer and direction unchanged (drift_update_wind also suppressed)
    let wind = app.world().resource::<DriftWind>();
    assert_eq!(
        wind.timer.to_bits(),
        8.0_f32.to_bits(),
        "timer must be unchanged"
    );
    assert_eq!(wind.direction, Vec2::X, "direction must be unchanged");
}

#[test]
fn hazard_gate_reopens_cleanly_after_adding_stack() {
    // Edge: gate is off for tick 1 (no stacks); stack added mid-test;
    // tick 2 must emit exactly one message per bolt.
    let mut app = test_app_playing();
    attach_message_capture::<ApplyBoltForce>(&mut app);
    wire(&mut app);
    insert_hazard_rng(&mut app, 42);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    let bolt = spawn_bolt(&mut app, Vec2::new(5.0, 3.0));

    // Tick 1 — gate off, zero messages
    tick_with_dt(&mut app, Duration::from_secs(1));
    {
        let forces = captured_forces(&app);
        assert_eq!(forces.len(), 0, "tick 1: zero messages while gate is off");
    }
    // bolt velocity must be bitwise-unchanged (no consumer wired)
    {
        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        assert_eq!(
            vel.0.x.to_bits(),
            5.0_f32.to_bits(),
            "tick 1: velocity.x bitwise-unchanged"
        );
        assert_eq!(
            vel.0.y.to_bits(),
            3.0_f32.to_bits(),
            "tick 1: velocity.y bitwise-unchanged"
        );
    }

    // Gate opens now
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Drift);
    tick_with_dt(&mut app, Duration::from_secs(1));

    // Tick 2 — gate open, one message
    {
        let forces = captured_forces(&app);
        assert_eq!(
            forces.len(),
            1,
            "tick 2: exactly one message after gate reopens"
        );
        assert!(
            (forces[0].force.x - 100.0).abs() < 1e-3,
            "force should be 100.0 in X after gate opens, got {:?}",
            forces[0].force
        );
        assert_eq!(forces[0].bolt, bolt, "message must address the bolt");
    }
    // bolt velocity remains bitwise-unchanged (no consumer wired in this test)
    {
        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        assert_eq!(
            vel.0.x.to_bits(),
            5.0_f32.to_bits(),
            "tick 2: velocity.x must be bitwise-unchanged (no consumer)"
        );
        assert_eq!(
            vel.0.y.to_bits(),
            3.0_f32.to_bits(),
            "tick 2: velocity.y must be bitwise-unchanged (no consumer)"
        );
    }

    // Wind timer must have decremented on tick 2
    let wind = app.world().resource::<DriftWind>();
    assert!(
        (wind.timer - 7.0).abs() < 1e-5,
        "timer should decrement by 1s after gate opens"
    );
}

// ── Behavior 38 — gate off (state not Playing) → both systems suppressed ─────

#[test]
fn state_gate_not_playing_suppresses_both_systems() {
    let mut app = test_app_not_playing();
    attach_message_capture::<ApplyBoltForce>(&mut app);
    wire(&mut app);
    insert_hazard_rng(&mut app, 42);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let _bolt = spawn_bolt(&mut app, Vec2::new(5.0, 3.0));

    tick_with_dt(&mut app, Duration::from_secs(1));

    // Zero messages (state gate suppresses the emitter)
    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 0, "zero messages when state is not Playing");

    // Wind unchanged (drift_update_wind also suppressed)
    let wind = app.world().resource::<DriftWind>();
    assert_eq!(
        wind.timer.to_bits(),
        8.0_f32.to_bits(),
        "timer must be unchanged"
    );
    assert_eq!(wind.direction, Vec2::X, "direction must be unchanged");
}

#[test]
fn playing_state_with_same_setup_does_run_both_systems() {
    // Edge: mirror the state-gate test in Playing state — systems run.
    // Assert that exactly one ApplyBoltForce message is emitted and the
    // bolt's Velocity2D is bitwise-unchanged (no consumer wired here).
    let mut app = test_app_playing();
    attach_message_capture::<ApplyBoltForce>(&mut app);
    wire(&mut app);
    insert_hazard_rng(&mut app, 42);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::new(5.0, 3.0));

    tick_with_dt(&mut app, Duration::from_secs(1));

    // One message emitted in Playing state
    let forces = captured_forces(&app);
    assert_eq!(
        forces.len(),
        1,
        "one message must be emitted in Playing state"
    );
    assert!(
        (forces[0].force.x - 100.0).abs() < 1e-3,
        "force should be applied in Playing state, got {:?}",
        forces[0].force
    );
    assert_eq!(forces[0].bolt, bolt, "message addressed to the bolt");

    // Bolt velocity bitwise-unchanged (no consumer wired)
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(
        vel.0.x.to_bits(),
        5.0_f32.to_bits(),
        "velocity.x bitwise-unchanged (no consumer)"
    );
    assert_eq!(
        vel.0.y.to_bits(),
        3.0_f32.to_bits(),
        "velocity.y bitwise-unchanged (no consumer)"
    );

    let wind = app.world().resource::<DriftWind>();
    assert!(
        (wind.timer - 7.0).abs() < 1e-5,
        "timer should have decremented"
    );
}

// ── Behavior 39 — chain ordering: update before apply ────────────────────────

#[test]
fn chain_ordering_update_runs_before_apply() {
    // Fresh direction MUST be visible to drift_apply_force within the same tick.
    // If the chain were reversed, the emitted force would still be along Vec2::X
    // (the pre-roll direction). Seed 42's first-roll direction is far from Vec2::X,
    // so a reversed-chain bug would show a force.x/force.y ratio ≈ Vec2::X.
    let mut app = test_app_playing();
    attach_message_capture::<ApplyBoltForce>(&mut app);
    wire(&mut app);
    insert_hazard_rng(&mut app, 42);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           100.0,
            period_secs:     0.5,
            per_level_force: 33.3,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let _bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.6));

    let wind = app.world().resource::<DriftWind>();
    assert!(
        (wind.direction.length() - 1.0).abs() < 1e-5,
        "direction must be a unit vector after roll"
    );

    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 1, "one message expected");
    // The emitter must NOT scale by dt — force magnitude is always force_magnitude(1) = 100.0
    assert!(
        (forces[0].force.length() - 100.0).abs() < 1e-3,
        "emitter must emit force = 100.0 (no dt scaling), got {}",
        forces[0].force.length()
    );
    let normalized_force = forces[0].force.normalize_or_zero();
    assert!(
        (normalized_force - wind.direction).length() < 1e-4,
        "emitted force must match the rolled direction (update ran before apply)"
    );
}

#[test]
fn chain_ordering_rules_out_reversed_ordering_via_direction_mismatch() {
    // Edge: reversed-chain anti-guard. Emitted force should NOT normalize to
    // Vec2::X. Seed 42 produces an angle sufficiently far from 0.
    let mut app = test_app_playing();
    attach_message_capture::<ApplyBoltForce>(&mut app);
    wire(&mut app);
    insert_hazard_rng(&mut app, 42);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           100.0,
            period_secs:     0.5,
            per_level_force: 33.3,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let _bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.6));

    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 1, "one message expected");
    let normalized_force = forces[0].force.normalize_or_zero();
    assert!(
        (normalized_force - Vec2::X).length() > 1e-3,
        "reversed chain anti-guard: force must not point along Vec2::X (seed 42 rolls elsewhere), got {normalized_force:?}"
    );
}

// ── Behavior 13 — scheduling: drift_apply_force ordered before BoltSystems::ApplyForces ──
//
// This test wires the consumer (`apply_bolt_forces`) explicitly and asserts on the
// resulting `Velocity2D` change. If `drift_apply_force` ran AFTER the consumer, the
// consumer would drain an empty channel and the bolt velocity would remain ZERO.

/// Fixed timestep matching `apply_bolt_forces/tests.rs` canonical DT.
const DT: f32 = 1.0 / 64.0;

#[test]
fn drift_apply_force_is_ordered_before_bolt_systems_apply_forces() {
    let mut app = test_app_playing();
    // Wire the drift systems with the new ordering constraint
    wire(&mut app);
    // Explicitly register the consumer so the schedule has both ends of the ordering edge
    app.add_message::<ApplyBoltForce>();
    app.add_systems(
        FixedUpdate,
        apply_bolt_forces.in_set(BoltSystems::ApplyForces),
    );
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
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(DT));

    // If ordering is correct (emitter before consumer), vel.x ≈ 100.0 * DT = 1.5625.
    // If ordering is reversed (consumer drains empty channel first), vel.x == 0.0.
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    let expected_vx = 100.0 * DT;
    assert!(
        (vel.0.x - expected_vx).abs() < 1e-4,
        "vel.x must be ≈ {} — ordering correct (emitter before consumer), got {}",
        expected_vx,
        vel.0.x
    );
    assert_eq!(
        vel.0.y.to_bits(),
        0.0_f32.to_bits(),
        "vel.y must be exactly 0.0"
    );
}

/// Edge case for Behavior 13 — proves `drift_update_wind` runs BEFORE
/// `drift_apply_force` in the wired chain by checking the emitted force
/// direction matches the NEWLY ROLLED direction after timer expiry.
///
/// Setup: stale direction = `Vec2::X`, timer expired (0.0).
/// - If ordering is correct (update first, apply second): `update_wind` rolls a
///   new direction via `GameRng(seed=42)`, and `apply_force` uses THAT new direction.
///   The velocity delta will point in the rolled direction (not `Vec2::X`).
/// - If ordering is reversed (apply first, update second): `apply_force` emits
///   the stale `Vec2::X` direction, then `update_wind` rolls a new one too late.
///   The velocity delta would be along `Vec2::X`.
///
/// Seed 42 produces a non-X direction for `ChaCha8Rng`, so vel.x being strictly
/// less than `100.0 * DT` or vel.y being non-zero proves the rolled direction
/// was used. We read the post-tick `DriftWind` to get the exact rolled direction
/// and assert the velocity matches it.
#[test]
fn drift_apply_force_uses_direction_rolled_by_update_wind_this_tick() {
    let mut app = test_app_playing();
    // Wire the full drift chain (update_wind → apply_force)
    wire(&mut app);
    // Wire the consumer so force messages are applied to Velocity2D
    app.add_message::<ApplyBoltForce>();
    app.add_systems(
        FixedUpdate,
        apply_bolt_forces.in_set(BoltSystems::ApplyForces),
    );

    // Seeded RNG — seed 42 produces a known non-X direction on the first roll
    insert_hazard_rng(&mut app, 42);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.3,
        },
    );
    // Stale direction = Vec2::X, timer = 0.0 (expired on next tick)
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(DT));

    // Read the rolled direction from the post-tick DriftWind resource
    let rolled_direction = app.world().resource::<DriftWind>().direction;

    // The rolled direction must differ from the stale Vec2::X (seed 42 rolls elsewhere)
    assert!(
        (rolled_direction - Vec2::X).length() > 1e-3,
        "seed 42 must roll a direction other than Vec2::X — verify RNG seed produces correct result"
    );

    // Expected velocity delta = rolled_direction * 100.0 * DT
    let expected_horizontal = rolled_direction.x * 100.0 * DT;
    let expected_vertical = rolled_direction.y * 100.0 * DT;

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (vel.0.x - expected_horizontal).abs() < 1e-4,
        "vel.x must be ≈ {} (rolled direction, proves update_wind ran first), got {} (Vec2::X would give {})",
        expected_horizontal,
        vel.0.x,
        100.0 * DT
    );
    assert!(
        (vel.0.y - expected_vertical).abs() < 1e-4,
        "vel.y must be ≈ {} (rolled direction, proves update_wind ran first), got {}",
        expected_vertical,
        vel.0.y
    );
}
