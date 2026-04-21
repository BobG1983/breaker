//! Group B — `erosion_restore` (bump-grade restore matrix).
//!
//! Wires only `erosion_restore` in `FixedUpdate` + registers
//! `BumpPerformed`. Pins restore behavior across all three `BumpGrade`
//! variants, full-width clamp, already-full no-op, multi-bump-per-frame
//! semantics, `restore_* = 0.0` guards, and over-unity restore clamping.

use std::time::Duration;

use bevy::prelude::*;

use super::{
    super::system::{ErosionConfig, ErosionState, erosion_restore},
    helpers::{
        canonical_config, install_erosion_config, install_erosion_state, test_app_playing,
        tick_with_dt, wire_restore_only,
    },
};
use crate::{
    breaker::messages::{BumpGrade, BumpPerformed},
    hazard::{definition::HazardKind, resources::ActiveHazards},
};

fn write_bump(app: &mut App, grade: BumpGrade) {
    app.world_mut().write_message(BumpPerformed {
        grade,
        bolt: None,
        breaker: Entity::PLACEHOLDER,
    });
}

// ── Preserved ──────────────────────────────────────────────────────────

#[test]
fn perfect_bump_restores_half_of_lost_width() {
    let mut app = test_app_playing();
    app.add_message::<BumpPerformed>();
    app.add_systems(FixedUpdate, erosion_restore);
    app.world_mut().insert_resource(ErosionConfig {
        shrink_rate:      0.05,
        min_width_frac:   0.35,
        restore_nonwhiff: 0.25,
        restore_perfect:  0.50,
    });
    app.world_mut().insert_resource(ErosionState {
        width_fraction: 0.60,
    });
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Erosion);

    write_bump(&mut app, BumpGrade::Perfect);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let state = app.world().resource::<ErosionState>();
    // lost = 0.40, restore = 0.40 * 0.50 = 0.20, width = 0.80
    assert!(
        (state.width_fraction - 0.80).abs() < 1e-5,
        "perfect bump from 0.60 with 50% restore → 0.80, got {}",
        state.width_fraction
    );
}

#[test]
fn early_bump_restores_quarter_of_lost_width() {
    let mut app = test_app_playing();
    app.add_message::<BumpPerformed>();
    app.add_systems(FixedUpdate, erosion_restore);
    app.world_mut().insert_resource(ErosionConfig {
        shrink_rate:      0.05,
        min_width_frac:   0.35,
        restore_nonwhiff: 0.25,
        restore_perfect:  0.50,
    });
    app.world_mut().insert_resource(ErosionState {
        width_fraction: 0.60,
    });
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Erosion);

    write_bump(&mut app, BumpGrade::Early);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let state = app.world().resource::<ErosionState>();
    // lost = 0.40, restore = 0.40 * 0.25 = 0.10, width = 0.70
    assert!((state.width_fraction - 0.70).abs() < 1e-5);
}

#[test]
fn late_bump_uses_nonwhiff_fraction() {
    let mut app = test_app_playing();
    app.add_message::<BumpPerformed>();
    app.add_systems(FixedUpdate, erosion_restore);
    app.world_mut().insert_resource(ErosionConfig {
        shrink_rate:      0.05,
        min_width_frac:   0.35,
        restore_nonwhiff: 0.25,
        restore_perfect:  0.50,
    });
    app.world_mut().insert_resource(ErosionState {
        width_fraction: 0.60,
    });
    // Retrofit: reader system enforces gate in-body via reader.clear().
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Erosion);

    write_bump(&mut app, BumpGrade::Late);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let state = app.world().resource::<ErosionState>();
    assert!((state.width_fraction - 0.70).abs() < 1e-5);
}

#[test]
fn restore_clamps_to_full_width() {
    let mut app = test_app_playing();
    app.add_message::<BumpPerformed>();
    app.add_systems(FixedUpdate, erosion_restore);
    app.world_mut().insert_resource(ErosionConfig {
        shrink_rate:      0.05,
        min_width_frac:   0.35,
        restore_nonwhiff: 0.25,
        restore_perfect:  1.0, // 100% → restore to full
    });
    app.world_mut().insert_resource(ErosionState {
        width_fraction: 0.50,
    });
    // Retrofit: reader system enforces gate in-body via reader.clear().
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Erosion);

    write_bump(&mut app, BumpGrade::Perfect);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let state = app.world().resource::<ErosionState>();
    assert!((state.width_fraction - 1.0).abs() < f32::EPSILON);
}

