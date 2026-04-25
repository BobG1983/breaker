use bevy::prelude::*;

use super::super::system::*;
use crate::{mutators::hazards::definition::HazardTuning, prelude::*};

// ══════════════════════════════════════════════════════════════════════
// activate — preserved scaffold test (mismatched tuning)
// ══════════════════════════════════════════════════════════════════════

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
    assert!(app.world().get_resource::<VolatilityConfig>().is_none());
}
