//! Cascade hazard — scaffold.

use bevy::prelude::*;

use crate::hazard::{
    definition::{HazardKind, HazardTuning},
    resources::hazard_active,
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct CascadeConfig {
    pub(crate) base_heal:      f32,
    pub(crate) per_level_heal: f32,
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Cascade {
        base_heal,
        per_level_heal,
    } = *tuning
    else {
        warn!("cascade::activate called with non-Cascade tuning");
        return;
    };
    commands.insert_resource(CascadeConfig {
        base_heal,
        per_level_heal,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(Update, warn_stub.run_if(hazard_active(HazardKind::Cascade)));
}

fn warn_stub(cfg: Option<Res<CascadeConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    warn!("Cascade activated — cell-heal-on-clear plumbing pending.");
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
}
