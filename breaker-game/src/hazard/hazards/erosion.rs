//! Erosion hazard — scaffold.

use bevy::prelude::*;

use crate::hazard::{
    definition::{HazardKind, HazardTuning},
    resources::hazard_active,
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct ErosionConfig {
    pub(crate) shrink_rate:      f32,
    pub(crate) min_width_frac:   f32,
    pub(crate) restore_nonwhiff: f32,
    pub(crate) restore_perfect:  f32,
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Erosion {
        shrink_rate,
        min_width_frac,
        restore_nonwhiff,
        restore_perfect,
    } = *tuning
    else {
        warn!("erosion::activate called with non-Erosion tuning");
        return;
    };
    commands.insert_resource(ErosionConfig {
        shrink_rate,
        min_width_frac,
        restore_nonwhiff,
        restore_perfect,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(Update, warn_stub.run_if(hazard_active(HazardKind::Erosion)));
}

fn warn_stub(cfg: Option<Res<ErosionConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    warn!("Erosion activated — breaker-shrink plumbing pending.");
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
    }
}
