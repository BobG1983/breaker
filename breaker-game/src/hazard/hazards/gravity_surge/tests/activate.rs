//! Group G — `activate` lifecycle.
//!
//! `activate(&tuning, &mut commands)` inserts `GravitySurgeConfig` from
//! `HazardTuning::GravitySurge`. Mismatched tuning is a warn + no-op.
//! Every test builds a minimal app and invokes `activate` via the
//! `CommandQueue`-based `activate_now` helper (deterministic one-shot
//! flush — do NOT use `app.add_systems` for these tests).

use bevy::prelude::*;

use super::{super::system::GravitySurgeConfig, helpers::activate_now};
use crate::{hazard::definition::HazardTuning, prelude::*};

// ── Behavior 46 — matching tuning inserts config with all 4 fields ────

#[test]
fn activate_with_matching_tuning_inserts_config() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        super::super::system::activate(
            &HazardTuning::GravitySurge {
                base_duration_secs:      2.0,
                per_level_duration_secs: 1.0,
                base_strength:           100.0,
                per_level_strength_frac: 0.2,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<GravitySurgeConfig>();
    assert!((cfg.base_strength - 100.0).abs() < f32::EPSILON);
}

#[test]
fn activate_pins_all_four_fields() {
    // Edge: widen the preserved test to cover every field.
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::GravitySurge {
            base_duration_secs:      2.0,
            per_level_duration_secs: 1.0,
            base_strength:           100.0,
            per_level_strength_frac: 0.2,
        },
    );

    let cfg = app.world().resource::<GravitySurgeConfig>();
    assert!((cfg.base_duration_secs - 2.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_duration_secs - 1.0).abs() < f32::EPSILON);
    assert!((cfg.base_strength - 100.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_strength_frac - 0.2).abs() < 1e-5);
}

// ── Behavior 47 — mismatched Decay variant inserts nothing — PRESERVED

#[test]
fn activate_with_mismatched_tuning_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        super::super::system::activate(
            &HazardTuning::Decay {
                base_percent:      0.05,
                per_level_percent: 0.03,
            },
            &mut commands,
        );
    });
    app.update();
    assert!(app.world().get_resource::<GravitySurgeConfig>().is_none());
}

#[test]
fn activate_with_nan_decay_tuning_no_panic() {
    // Edge: noisy NaN tuning in the mismatched branch — no panic, no insert.
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Decay {
            base_percent:      f32::NAN,
            per_level_percent: f32::NAN,
        },
    );
    assert!(app.world().get_resource::<GravitySurgeConfig>().is_none());
}

// ── Behavior 48 — mismatched Haste variant inserts nothing ────────────

#[test]
fn activate_with_haste_tuning_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Haste {
            base_percent:      0.1,
            per_level_percent: 0.05,
        },
    );
    assert!(app.world().get_resource::<GravitySurgeConfig>().is_none());
}

#[test]
fn activate_with_overcharge_tuning_does_nothing() {
    // Edge: another mismatched variant — confirms discriminant check is
    // GravitySurge-specific, not just Decay-specific.
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Overcharge {
            base_frac:      0.05,
            per_level_frac: 0.03,
        },
    );
    assert!(app.world().get_resource::<GravitySurgeConfig>().is_none());
}

// ── Behavior 49 — second activate overwrites (last-write-wins) ───────

#[test]
fn second_activate_overwrites_gravity_surge_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::GravitySurge {
            base_duration_secs:      1.0,
            per_level_duration_secs: 0.5,
            base_strength:           100.0,
            per_level_strength_frac: 0.1,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::GravitySurge {
            base_duration_secs:      4.0,
            per_level_duration_secs: 2.0,
            base_strength:           1000.0,
            per_level_strength_frac: 0.75,
        },
    );

    let cfg = app.world().resource::<GravitySurgeConfig>();
    assert!((cfg.base_duration_secs - 4.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_duration_secs - 2.0).abs() < f32::EPSILON);
    assert!((cfg.base_strength - 1000.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_strength_frac - 0.75).abs() < 1e-5);
}

#[test]
fn third_activate_with_zero_values_is_respected() {
    // Edge: third activate with zero values overwrites cleanly.
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::GravitySurge {
            base_duration_secs:      1.0,
            per_level_duration_secs: 0.5,
            base_strength:           100.0,
            per_level_strength_frac: 0.1,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::GravitySurge {
            base_duration_secs:      4.0,
            per_level_duration_secs: 2.0,
            base_strength:           1000.0,
            per_level_strength_frac: 0.75,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::GravitySurge {
            base_duration_secs:      0.1,
            per_level_duration_secs: 0.0,
            base_strength:           50.0,
            per_level_strength_frac: 0.0,
        },
    );

    let cfg = app.world().resource::<GravitySurgeConfig>();
    assert!((cfg.base_duration_secs - 0.1).abs() < 1e-5);
    assert!(cfg.per_level_duration_secs.abs() < f32::EPSILON);
    assert!((cfg.base_strength - 50.0).abs() < f32::EPSILON);
    assert!(cfg.per_level_strength_frac.abs() < f32::EPSILON);
}

// ── Behavior 50 — activate on an empty world does not panic ──────────

#[test]
fn activate_on_empty_world_does_not_panic() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::GravitySurge {
            base_duration_secs:      2.0,
            per_level_duration_secs: 1.0,
            base_strength:           500.0,
            per_level_strength_frac: 0.5,
        },
    );

    let cfg = app.world().resource::<GravitySurgeConfig>();
    assert!((cfg.base_duration_secs - 2.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_duration_secs - 1.0).abs() < f32::EPSILON);
    assert!((cfg.base_strength - 500.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_strength_frac - 0.5).abs() < 1e-5);
}

#[test]
fn mismatched_activate_after_matching_does_not_clear_config() {
    // Edge: mismatched variant does NOT remove an existing config.
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::GravitySurge {
            base_duration_secs:      2.0,
            per_level_duration_secs: 1.0,
            base_strength:           500.0,
            per_level_strength_frac: 0.5,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::Decay {
            base_percent:      0.05,
            per_level_percent: 0.03,
        },
    );

    // Config still present with original values.
    let cfg = app.world().resource::<GravitySurgeConfig>();
    assert!((cfg.base_duration_secs - 2.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_duration_secs - 1.0).abs() < f32::EPSILON);
    assert!((cfg.base_strength - 500.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_strength_frac - 0.5).abs() < 1e-5);
}
