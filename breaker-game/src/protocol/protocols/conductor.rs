//! Conductor protocol — scaffold.

use bevy::prelude::*;

use crate::protocol::{
    definition::{ProtocolKind, ProtocolTuning},
    resources::protocol_active,
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct ConductorConfig {
    pub(crate) primary_swap_window: f32,
}

pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::Conductor {
        primary_swap_window,
    } = *tuning
    else {
        warn!("conductor::activate called with non-Conductor tuning");
        return;
    };
    commands.insert_resource(ConductorConfig {
        primary_swap_window,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        warn_stub.run_if(protocol_active(ProtocolKind::Conductor)),
    );
}

fn warn_stub(cfg: Option<Res<ConductorConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    info!("Conductor activated — primary-swap plumbing pending.");
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
                &ProtocolTuning::Conductor {
                    primary_swap_window: 0.2,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<ConductorConfig>();
        assert!((cfg.primary_swap_window - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn activate_with_mismatched_tuning_does_nothing() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &ProtocolTuning::Greed {
                    rarity_boost_per_skip: 0.05,
                },
                &mut commands,
            );
        });
        app.update();
        assert!(app.world().get_resource::<ConductorConfig>().is_none());
    }
}
