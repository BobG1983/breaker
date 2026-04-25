//! Group A — `erosion_shrink` (shrink formula pinning).
//!
//! Wires only `erosion_shrink` in `FixedUpdate`. Pins the formula
//! `delta = shrink_rate * stacks * dt` and the clamp-to-`min_width_frac`
//! floor across stack counts, tick counts, dt values, and parameter edges.

use std::time::Duration;

use bevy::prelude::*;

use super::{
    super::system::{ErosionConfig, ErosionState, erosion_shrink},
    helpers::{
        add_erosion_stacks, canonical_config, install_erosion_config, install_erosion_state,
        test_app_playing, tick_with_dt, wire_shrink_only,
    },
};
use crate::mutators::hazards::{definition::HazardKind, resources::ActiveHazards};

// ── Preserved ───────────────────────────────────────────────────────────

#[test]
fn shrink_stack_one_reduces_width_fraction() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, erosion_shrink);
    app.world_mut().insert_resource(ErosionConfig {
        shrink_rate:      0.05,
        min_width_frac:   0.35,
        restore_nonwhiff: 0.25,
        restore_perfect:  0.50,
    });
    app.world_mut().insert_resource(ErosionState::default());
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Erosion);

    tick_with_dt(&mut app, Duration::from_secs(1));

    let state = app.world().resource::<ErosionState>();
    assert!(
        (state.width_fraction - 0.95).abs() < 1e-5,
        "1s at stack 1 with 0.05 rate → width 0.95, got {}",
        state.width_fraction
    );
}

#[test]
fn shrink_stack_three_scales_linearly() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, erosion_shrink);
    app.world_mut().insert_resource(ErosionConfig {
        shrink_rate:      0.05,
        min_width_frac:   0.35,
        restore_nonwhiff: 0.25,
        restore_perfect:  0.50,
    });
    app.world_mut().insert_resource(ErosionState::default());
    for _ in 0..3 {
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Erosion);
    }

    tick_with_dt(&mut app, Duration::from_secs(1));

    let state = app.world().resource::<ErosionState>();
    // 1.0 - 0.05 * 3 = 0.85
    assert!((state.width_fraction - 0.85).abs() < 1e-5);
}

#[test]
fn shrink_clamps_to_min_width_frac() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, erosion_shrink);
    app.world_mut().insert_resource(ErosionConfig {
        shrink_rate:      1.0, // huge — would go to 0 in one sec
        min_width_frac:   0.35,
        restore_nonwhiff: 0.25,
        restore_perfect:  0.50,
    });
    app.world_mut().insert_resource(ErosionState {
        width_fraction: 0.40,
    });
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Erosion);

    tick_with_dt(&mut app, Duration::from_secs(1));

    let state = app.world().resource::<ErosionState>();
    assert!((state.width_fraction - 0.35).abs() < f32::EPSILON);
}

// ── A1 — stack 0 produces no shrink ─────────────────────────────────────

#[test]
fn stack_zero_produces_no_shrink_across_multiple_ticks() {
    let mut app = test_app_playing();
    wire_shrink_only(&mut app);
    install_erosion_config(&mut app, canonical_config());
    install_erosion_state(&mut app, 1.0);
    // ActiveHazards left empty (0 Erosion stacks).

    tick_with_dt(&mut app, Duration::from_secs(1));
    {
        let state = app.world().resource::<ErosionState>();
        assert!(
            (state.width_fraction - 1.0).abs() < f32::EPSILON,
            "width_fraction should be 1.0"
        );
    }

    // Edge: two consecutive 1.0s ticks at stack 0 still yield 1.0.
    tick_with_dt(&mut app, Duration::from_secs(1));
    let state = app.world().resource::<ErosionState>();
    assert!(
        (state.width_fraction - 1.0).abs() < f32::EPSILON,
        "width_fraction should be 1.0"
    );
}

// ── A2 — stack 1 with half-second dt pins formula ──────────────────────

