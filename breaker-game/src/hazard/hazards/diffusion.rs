//! Diffusion hazard — scaffold.

use bevy::prelude::*;

use crate::hazard::{
    definition::{HazardKind, HazardTuning},
    resources::hazard_active,
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct DiffusionConfig {
    pub(crate) base_share_frac:      f32,
    pub(crate) per_level_share_frac: f32,
    pub(crate) depth_every_levels:   u32,
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Diffusion {
        base_share_frac,
        per_level_share_frac,
        depth_every_levels,
    } = *tuning
    else {
        warn!("diffusion::activate called with non-Diffusion tuning");
        return;
    };
    commands.insert_resource(DiffusionConfig {
        base_share_frac,
        per_level_share_frac,
        depth_every_levels,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        warn_stub.run_if(hazard_active(HazardKind::Diffusion)),
    );
}

fn warn_stub(cfg: Option<Res<DiffusionConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    warn!("Diffusion activated — damage-pipeline redistribution plumbing pending.");
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
                &HazardTuning::Diffusion {
                    base_share_frac:      0.2,
                    per_level_share_frac: 0.1,
                    depth_every_levels:   5,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<DiffusionConfig>();
        assert!((cfg.base_share_frac - 0.2).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<DiffusionConfig>().is_none());
    }
}
