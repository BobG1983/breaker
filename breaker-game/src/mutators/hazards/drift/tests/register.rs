//! Group D — `register` integration: full chain, ordering, gating.
//!
//! These tests exercise the production wiring (`register(&mut app)`) so the
//! run conditions `hazard_active(Drift)` and `in_state(NodeState::Playing)`
//! are active, and the chain ordering
//! `(drift_update_wind, drift_apply_force).chain()` is enforced.

use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::{
    super::system::{DriftConfig, DriftWind, register},
    helpers::{
        add_drift_stacks, canonical_config, insert_rng, install_drift_config, install_drift_wind,
        spawn_bolt, test_app_not_playing, test_app_playing, tick_with_dt,
    },
};
use crate::mutators::hazards::{definition::HazardKind, resources::ActiveHazards};

// ── Behavior 34 — full chain: one tick updates wind AND applies force ────

#[test]
fn full_chain_updates_wind_and_applies_force_single_tick() {
    let mut app = test_app_playing();
    register(&mut app);
    insert_rng(&mut app, 42);
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

    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.x - 100.0).abs() < 1e-3,
        "force applied in X, got {:?}",
        velocity.0
    );
    let wind = app.world().resource::<DriftWind>();
    assert!(
        (wind.timer - 7.0).abs() < 1e-5,
        "timer should decrement by 1s, got {}",
        wind.timer
    );
}

#[test]
fn full_chain_accumulates_across_two_ticks() {
    // Edge: 2 ticks of 1s — X should be ≈200, timer should be ≈6.0.
    let mut app = test_app_playing();
    register(&mut app);
    insert_rng(&mut app, 42);
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

    tick_with_dt(&mut app, Duration::from_secs(1));
    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!((velocity.0.x - 200.0).abs() < 1e-3);
    let wind = app.world().resource::<DriftWind>();
    assert!((wind.timer - 6.0).abs() < 1e-5);
}

// ── Behavior 35 — timer expiry → fresh roll applied same-tick ────────────

#[test]
fn first_tick_with_expired_timer_rolls_and_applies_new_direction() {
    // Timer expires → fresh direction is rolled BEFORE force is applied
    // (chain order: update → apply). The force magnitude (100 * 1s) is
    // exactly 100 regardless of which direction was rolled.
    let mut app = test_app_playing();
    register(&mut app);
    insert_rng(&mut app, 42);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);

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

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.length() - 100.0).abs() < 1e-3,
        "bolt speed should equal force*dt=100, got len={}",
        velocity.0.length()
    );
    let normalized = velocity.0.normalize_or_zero();
    assert!(
        (normalized - wind.direction).length() < 1e-4,
        "bolt velocity must point in the rolled direction, not Vec2::X"
    );
}

#[test]
fn first_tick_rolled_direction_rules_out_reversed_ordering() {
    // Edge: anti-ordering guard. If the chain were reversed, the bolt
    // would be accelerated along the PRE-roll direction (Vec2::X) and
    // then the direction would roll. The speed is still 100, but the
    // bolt's velocity would normalize to Vec2::X — which seed 42 does
    // NOT produce. Assert length > 50 (guarding against zero velocity).
    let mut app = test_app_playing();
    register(&mut app);
    insert_rng(&mut app, 42);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        velocity.0.length() > 50.0,
        "velocity magnitude must be positive and non-trivial"
    );
}

// ── Behavior 36 — multi-tick: direction holds across ticks, timer > 0 ────

#[test]
fn multi_tick_direction_holds_while_timer_positive() {
    let mut app = test_app_playing();
    register(&mut app);
    insert_rng(&mut app, 42);
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

    for _ in 0..4 {
        tick_with_dt(&mut app, Duration::from_secs(1));
    }

    let wind = app.world().resource::<DriftWind>();
    assert_eq!(
        wind.direction,
        Vec2::X,
        "direction should be bitwise unchanged while timer > 0"
    );
    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.x - 400.0).abs() < 1e-3,
        "force × 4s, got {}",
        velocity.0.x
    );
    assert!(velocity.0.y.abs() < 1e-5);
}

#[test]
fn fifth_tick_crosses_threshold_and_rolls_new_direction() {
    // Edge: after 4 ticks the timer is 4.0; a fifth tick at dt=5s rolls.
    // The bolt's pre-existing velocity is (400, 0). The new impulse is
    // direction * 500 in some rolled direction. Magnitude should be
    // between |500 - 400| = 100 and 500 + 400 = 900.
    let mut app = test_app_playing();
    register(&mut app);
    insert_rng(&mut app, 42);
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

    for _ in 0..4 {
        tick_with_dt(&mut app, Duration::from_secs(1));
    }
    tick_with_dt(&mut app, Duration::from_secs(5));

    let wind = app.world().resource::<DriftWind>();
    assert!((wind.direction.length() - 1.0).abs() < 1e-5);
    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    let magnitude = velocity.0.length();
    assert!(
        magnitude > 100.0 && magnitude < 900.0,
        "magnitude should be in |prior ± impulse| bounds, got {magnitude}"
    );
}

