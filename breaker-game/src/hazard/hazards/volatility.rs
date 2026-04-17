//! Volatility hazard — scaffold.

use bevy::prelude::*;

use crate::hazard::{
    definition::{HazardKind, HazardTuning},
    resources::hazard_active,
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct VolatilityConfig {
    pub(crate) hp_per_5s:                f32,
    pub(crate) cap_multiplier:           f32,
    pub(crate) per_level_reduction_frac: f32,
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Volatility {
        hp_per_5s,
        cap_multiplier,
        per_level_reduction_frac,
    } = *tuning
    else {
        warn!("volatility::activate called with non-Volatility tuning");
        return;
    };
    commands.insert_resource(VolatilityConfig {
        hp_per_5s,
        cap_multiplier,
        per_level_reduction_frac,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        warn_stub.run_if(hazard_active(HazardKind::Volatility)),
    );
}

fn warn_stub(cfg: Option<Res<VolatilityConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    info!("Volatility activated — cell-HP-growth plumbing pending.");
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
                &HazardTuning::Volatility {
                    hp_per_5s:                1.0,
                    cap_multiplier:           3.0,
                    per_level_reduction_frac: 0.1,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<VolatilityConfig>();
        assert!((cfg.hp_per_5s - 1.0).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<VolatilityConfig>().is_none());
    }
}
