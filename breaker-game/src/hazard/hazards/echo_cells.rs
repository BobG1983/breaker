//! `EchoCells` hazard — scaffold.

use bevy::prelude::*;

use crate::hazard::{
    definition::{HazardKind, HazardTuning},
    resources::hazard_active,
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct EchoCellsConfig {
    pub(crate) delay_secs:           f32,
    pub(crate) base_hp:              f32,
    pub(crate) per_level_multiplier: f32,
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::EchoCells {
        delay_secs,
        base_hp,
        per_level_multiplier,
    } = *tuning
    else {
        warn!("echo_cells::activate called with non-EchoCells tuning");
        return;
    };
    commands.insert_resource(EchoCellsConfig {
        delay_secs,
        base_hp,
        per_level_multiplier,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        warn_stub.run_if(hazard_active(HazardKind::EchoCells)),
    );
}

fn warn_stub(cfg: Option<Res<EchoCellsConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    info!("EchoCells activated — ghost-cell respawn plumbing pending.");
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
                &HazardTuning::EchoCells {
                    delay_secs:           2.0,
                    base_hp:              1.0,
                    per_level_multiplier: 2.0,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<EchoCellsConfig>();
        assert!((cfg.delay_secs - 2.0).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<EchoCellsConfig>().is_none());
    }
}