// ── Behavior 37 — gate off (hazard inactive) → both systems suppressed ───

#[test]
fn hazard_inactive_gate_suppresses_both_systems() {
    let mut app = test_app_playing();
    register(&mut app);
    insert_rng(&mut app, 42);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    // No add_drift_stacks → stacks(Drift) == 0 → hazard_active is false.
    let bolt = spawn_bolt(&mut app, Vec2::new(5.0, 3.0));

    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(velocity.0.x.to_bits(), 5.0_f32.to_bits());
    assert_eq!(velocity.0.y.to_bits(), 3.0_f32.to_bits());
    let wind = app.world().resource::<DriftWind>();
    assert_eq!(wind.timer.to_bits(), 8.0_f32.to_bits());
    assert_eq!(wind.direction, Vec2::X);
}

#[test]
fn hazard_gate_reopens_cleanly_after_adding_stack() {
    // Edge: add 1 stack mid-way → next tick opens the gate.
    let mut app = test_app_playing();
    register(&mut app);
    insert_rng(&mut app, 42);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    let bolt = spawn_bolt(&mut app, Vec2::new(5.0, 3.0));

    tick_with_dt(&mut app, Duration::from_secs(1));

    // Gate opens now.
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Drift);
    tick_with_dt(&mut app, Duration::from_secs(1));

    let wind = app.world().resource::<DriftWind>();
    assert!(
        (wind.timer - 7.0).abs() < 1e-5,
        "timer should decrement by 1s after gate opens"
    );
    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.x - 105.0).abs() < 1e-3,
        "bolt should gain 100 in X after gate opens, got {}",
        velocity.0.x
    );
}

// ── Behavior 38 — gate off (state not Playing) → both systems suppressed ─

#[test]
fn state_gate_not_playing_suppresses_both_systems() {
    let mut app = test_app_not_playing();
    register(&mut app);
    insert_rng(&mut app, 42);
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

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(velocity.0.x.to_bits(), 5.0_f32.to_bits());
    assert_eq!(velocity.0.y.to_bits(), 3.0_f32.to_bits());
    let wind = app.world().resource::<DriftWind>();
    assert_eq!(wind.timer.to_bits(), 8.0_f32.to_bits());
    assert_eq!(wind.direction, Vec2::X);
}

#[test]
fn playing_state_with_same_setup_does_run_both_systems() {
    // Edge: mirror the state-gate test in Playing state — systems run.
    let mut app = test_app_playing();
    register(&mut app);
    insert_rng(&mut app, 42);
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

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.x - 105.0).abs() < 1e-3,
        "force should be applied in Playing state, got {}",
        velocity.0.x
    );
    let wind = app.world().resource::<DriftWind>();
    assert!((wind.timer - 7.0).abs() < 1e-5);
}

// ── Behavior 39 — chain ordering: update before apply (regression) ───────

#[test]
fn chain_ordering_update_runs_before_apply() {
    // Fresh direction MUST be visible to drift_apply_force within the
    // same tick. If the chain were reversed, bolt velocity would still
    // be along Vec2::X (the pre-roll direction). Seed 42's first-roll
    // direction is far from Vec2::X, so a reversed-chain bug would show
    // a normalized velocity ≈ Vec2::X.
    let mut app = test_app_playing();
    register(&mut app);
    insert_rng(&mut app, 42);
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
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.6));

    let wind = app.world().resource::<DriftWind>();
    assert!((wind.direction.length() - 1.0).abs() < 1e-5);

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.length() - 60.0).abs() < 1e-3,
        "magnitude = force*dt = 100*0.6 = 60, got {}",
        velocity.0.length()
    );
    let normalized = velocity.0.normalize_or_zero();
    assert!(
        (normalized - wind.direction).length() < 1e-4,
        "velocity must match the rolled direction (update ran before apply)"
    );
}

#[test]
fn chain_ordering_rules_out_reversed_ordering_via_direction_mismatch() {
    // Edge: reversed-chain anti-guard. Velocity should NOT normalize to
    // Vec2::X. Seed 42 produces an angle sufficiently far from 0.
    let mut app = test_app_playing();
    register(&mut app);
    insert_rng(&mut app, 42);
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
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.6));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    let normalized = velocity.0.normalize_or_zero();
    assert!(
        (normalized - Vec2::X).length() > 1e-3,
        "reversed chain anti-guard: velocity must not point along Vec2::X, got {normalized:?}"
    );
}
