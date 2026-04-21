use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::{hazard::definition::HazardTuning, prelude::*};

// ── activate — preserved + new ────────────────────────────────────────

// Behavior 28 — matching tuning inserts config.
#[test]
fn activate_with_matching_tuning_inserts_config() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Fracture {
                base_splits:      2,
                per_level_splits: 1,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<FractureConfig>();
    assert_eq!(cfg.base_splits, 2);
}

#[test]
fn activate_now_with_matching_tuning_pins_both_fields() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Fracture {
            base_splits:      2,
            per_level_splits: 1,
        },
    );

    let cfg = app.world().resource::<FractureConfig>();
    assert_eq!(cfg.base_splits, 2);
    assert_eq!(cfg.per_level_splits, 1);
}

// Behavior 29 — mismatched Decay tuning inserts nothing.
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
    assert!(app.world().get_resource::<FractureConfig>().is_none());
}

#[test]
fn activate_now_with_mismatched_decay_nan_does_not_panic() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Decay {
            base_percent:      f32::NAN,
            per_level_percent: f32::NAN,
        },
    );
    assert!(app.world().get_resource::<FractureConfig>().is_none());
}

// Behavior 30 — mismatched Haste tuning inserts nothing.
#[test]
fn activate_now_with_mismatched_haste_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Haste {
            base_percent:      0.1,
            per_level_percent: 0.05,
        },
    );
    assert!(app.world().get_resource::<FractureConfig>().is_none());
}

#[test]
fn activate_now_with_mismatched_drift_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Drift {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.3,
        },
    );
    assert!(app.world().get_resource::<FractureConfig>().is_none());
}

// Behavior 31 — mismatched Overcharge tuning inserts nothing.
#[test]
fn activate_now_with_mismatched_overcharge_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Overcharge {
            base_frac:      0.05,
            per_level_frac: 0.03,
        },
    );
    assert!(app.world().get_resource::<FractureConfig>().is_none());
}

#[test]
fn activate_now_with_mismatched_cascade_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Cascade {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    assert!(app.world().get_resource::<FractureConfig>().is_none());
}

// Behavior 32 — second activate overwrites (last-write-wins).
#[test]
fn second_activate_overwrites_fracture_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Fracture {
            base_splits:      1,
            per_level_splits: 0,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::Fracture {
            base_splits:      4,
            per_level_splits: 2,
        },
    );

    let cfg = app.world().resource::<FractureConfig>();
    assert_eq!(cfg.base_splits, 4);
    assert_eq!(cfg.per_level_splits, 2);
}

#[test]
fn third_activate_overwrites_to_zero_values() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Fracture {
            base_splits:      1,
            per_level_splits: 0,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::Fracture {
            base_splits:      4,
            per_level_splits: 2,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::Fracture {
            base_splits:      0,
            per_level_splits: 0,
        },
    );

    let cfg = app.world().resource::<FractureConfig>();
    assert_eq!(cfg.base_splits, 0);
    assert_eq!(cfg.per_level_splits, 0);
}

// Behavior 33 — activate on empty world does not panic.
#[test]
fn activate_now_on_empty_world_does_not_panic() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Fracture {
            base_splits:      2,
            per_level_splits: 1,
        },
    );

    let cfg = app.world().resource::<FractureConfig>();
    assert_eq!(cfg.base_splits, 2);
    assert_eq!(cfg.per_level_splits, 1);
}

#[test]
fn activate_now_mismatch_after_match_preserves_existing_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Fracture {
            base_splits:      2,
            per_level_splits: 1,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::Decay {
            base_percent:      0.05,
            per_level_percent: 0.03,
        },
    );

    let cfg = app.world().resource::<FractureConfig>();
    assert_eq!(cfg.base_splits, 2);
    assert_eq!(cfg.per_level_splits, 1);
}
