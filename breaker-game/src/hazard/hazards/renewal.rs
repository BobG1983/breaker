//! Renewal hazard — scaffold.

use bevy::prelude::*;

use crate::hazard::{
    definition::{HazardKind, HazardTuning},
    resources::hazard_active,
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct RenewalConfig {
    pub(crate) base_period_secs:         f32,
    pub(crate) per_level_reduction_frac: f32,
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Renewal {
        base_period_secs,
        per_level_reduction_frac,
    } = *tuning
    else {
        warn!("renewal::activate called with non-Renewal tuning");
        return;
    };
    commands.insert_resource(RenewalConfig {
        base_period_secs,
        per_level_reduction_frac,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(Update, warn_stub.run_if(hazard_active(HazardKind::Renewal)));
}

fn warn_stub(cfg: Option<Res<RenewalConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    info!("Renewal activated — periodic-node-refresh plumbing pending.");
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
                &HazardTuning::Renewal {
                    base_period_secs:         10.0,
                    per_level_reduction_frac: 0.2,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<RenewalConfig>();
        assert!((cfg.base_period_secs - 10.0).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<RenewalConfig>().is_none());
    }
}