#[test]
fn no_restore_when_already_at_full_width() {
    let mut app = test_app_playing();
    app.add_message::<BumpPerformed>();
    app.add_systems(FixedUpdate, erosion_restore);
    app.world_mut().insert_resource(ErosionConfig {
        shrink_rate:      0.05,
        min_width_frac:   0.35,
        restore_nonwhiff: 0.25,
        restore_perfect:  0.50,
    });
    app.world_mut().insert_resource(ErosionState {
        width_fraction: 1.0,
    });
    // Retrofit: reader system enforces gate in-body via reader.clear().
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Erosion);

    write_bump(&mut app, BumpGrade::Perfect);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let state = app.world().resource::<ErosionState>();
    assert!((state.width_fraction - 1.0).abs() < f32::EPSILON);
}

// ── B10 — Perfect bump from 0.80 restores 0.10; chain pin via 2nd bump ─

#[test]
fn perfect_bump_from_0_80_restores_0_10_and_chains_to_0_95() {
    let mut app = test_app_playing();
    wire_restore_only(&mut app);
    install_erosion_config(&mut app, canonical_config());
    install_erosion_state(&mut app, 0.80);

    write_bump(&mut app, BumpGrade::Perfect);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    {
        let state = app.world().resource::<ErosionState>();
        // lost 0.20 × 0.50 = 0.10 → 0.90
        assert!(
            (state.width_fraction - 0.90).abs() < 1e-5,
            "expected 0.90, got {}",
            state.width_fraction
        );
    }

    // Edge: second Perfect bump immediately after — lost 0.10 × 0.50 = 0.05 → 0.95.
    write_bump(&mut app, BumpGrade::Perfect);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let state = app.world().resource::<ErosionState>();
    assert!(
        (state.width_fraction - 0.95).abs() < 1e-5,
        "expected 0.95, got {}",
        state.width_fraction
    );
}

// ── B11 — Perfect bump from exactly min_width_frac; chains with Early ──

#[test]
fn perfect_bump_from_min_floor_restores_half_then_early_chains() {
    let mut app = test_app_playing();
    wire_restore_only(&mut app);
    install_erosion_config(&mut app, canonical_config());
    install_erosion_state(&mut app, 0.35);

    write_bump(&mut app, BumpGrade::Perfect);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    {
        let state = app.world().resource::<ErosionState>();
        // lost 0.65 × 0.50 = 0.325 → 0.675
        assert!(
            (state.width_fraction - 0.675).abs() < 1e-5,
            "expected 0.675, got {}",
            state.width_fraction
        );
    }

    // Edge: follow with Early — lost 0.325 × 0.25 = 0.08125 → 0.75625.
    write_bump(&mut app, BumpGrade::Early);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let state = app.world().resource::<ErosionState>();
    assert!(
        (state.width_fraction - 0.75625).abs() < 1e-5,
        "expected 0.75625, got {}",
        state.width_fraction
    );
}

// ── B12 — Early bump at non-full width ─────────────────────────────────

#[test]
fn early_bump_at_non_full_width_restores_exactly_nonwhiff_times_lost() {
    let mut app = test_app_playing();
    wire_restore_only(&mut app);
    install_erosion_config(&mut app, canonical_config());
    install_erosion_state(&mut app, 0.50);

    write_bump(&mut app, BumpGrade::Early);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    {
        let state = app.world().resource::<ErosionState>();
        // lost 0.50, restore 0.50 * 0.25 = 0.125, width = 0.625
        assert!((state.width_fraction - 0.625).abs() < 1e-5);
    }

    // Edge: `restore_nonwhiff = 0.40` from state 0.50 → width ≈ 0.70.
    let mut app = test_app_playing();
    wire_restore_only(&mut app);
    install_erosion_config(
        &mut app,
        ErosionConfig {
            shrink_rate:      0.05,
            min_width_frac:   0.35,
            restore_nonwhiff: 0.40,
            restore_perfect:  0.50,
        },
    );
    install_erosion_state(&mut app, 0.50);

    write_bump(&mut app, BumpGrade::Early);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let state = app.world().resource::<ErosionState>();
    // lost 0.50, restore 0.50 * 0.40 = 0.20, width = 0.70
    assert!((state.width_fraction - 0.70).abs() < 1e-5);
}

// ── B13 — two Perfect bumps in same frame process sequentially ─────────

