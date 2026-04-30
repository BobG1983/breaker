use bevy::prelude::*;

use super::super::super::system::*;
use crate::{mutators::hazards::definition::HazardTuning, prelude::*};

// ── activate — preserved scaffold tests ────────────────────────────────

#[test]
fn activate_with_matching_tuning_inserts_config() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::EchoCells {
                delay_secs:           2.0,
                base_hp:              1.0,
                per_level_multiplier: 2.0,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<EchoCellsConfig>();
    assert!((cfg.delay_secs - 2.0).abs() < f32::EPSILON);
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
    assert!(app.world().get_resource::<EchoCellsConfig>().is_none());
}
