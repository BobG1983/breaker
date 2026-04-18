//! Group E — `activate` lifecycle.
//!
//! Fresh `TestAppBuilder::new().build()` apps with a one-shot `Update`
//! system that calls `activate(...)` via `Commands`. Pins activation
//! extraction from `HazardTuning::Erosion`, the mismatched-tuning
//! guard, and idempotency/overwrite semantics.

use bevy::prelude::*;

use super::super::system::{ErosionConfig, ErosionState, activate};
use crate::{hazard::definition::HazardTuning, prelude::*};

// ── Preserved ──────────────────────────────────────────────────────────

#[test]
fn activate_with_matching_tuning_inserts_config_and_state() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Erosion {
                shrink_rate:      0.05,
                min_width_frac:   0.35,
                restore_nonwhiff: 0.25,
                restore_perfect:  0.5,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<ErosionConfig>();
    assert!((cfg.shrink_rate - 0.05).abs() < f32::EPSILON);
    let state = app.world().resource::<ErosionState>();
    assert!((state.width_fraction - 1.0).abs() < f32::EPSILON);
}

#[test]
fn activate_with_mismatched_tuning_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Decay {
                base_percent:      0.05,
                per_level_percent: 0.03,
            },
            &mut commands,
        );
    });
    app.update();
    assert!(app.world().get_resource::<ErosionConfig>().is_none());
    assert!(app.world().get_resource::<ErosionState>().is_none());
}

// ── E32 — activate round-trips all four fields (main + alternate set) ──

#[test]
fn activate_round_trips_all_four_fields_of_erosion_tuning() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Erosion {
                shrink_rate:      0.07,
                min_width_frac:   0.40,
                restore_nonwhiff: 0.30,
                restore_perfect:  0.60,
            },
            &mut commands,
        );
    });
    app.update();

    {
        let cfg = app.world().resource::<ErosionConfig>();
        assert!((cfg.shrink_rate - 0.07).abs() < f32::EPSILON);
        assert!((cfg.min_width_frac - 0.40).abs() < f32::EPSILON);
        assert!((cfg.restore_nonwhiff - 0.30).abs() < f32::EPSILON);
        assert!((cfg.restore_perfect - 0.60).abs() < f32::EPSILON);
        let state = app.world().resource::<ErosionState>();
        assert!((state.width_fraction - 1.0).abs() < f32::EPSILON);
    }

    // Edge: a different set (0.01, 0.50, 0.10, 0.20) proves no fields swap or hardcode.
    let mut app2 = TestAppBuilder::new().build();
    app2.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Erosion {
                shrink_rate:      0.01,
                min_width_frac:   0.50,
                restore_nonwhiff: 0.10,
                restore_perfect:  0.20,
            },
            &mut commands,
        );
    });
    app2.update();

    let cfg = app2.world().resource::<ErosionConfig>();
    assert!((cfg.shrink_rate - 0.01).abs() < f32::EPSILON);
    assert!((cfg.min_width_frac - 0.50).abs() < f32::EPSILON);
    assert!((cfg.restore_nonwhiff - 0.10).abs() < f32::EPSILON);
    assert!((cfg.restore_perfect - 0.20).abs() < f32::EPSILON);
}

// ── E33 — second activate with different values OVERWRITES + resets ────

#[test]
fn second_activate_overwrites_config_and_resets_state_to_one() {
    let mut app = TestAppBuilder::new().build();
    // Pre-insert stale resources.
    app.world_mut().insert_resource(ErosionConfig {
        shrink_rate:      0.01,
        min_width_frac:   0.50,
        restore_nonwhiff: 0.10,
        restore_perfect:  0.20,
    });
    app.world_mut().insert_resource(ErosionState {
        width_fraction: 0.42,
    });
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Erosion {
                shrink_rate:      0.05,
                min_width_frac:   0.35,
                restore_nonwhiff: 0.25,
                restore_perfect:  0.50,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<ErosionConfig>();
    assert!((cfg.shrink_rate - 0.05).abs() < f32::EPSILON);
    assert!((cfg.min_width_frac - 0.35).abs() < f32::EPSILON);
    assert!((cfg.restore_nonwhiff - 0.25).abs() < f32::EPSILON);
    assert!((cfg.restore_perfect - 0.50).abs() < f32::EPSILON);

    let state = app.world().resource::<ErosionState>();
    assert!(
        (state.width_fraction - 1.0).abs() < f32::EPSILON,
        "activate must reset state to 1.0, got {}",
        state.width_fraction
    );
}

// ── E34 — repeat activate with same tuning is idempotent ──────────────

#[test]
fn repeat_activate_with_same_tuning_is_idempotent_across_three_ticks() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Erosion {
                shrink_rate:      0.05,
                min_width_frac:   0.35,
                restore_nonwhiff: 0.25,
                restore_perfect:  0.50,
            },
            &mut commands,
        );
    });
    app.update();
    app.update();
    app.update();

    let cfg = app.world().resource::<ErosionConfig>();
    assert!((cfg.shrink_rate - 0.05).abs() < f32::EPSILON);
    assert!((cfg.min_width_frac - 0.35).abs() < f32::EPSILON);
    let state = app.world().resource::<ErosionState>();
    assert!((state.width_fraction - 1.0).abs() < f32::EPSILON);
}

// ── E35 — non-Erosion tuning variant leaves existing resources intact ──

#[test]
fn non_erosion_tuning_variant_leaves_existing_resources_intact() {
    let mut app = TestAppBuilder::new().build();
    // Pre-insert stale Erosion resources.
    app.world_mut().insert_resource(ErosionConfig {
        shrink_rate:      0.09,
        min_width_frac:   0.30,
        restore_nonwhiff: 0.15,
        restore_perfect:  0.45,
    });
    app.world_mut().insert_resource(ErosionState {
        width_fraction: 0.55,
    });
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Haste {
                base_percent:      20.0,
                per_level_percent: 10.0,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<ErosionConfig>();
    assert!(
        (cfg.shrink_rate - 0.09).abs() < f32::EPSILON,
        "non-Erosion tuning must not overwrite existing ErosionConfig"
    );
    assert!((cfg.min_width_frac - 0.30).abs() < f32::EPSILON);
    assert!((cfg.restore_nonwhiff - 0.15).abs() < f32::EPSILON);
    assert!((cfg.restore_perfect - 0.45).abs() < f32::EPSILON);
    let state = app.world().resource::<ErosionState>();
    assert!(
        (state.width_fraction - 0.55).abs() < f32::EPSILON,
        "non-Erosion tuning must not reset ErosionState"
    );
}