#[test]
fn multiple_perfect_bumps_in_same_frame_process_sequentially() {
    let mut app = test_app_playing();
    wire_restore_only(&mut app);
    install_erosion_config(&mut app, canonical_config());
    install_erosion_state(&mut app, 0.60);

    write_bump(&mut app, BumpGrade::Perfect);
    write_bump(&mut app, BumpGrade::Perfect);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    {
        let state = app.world().resource::<ErosionState>();
        // first: lost 0.40 × 0.50 = 0.20 → 0.80. second: lost 0.20 × 0.50 = 0.10 → 0.90.
        assert!((state.width_fraction - 0.90).abs() < 1e-5);
    }

    // Edge: THREE Perfect bumps in one tick from fresh state 0.60.
    let mut app = test_app_playing();
    wire_restore_only(&mut app);
    install_erosion_config(&mut app, canonical_config());
    install_erosion_state(&mut app, 0.60);
    for _ in 0..3 {
        write_bump(&mut app, BumpGrade::Perfect);
    }
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let state = app.world().resource::<ErosionState>();
    // 0.60 → 0.80 → 0.90 → 0.95
    assert!((state.width_fraction - 0.95).abs() < 1e-5);
}

// ── B14a — two-tick per-tick intermediate state pin ────────────────────

#[test]
fn first_bump_perfect_then_early_emits_in_order() {
    let mut app = test_app_playing();
    wire_restore_only(&mut app);
    install_erosion_config(&mut app, canonical_config());
    install_erosion_state(&mut app, 0.80);

    // Phase 1 — Perfect bump only.
    write_bump(&mut app, BumpGrade::Perfect);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    {
        let state = app.world().resource::<ErosionState>();
        // lost 0.20 × 0.50 = 0.10 → 0.90
        assert!(
            (state.width_fraction - 0.90).abs() < 1e-5,
            "phase 1 expected 0.90, got {}",
            state.width_fraction
        );
    }

    // Phase 2 — Early bump chains off the restored 0.90.
    write_bump(&mut app, BumpGrade::Early);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let state = app.world().resource::<ErosionState>();
    // new lost 0.10 × 0.25 = 0.025 → 0.925.
    assert!(
        (state.width_fraction - 0.925).abs() < 1e-5,
        "phase 2 expected 0.925, got {}",
        state.width_fraction
    );
}

#[test]
fn first_bump_early_then_perfect_reverses_two_tick_chain() {
    // Edge of B14a — reversed: Early first, Perfect second.
    let mut app = test_app_playing();
    wire_restore_only(&mut app);
    install_erosion_config(&mut app, canonical_config());
    install_erosion_state(&mut app, 0.80);

    write_bump(&mut app, BumpGrade::Early);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    {
        let state = app.world().resource::<ErosionState>();
        // lost 0.20 × 0.25 = 0.05 → 0.85.
        assert!((state.width_fraction - 0.85).abs() < 1e-5);
    }

    write_bump(&mut app, BumpGrade::Perfect);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let state = app.world().resource::<ErosionState>();
    // lost 0.15 × 0.50 = 0.075 → 0.925. Not 0.95 (sum-deltas) — proves state-aware chain.
    assert!(
        (state.width_fraction - 0.925).abs() < 1e-5,
        "phase 2 expected 0.925, got {}",
        state.width_fraction
    );
}

// ── B14b — single-tick insertion-order pin ─────────────────────────────

#[test]
fn two_perfects_in_one_tick_chain_correctly() {
    let mut app = test_app_playing();
    wire_restore_only(&mut app);
    install_erosion_config(&mut app, canonical_config());
    install_erosion_state(&mut app, 0.60);

    write_bump(&mut app, BumpGrade::Perfect);
    write_bump(&mut app, BumpGrade::Perfect);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let state = app.world().resource::<ErosionState>();
    // Chain: 0.60 → 0.80 → 0.90. Not 0.80 (best-grade only). Not 1.00 (sum deltas).
    assert!(
        (state.width_fraction - 0.90).abs() < 1e-5,
        "expected 0.90, got {}",
        state.width_fraction
    );
}

// ── B15 — restore_perfect == 0.0 silences only Perfect bumps ──────────

#[test]
fn zero_restore_perfect_silences_perfect_bumps_but_not_early() {
    let mut app = test_app_playing();
    wire_restore_only(&mut app);
    install_erosion_config(
        &mut app,
        ErosionConfig {
            shrink_rate:      0.05,
            min_width_frac:   0.35,
            restore_nonwhiff: 0.25,
            restore_perfect:  0.0,
        },
    );
    install_erosion_state(&mut app, 0.60);

    write_bump(&mut app, BumpGrade::Perfect);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    {
        let state = app.world().resource::<ErosionState>();
        assert!(
            (state.width_fraction - 0.60).abs() < f32::EPSILON,
            "width_fraction should be 0.60"
        );
    }

    // Edge: Early still restores via `restore_nonwhiff = 0.25`.
    write_bump(&mut app, BumpGrade::Early);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let state = app.world().resource::<ErosionState>();
    // lost 0.40 × 0.25 = 0.10 → 0.70.
    assert!(
        (state.width_fraction - 0.70).abs() < 1e-5,
        "expected 0.70, got {}",
        state.width_fraction
    );
}

