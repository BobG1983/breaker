//! Decay hazard — scaffold.

use bevy::prelude::*;

use crate::hazard::{
    definition::{HazardKind, HazardTuning},
    resources::hazard_active,
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct DecayConfig {
    pub(crate) base_percent:      f32,
    pub(crate) per_level_percent: f32,
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Decay {
        base_percent,
        per_level_percent,
    } = *tuning
    else {
        warn!("decay::activate called with non-Decay tuning");
        return;
    };
    commands.insert_resource(DecayConfig {
        base_percent,
        per_level_percent,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(Update, warn_stub.run_if(hazard_active(HazardKind::Decay)));
}

fn warn_stub(cfg: Option<Res<DecayConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    warn!("Decay activated — cell HP drain plumbing pending.");
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
                &HazardTuning::Decay {
                    base_percent:      0.05,
                    per_level_percent: 0.03,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<DecayConfig>();
        assert!((cfg.base_percent - 0.05).abs() < f32::EPSILON);
        assert!((cfg.per_level_percent - 0.03).abs() < f32::EPSILON);
    }

    #[test]
    fn activate_with_mismatched_tuning_does_nothing() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &HazardTuning::Haste {
                    base_percent:      0.1,
                    per_level_percent: 0.05,
                },
                &mut commands,
            );
        });
        app.update();
        assert!(app.world().get_resource::<DecayConfig>().is_none());
    }
}
