//! Tether hazard — scaffold.

use bevy::prelude::*;

use crate::hazard::{
    definition::{HazardKind, HazardTuning},
    resources::hazard_active,
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct TetherConfig {
    pub(crate) base_share:         f32,
    pub(crate) per_level_share:    f32,
    pub(crate) base_coverage:      f32,
    pub(crate) per_level_coverage: f32,
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Tether {
        base_share_frac,
        per_level_share_frac,
        base_coverage_frac,
        per_level_coverage_frac,
    } = *tuning
    else {
        warn!("tether::activate called with non-Tether tuning");
        return;
    };
    commands.insert_resource(TetherConfig {
        base_share:         base_share_frac,
        per_level_share:    per_level_share_frac,
        base_coverage:      base_coverage_frac,
        per_level_coverage: per_level_coverage_frac,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(Update, warn_stub.run_if(hazard_active(HazardKind::Tether)));
}

fn warn_stub(cfg: Option<Res<TetherConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    warn!("Tether activated — paired-cell damage-sharing plumbing pending.");
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
                &HazardTuning::Tether {
                    base_share_frac:         0.25,
                    per_level_share_frac:    0.1,
                    base_coverage_frac:      0.4,
                    per_level_coverage_frac: 0.1,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<TetherConfig>();
        assert!((cfg.base_share - 0.25).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<TetherConfig>().is_none());
    }
}
