//! Fracture hazard — scaffold.

use bevy::prelude::*;

use crate::hazard::{
    definition::{HazardKind, HazardTuning},
    resources::hazard_active,
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct FractureConfig {
    pub(crate) base_splits:      u32,
    pub(crate) per_level_splits: u32,
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Fracture {
        base_splits,
        per_level_splits,
    } = *tuning
    else {
        warn!("fracture::activate called with non-Fracture tuning");
        return;
    };
    commands.insert_resource(FractureConfig {
        base_splits,
        per_level_splits,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        warn_stub.run_if(hazard_active(HazardKind::Fracture)),
    );
}

fn warn_stub(cfg: Option<Res<FractureConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    info!("Fracture activated — cell-split plumbing pending.");
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
                &HazardTuning::Fracture {
                    base_splits:      1,
                    per_level_splits: 1,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<FractureConfig>();
        assert_eq!(cfg.base_splits, 1);
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
        assert!(app.world().get_resource::<FractureConfig>().is_none());
    }
}
