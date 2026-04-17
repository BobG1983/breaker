//! Overcharge hazard — scaffold.

use bevy::prelude::*;

use crate::hazard::{
    definition::{HazardKind, HazardTuning},
    resources::hazard_active,
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct OverchargeConfig {
    pub(crate) base_frac:      f32,
    pub(crate) per_level_frac: f32,
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Overcharge {
        base_frac,
        per_level_frac,
    } = *tuning
    else {
        warn!("overcharge::activate called with non-Overcharge tuning");
        return;
    };
    commands.insert_resource(OverchargeConfig {
        base_frac,
        per_level_frac,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        warn_stub.run_if(hazard_active(HazardKind::Overcharge)),
    );
}

fn warn_stub(cfg: Option<Res<OverchargeConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    warn!("Overcharge activated — bolt-damage ramp plumbing pending.");
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
                &HazardTuning::Overcharge {
                    base_frac:      0.1,
                    per_level_frac: 0.05,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<OverchargeConfig>();
        assert!((cfg.base_frac - 0.1).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<OverchargeConfig>().is_none());
    }
}
