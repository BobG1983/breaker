//! Drift hazard — scaffold.

use bevy::prelude::*;

use crate::hazard::{
    definition::{HazardKind, HazardTuning},
    resources::hazard_active,
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct DriftConfig {
    pub(crate) force:           f32,
    pub(crate) period_secs:     f32,
    pub(crate) per_level_force: f32,
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Drift {
        force,
        period_secs,
        per_level_force,
    } = *tuning
    else {
        warn!("drift::activate called with non-Drift tuning");
        return;
    };
    commands.insert_resource(DriftConfig {
        force,
        period_secs,
        per_level_force,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(Update, warn_stub.run_if(hazard_active(HazardKind::Drift)));
}

fn warn_stub(cfg: Option<Res<DriftConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    warn!("Drift activated — periodic-lateral-force plumbing pending.");
    *seen = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::TestAppBuilder;

    #[test]
    fn activate_with_matching_tuning_inserts_config() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &HazardTuning::Drift {
                    force:           100.0,
                    period_secs:     8.0,
                    per_level_force: 33.3,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<DriftConfig>();
        assert!((cfg.force - 100.0).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<DriftConfig>().is_none());
    }
}