#[test]
fn stack_one_half_second_dt_pins_formula_and_accumulates() {
    let mut app = test_app_playing();
    wire_shrink_only(&mut app);
    install_erosion_config(&mut app, canonical_config());
    install_erosion_state(&mut app, 1.0);
    add_erosion_stacks(&mut app, 1);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.5));

    {
        let state = app.world().resource::<ErosionState>();
        // 1.0 - 0.05 * 1 * 0.5 = 0.975
        assert!(
            (state.width_fraction - 0.975).abs() < 1e-5,
            "expected 0.975, got {}",
            state.width_fraction
        );
    }

    // Edge: a second tick at the same dt accumulates to ≈ 0.950.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.5));
    let state = app.world().resource::<ErosionState>();
    assert!(
        (state.width_fraction - 0.95).abs() < 1e-5,
        "expected 0.95 after two 0.5s ticks, got {}",
        state.width_fraction
    );
}

// ── A3 — stack 2 shrinks at 2x the stack-1 rate ────────────────────────

#[test]
fn stack_two_shrinks_at_twice_stack_one_rate() {
    let mut app = test_app_playing();
    wire_shrink_only(&mut app);
    install_erosion_config(&mut app, canonical_config());
    install_erosion_state(&mut app, 1.0);
    add_erosion_stacks(&mut app, 2);

    tick_with_dt(&mut app, Duration::from_secs(1));

    {
        let state = app.world().resource::<ErosionState>();
        // 1.0 - 0.05 * 2 * 1.0 = 0.90
        assert!((state.width_fraction - 0.90).abs() < 1e-5);
    }

    // Edge: stack 2 @ dt=0.5s produces the same shrink as stack 1 @ dt=1.0s.
    install_erosion_state(&mut app, 1.0);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.5));
    let state = app.world().resource::<ErosionState>();
    // 1.0 - 0.05 * 2 * 0.5 = 0.95
    assert!((state.width_fraction - 0.95).abs() < 1e-5);
}

// ── A4 — stack 5 shrinks at 5x the stack-1 rate ────────────────────────

#[test]
fn stack_five_shrinks_at_five_times_stack_one_rate() {
    let mut app = test_app_playing();
    wire_shrink_only(&mut app);
    install_erosion_config(&mut app, canonical_config());
    install_erosion_state(&mut app, 1.0);
    add_erosion_stacks(&mut app, 5);

    tick_with_dt(&mut app, Duration::from_secs(1));
    {
        let state = app.world().resource::<ErosionState>();
        // 1.0 - 0.05 * 5 * 1.0 = 0.75
        assert!((state.width_fraction - 0.75).abs() < 1e-5);
    }

    // Edge: stack 5 @ dt=0.2s → 0.95 (same as stack 1 @ dt=1.0s).
    install_erosion_state(&mut app, 1.0);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.2));
    let state = app.world().resource::<ErosionState>();
    // 1.0 - 0.05 * 5 * 0.2 = 0.95
    assert!((state.width_fraction - 0.95).abs() < 1e-5);
}

// ── A5 — consecutive ticks accumulate linearly at stack 1 ──────────────

#[test]
fn consecutive_1s_ticks_accumulate_linearly_at_stack_one() {
    let mut app = test_app_playing();
    wire_shrink_only(&mut app);
    install_erosion_config(&mut app, canonical_config());
    install_erosion_state(&mut app, 1.0);
    add_erosion_stacks(&mut app, 1);

    for _ in 0..3 {
        tick_with_dt(&mut app, Duration::from_secs(1));
    }
    {
        let state = app.world().resource::<ErosionState>();
        // 1.0 - 0.05 * 3 = 0.85
        assert!((state.width_fraction - 0.85).abs() < 1e-5);
    }

    // Edge: continue to ten total ticks → width = 0.50.
    for _ in 0..7 {
        tick_with_dt(&mut app, Duration::from_secs(1));
    }
    let state = app.world().resource::<ErosionState>();
    assert!(
        (state.width_fraction - 0.50).abs() < 1e-5,
        "expected 0.50 after ten 1s ticks, got {}",
        state.width_fraction
    );
}