// ── B16 — restore_nonwhiff == 0.0 silences Early+Late bumps ────────────

#[test]
fn zero_restore_nonwhiff_silences_early_and_late_bumps() {
    let mut app = test_app_playing();
    wire_restore_only(&mut app);
    install_erosion_config(
        &mut app,
        ErosionConfig {
            shrink_rate:      0.05,
            min_width_frac:   0.35,
            restore_nonwhiff: 0.0,
            restore_perfect:  0.50,
        },
    );
    install_erosion_state(&mut app, 0.60);

    write_bump(&mut app, BumpGrade::Early);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    {
        let state = app.world().resource::<ErosionState>();
        assert!(
            (state.width_fraction - 0.60).abs() < f32::EPSILON,
            "width_fraction should be 0.60"
        );
    }

    // Edge: Late is gated by the same `restore_nonwhiff`.
    write_bump(&mut app, BumpGrade::Late);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let state = app.world().resource::<ErosionState>();
    assert!(
        (state.width_fraction - 0.60).abs() < f32::EPSILON,
        "width_fraction should be 0.60"
    );
}

// ── B17 — restore fraction > 1.0 is clamped to lost ───────────────────

#[test]
fn over_unity_restore_perfect_is_clamped_by_min_lost() {
    let mut app = test_app_playing();
    wire_restore_only(&mut app);
    install_erosion_config(
        &mut app,
        ErosionConfig {
            shrink_rate:      0.05,
            min_width_frac:   0.35,
            restore_nonwhiff: 0.25,
            restore_perfect:  5.0,
        },
    );
    install_erosion_state(&mut app, 0.60);

    write_bump(&mut app, BumpGrade::Perfect);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    {
        let state = app.world().resource::<ErosionState>();
        // computed restore 0.40 × 5.0 = 2.0, capped by .min(lost = 0.40) → 0.40;
        // (0.60 + 0.40).min(1.0) = 1.0.
        assert!((state.width_fraction - 1.0).abs() < f32::EPSILON);
    }

    // Edge: from a fresh state of 0.80 — restore = min(0.20 * 5.0, 0.20) = 0.20 → 1.0.
    install_erosion_state(&mut app, 0.80);
    write_bump(&mut app, BumpGrade::Perfect);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let state = app.world().resource::<ErosionState>();
    assert!((state.width_fraction - 1.0).abs() < f32::EPSILON);
}

// ── B18 — no bump message means no state change ───────────────────────

#[test]
fn no_bump_message_means_no_state_change_across_two_ticks() {
    let mut app = test_app_playing();
    wire_restore_only(&mut app);
    install_erosion_config(&mut app, canonical_config());
    install_erosion_state(&mut app, 0.60);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    {
        let state = app.world().resource::<ErosionState>();
        assert!(
            (state.width_fraction - 0.60).abs() < f32::EPSILON,
            "width_fraction should be 0.60"
        );
    }

    // Edge: two consecutive empty ticks.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    let state = app.world().resource::<ErosionState>();
    assert!(
        (state.width_fraction - 0.60).abs() < f32::EPSILON,
        "width_fraction should be 0.60"
    );
}

// ── B19 — no-config guard ──────────────────────────────────────────────

#[test]
fn no_config_guard_drains_reader_leaves_state_and_runs_cleanly_without_state() {
    // `ErosionConfig` absent → reader.clear() fires, state untouched.
    let mut app = test_app_playing();
    wire_restore_only(&mut app);
    install_erosion_state(&mut app, 0.60);
    // Intentionally no install_erosion_config.

    write_bump(&mut app, BumpGrade::Perfect);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    {
        let state = app.world().resource::<ErosionState>();
        assert!(
            (state.width_fraction - 0.60).abs() < f32::EPSILON,
            "width_fraction should be 0.60"
        );
    }

    // Second tick with no further messages — still 0.60.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    {
        let state = app.world().resource::<ErosionState>();
        assert!(
            (state.width_fraction - 0.60).abs() < f32::EPSILON,
            "width_fraction should be 0.60"
        );
    }

    // Edge: fresh app with both ErosionConfig AND ErosionState absent — no panic.
    let mut app = test_app_playing();
    wire_restore_only(&mut app);
    // Neither resource inserted.

    write_bump(&mut app, BumpGrade::Perfect);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(app.world().get_resource::<ErosionConfig>().is_none());
    assert!(app.world().get_resource::<ErosionState>().is_none());
}
