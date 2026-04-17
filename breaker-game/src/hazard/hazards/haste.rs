//! Haste hazard — scaffold.

use bevy::prelude::*;

use crate::hazard::{
    definition::{HazardKind, HazardTuning},
    resources::hazard_active,
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct HasteConfig {
    pub(crate) base_percent:      f32,
    pub(crate) per_level_percent: f32,
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Haste {
        base_percent,
        per_level_percent,
    } = *tuning
    else {
        warn!("haste::activate called with non-Haste tuning");
        return;
    };
    commands.insert_resource(HasteConfig {
        base_percent,
        per_level_percent,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(Update, warn_stub.run_if(hazard_active(HazardKind::Haste)));
}

fn warn_stub(cfg: Option<Res<HasteConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    info!("Haste activated — cell-timer acceleration plumbing pending.");
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
                &HazardTuning::Haste {
                    base_percent:      0.1,
                    per_level_percent: 0.05,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<HasteConfig>();
        assert!((cfg.base_percent - 0.1).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<HasteConfig>().is_none());
    }
}
