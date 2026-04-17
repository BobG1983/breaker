//! Sympathy hazard — scaffold.

use bevy::prelude::*;

use crate::hazard::{
    definition::{HazardKind, HazardTuning},
    resources::hazard_active,
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct SympathyConfig {
    pub(crate) base_heal_frac:      f32,
    pub(crate) per_level_heal_frac: f32,
    pub(crate) depth_every_levels:  u32,
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Sympathy {
        base_heal_frac,
        per_level_heal_frac,
        depth_every_levels,
    } = *tuning
    else {
        warn!("sympathy::activate called with non-Sympathy tuning");
        return;
    };
    commands.insert_resource(SympathyConfig {
        base_heal_frac,
        per_level_heal_frac,
        depth_every_levels,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        warn_stub.run_if(hazard_active(HazardKind::Sympathy)),
    );
}

fn warn_stub(cfg: Option<Res<SympathyConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    warn!("Sympathy activated — adjacent-cell heal plumbing pending.");
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
                &HazardTuning::Sympathy {
                    base_heal_frac:      0.25,
                    per_level_heal_frac: 0.05,
                    depth_every_levels:  5,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<SympathyConfig>();
        assert!((cfg.base_heal_frac - 0.25).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<SympathyConfig>().is_none());
    }
}
