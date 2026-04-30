use super::super::{super::system::*, helpers::*};
use crate::{mutators::hazards::definition::HazardTuning, prelude::*};

// ── E. activate — extended coverage ───────────────────────────────────

// Behavior 42 — matching EchoCells tuning pins all three fields.
#[test]
fn activate_now_with_matching_tuning_pins_all_three_fields() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::EchoCells {
            delay_secs:           3.0,
            base_hp:              7.5,
            per_level_multiplier: 1.25,
        },
    );

    let cfg = app.world().resource::<EchoCellsConfig>();
    assert!((cfg.delay_secs - 3.0).abs() < f32::EPSILON);
    assert!((cfg.base_hp - 7.5).abs() < f32::EPSILON);
    assert!((cfg.per_level_multiplier - 1.25).abs() < f32::EPSILON);
}

// Behavior 43 — mismatched Drift tuning inserts nothing.
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
    assert!(app.world().get_resource::<EchoCellsConfig>().is_none());
}

// Behavior 44 — mismatched Fracture tuning inserts nothing.
#[test]
fn activate_now_with_mismatched_fracture_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Fracture {
            base_splits:      2,
            per_level_splits: 1,
        },
    );
    assert!(app.world().get_resource::<EchoCellsConfig>().is_none());
}

// Behavior 45 — mismatched Overcharge tuning inserts nothing.
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
    assert!(app.world().get_resource::<EchoCellsConfig>().is_none());
}

// Behavior 46 — mismatched Cascade with NaN does not panic.
#[test]
fn activate_now_with_mismatched_cascade_nan_does_not_panic() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Cascade {
            base_heal:      f32::NAN,
            per_level_heal: f32::NAN,
        },
    );
    assert!(app.world().get_resource::<EchoCellsConfig>().is_none());
}

// Behavior 47 — second activate overwrites (last-write-wins).
#[test]
fn second_activate_overwrites_echo_cells_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::EchoCells {
            delay_secs:           1.5,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::EchoCells {
            delay_secs:           3.0,
            base_hp:              5.0,
            per_level_multiplier: 1.5,
        },
    );

    let cfg = app.world().resource::<EchoCellsConfig>();
    assert!((cfg.delay_secs - 3.0).abs() < f32::EPSILON);
    assert!((cfg.base_hp - 5.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_multiplier - 1.5).abs() < f32::EPSILON);
}

// Behavior 48 — third activate overwrites to boundary zeros.
#[test]
fn third_activate_overwrites_to_boundary_values() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::EchoCells {
            delay_secs:           1.5,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::EchoCells {
            delay_secs:           3.0,
            base_hp:              5.0,
            per_level_multiplier: 1.5,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::EchoCells {
            delay_secs:           0.0,
            base_hp:              0.0,
            per_level_multiplier: 0.0,
        },
    );

    let cfg = app.world().resource::<EchoCellsConfig>();
    assert!((cfg.delay_secs - 0.0).abs() < f32::EPSILON);
    assert!((cfg.base_hp - 0.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_multiplier - 0.0).abs() < f32::EPSILON);
}

// Behavior 49 — mismatch after match preserves the existing config.
#[test]
fn activate_now_mismatch_after_match_preserves_existing_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::EchoCells {
            delay_secs:           2.0,
            base_hp:              4.0,
            per_level_multiplier: 2.0,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::Haste {
            base_percent:      0.1,
            per_level_percent: 0.05,
        },
    );

    let cfg = app.world().resource::<EchoCellsConfig>();
    assert!((cfg.delay_secs - 2.0).abs() < f32::EPSILON);
    assert!((cfg.base_hp - 4.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_multiplier - 2.0).abs() < f32::EPSILON);
}

// Behavior 50 — activate on an empty world does not panic.
#[test]
fn activate_now_on_empty_world_does_not_panic() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::EchoCells {
            delay_secs:           1.5,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        },
    );

    let cfg = app.world().resource::<EchoCellsConfig>();
    assert!((cfg.delay_secs - 1.5).abs() < f32::EPSILON);
    assert!((cfg.base_hp - 1.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_multiplier - 2.0).abs() < f32::EPSILON);
}
