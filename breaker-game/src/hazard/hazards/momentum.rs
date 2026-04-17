//! Momentum hazard — scaffold.

use bevy::prelude::*;

use crate::hazard::{
    definition::{HazardKind, HazardTuning},
    resources::hazard_active,
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct MomentumConfig {
    pub(crate) base_hp_per_hit:      f32,
    pub(crate) per_level_hp_per_hit: f32,
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Momentum {
        base_hp_per_hit,
        per_level_hp_per_hit,
    } = *tuning
    else {
        warn!("momentum::activate called with non-Momentum tuning");
        return;
    };
    commands.insert_resource(MomentumConfig {
        base_hp_per_hit,
        per_level_hp_per_hit,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        warn_stub.run_if(hazard_active(HazardKind::Momentum)),
    );
}

fn warn_stub(cfg: Option<Res<MomentumConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    warn!("Momentum activated — damage-pipeline non-lethal redistribution pending.");
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
                &HazardTuning::Momentum {
                    base_hp_per_hit:      1.0,
                    per_level_hp_per_hit: 0.5,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<MomentumConfig>();
        assert!((cfg.base_hp_per_hit - 1.0).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<MomentumConfig>().is_none());
    }
}
