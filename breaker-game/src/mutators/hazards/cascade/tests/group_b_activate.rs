use bevy::prelude::*;

use super::super::system::*;
use crate::{mutators::hazards::definition::HazardTuning, prelude::*};

// ════════════════════════════════════════════════════════════════════════════
// Group B — activate preserved scaffold tests (unchanged)
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn activate_with_matching_tuning_inserts_config() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Cascade {
                base_heal:      1.0,
                per_level_heal: 0.5,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<CascadeConfig>();
    assert!((cfg.base_heal - 1.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_heal - 0.5).abs() < f32::EPSILON);
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
    assert!(app.world().get_resource::<CascadeConfig>().is_none());
}

// Behavior 4 edge case: second activate overwrites prior config (last-write-wins
// semantics from `commands.insert_resource`).
#[test]
fn second_activate_overwrites_prior_cascade_config() {
    let mut app = TestAppBuilder::new().build();
    app.world_mut().insert_resource(CascadeConfig {
        base_heal:      1.0,
        per_level_heal: 0.5,
    });
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Cascade {
                base_heal:      99.0,
                per_level_heal: 9.9,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<CascadeConfig>();
    assert!(
        (cfg.base_heal - 99.0).abs() < f32::EPSILON,
        "second activate must overwrite base_heal, got {}",
        cfg.base_heal
    );
    assert!(
        (cfg.per_level_heal - 9.9).abs() < f32::EPSILON,
        "second activate must overwrite per_level_heal, got {}",
        cfg.per_level_heal
    );
}
