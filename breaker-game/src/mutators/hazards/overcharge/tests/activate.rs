//! Group E — `activate` lifecycle.
//!
//! Fresh `TestAppBuilder::new().build()` apps with a one-shot `Update`
//! system that calls `activate(...)` via `Commands`.

use bevy::prelude::*;

use super::super::system::{OverchargeConfig, activate};
use crate::{mutators::hazards::definition::HazardTuning, prelude::*};

// ── Behavior 34 — activate with matching Overcharge tuning inserts config ─

#[test]
fn activate_with_matching_tuning_inserts_config() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Overcharge {
                base_frac:      0.1,
                per_level_frac: 0.05,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<OverchargeConfig>();
    assert!((cfg.base_frac - 0.1).abs() < f32::EPSILON);
    assert!((cfg.per_level_frac - 0.05).abs() < f32::EPSILON);
}

#[test]
fn activate_round_trips_canonical_tuning_values_into_resource() {
    // Edge: the canonical pair (0.05, 0.03) round-trips unchanged.
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Overcharge {
                base_frac:      0.05,
                per_level_frac: 0.03,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<OverchargeConfig>();
    assert!((cfg.base_frac - 0.05).abs() < f32::EPSILON);
    assert!((cfg.per_level_frac - 0.03).abs() < f32::EPSILON);
}

// ── Behavior 35 — activate with mismatched Decay tuning inserts nothing ─

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
    assert!(app.world().get_resource::<OverchargeConfig>().is_none());
}

// ── Behavior 36 — activate with Haste tuning also inserts nothing ───────

#[test]
fn activate_with_haste_tuning_inserts_nothing() {
    let mut app = TestAppBuilder::new().build();
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
    assert!(app.world().get_resource::<OverchargeConfig>().is_none());
}

#[test]
fn activate_with_erosion_tuning_inserts_nothing() {
    // Edge: repeat with Erosion — still no OverchargeConfig. Confirms the
    // let-else pattern rejects every non-Overcharge variant.
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
    assert!(app.world().get_resource::<OverchargeConfig>().is_none());
}

// ── Behavior 37 — second activate call overwrites (last-write-wins) ─────

#[test]
fn second_activate_call_overwrites_prior_config() {
    let mut app = TestAppBuilder::new().build();
    // Pre-insert a stale config so `activate` must overwrite it.
    app.world_mut().insert_resource(OverchargeConfig {
        base_frac:      0.01,
        per_level_frac: 0.002,
    });
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Overcharge {
                base_frac:      0.07,
                per_level_frac: 0.04,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<OverchargeConfig>();
    assert!((cfg.base_frac - 0.07).abs() < f32::EPSILON);
    assert!((cfg.per_level_frac - 0.04).abs() < f32::EPSILON);
}

#[test]
fn second_activate_call_idempotent_when_tuning_unchanged() {
    // Edge: firing the same tuning a second time leaves the values intact.
    let mut app = TestAppBuilder::new().build();
    app.world_mut().insert_resource(OverchargeConfig {
        base_frac:      0.01,
        per_level_frac: 0.002,
    });
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Overcharge {
                base_frac:      0.07,
                per_level_frac: 0.04,
            },
            &mut commands,
        );
    });
    app.update();
    app.update();

    let cfg = app.world().resource::<OverchargeConfig>();
    assert!((cfg.base_frac - 0.07).abs() < f32::EPSILON);
    assert!((cfg.per_level_frac - 0.04).abs() < f32::EPSILON);
}

// ── Behavior 38 — activate on empty world does not panic ────────────────

#[test]
fn activate_on_empty_world_does_not_panic() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Overcharge {
                base_frac:      0.05,
                per_level_frac: 0.03,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<OverchargeConfig>();
    assert!((cfg.base_frac - 0.05).abs() < f32::EPSILON);
}

#[test]
fn activate_with_mismatch_on_empty_world_does_not_panic_and_inserts_nothing() {
    // Edge: same empty world, mismatched variant → no panic, no insert.
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
    assert!(app.world().get_resource::<OverchargeConfig>().is_none());
}