// ── A6 — removed: the intended "delta <= 0.0 guard" is exercised by
//         zero-stack tests (A8, A9) where delta is exactly 0.0; the
//         original near-zero-dt formulation was incompatible with Bevy's
//         fixed-update scheduler (sub-microsecond timesteps drain the
//         accumulator in a very large loop).

// ── No-config guard — mirrors B19/MB2 for coverage parity ──────────────

#[test]
fn shrink_without_config_or_state_is_a_noop() {
    // With NO ErosionConfig and NO ErosionState, the Option<Res<_>>
    // early-return on system.rs:96 must fire cleanly. No panic; world
    // stays empty of Erosion resources.
    let mut app = test_app_playing();
    wire_shrink_only(&mut app);
    add_erosion_stacks(&mut app, 1);
    // Deliberately do NOT install_erosion_config or install_erosion_state.

    tick_with_dt(&mut app, Duration::from_secs(1));

    assert!(app.world().get_resource::<ErosionConfig>().is_none());
    assert!(app.world().get_resource::<ErosionState>().is_none());
}

// ── A7 — shrink_rate == 0.0 produces no shrink ────────────────────────

#[test]
fn zero_shrink_rate_produces_no_shrink() {
    let mut app = test_app_playing();
    wire_shrink_only(&mut app);
    install_erosion_config(
        &mut app,
        ErosionConfig {
            shrink_rate:      0.0,
            min_width_frac:   0.35,
            restore_nonwhiff: 0.25,
            restore_perfect:  0.50,
        },
    );
    install_erosion_state(&mut app, 1.0);
    add_erosion_stacks(&mut app, 3);

    tick_with_dt(&mut app, Duration::from_secs(1));
    {
        let state = app.world().resource::<ErosionState>();
        assert!(
            (state.width_fraction - 1.0).abs() < f32::EPSILON,
            "width_fraction should be 1.0"
        );
    }

    // Edge: 10 stacks and dt=2.0s still no shrink.
    add_erosion_stacks(&mut app, 7); // now 10 total
    tick_with_dt(&mut app, Duration::from_secs_f32(2.0));
    let state = app.world().resource::<ErosionState>();
    assert!(
        (state.width_fraction - 1.0).abs() < f32::EPSILON,
        "width_fraction should be 1.0"
    );
}

// ── A8 — negative shrink_rate is treated as no-op ─────────────────────

#[test]
fn negative_shrink_rate_guard_produces_no_change() {
    let mut app = test_app_playing();
    wire_shrink_only(&mut app);
    install_erosion_config(
        &mut app,
        ErosionConfig {
            shrink_rate:      -0.05,
            min_width_frac:   0.35,
            restore_nonwhiff: 0.25,
            restore_perfect:  0.50,
        },
    );
    install_erosion_state(&mut app, 0.80);
    add_erosion_stacks(&mut app, 1);

    tick_with_dt(&mut app, Duration::from_secs(1));
    {
        let state = app.world().resource::<ErosionState>();
        // Raw delta -0.05; guard fires, state unchanged.
        assert!(
            (state.width_fraction - 0.80).abs() < f32::EPSILON,
            "width_fraction should be 0.80"
        );
    }

    // Edge: 3 stacks, dt=2.0s, raw delta -0.30 — state stays 0.80.
    add_erosion_stacks(&mut app, 2); // now 3 total
    tick_with_dt(&mut app, Duration::from_secs_f32(2.0));
    let state = app.world().resource::<ErosionState>();
    assert!(
        (state.width_fraction - 0.80).abs() < f32::EPSILON,
        "width_fraction should be 0.80"
    );
}

// ── A9 — state already at min floor stays at floor across ticks ───────

#[test]
fn state_at_min_floor_stays_at_floor_across_ten_ticks() {
    let mut app = test_app_playing();
    wire_shrink_only(&mut app);
    install_erosion_config(&mut app, canonical_config());
    install_erosion_state(&mut app, 0.35);
    add_erosion_stacks(&mut app, 3);

    for _ in 0..10 {
        tick_with_dt(&mut app, Duration::from_secs(1));
        let state = app.world().resource::<ErosionState>();
        assert!((state.width_fraction - 0.35).abs() < f32::EPSILON);
    }
}
