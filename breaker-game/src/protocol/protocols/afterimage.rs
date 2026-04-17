//! Afterimage protocol — scaffold.

use bevy::prelude::*;

use crate::protocol::{
    definition::{ProtocolKind, ProtocolTuning},
    resources::protocol_active,
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct AfterimageConfig {
    pub(crate) phantom_duration:      f32,
    pub(crate) phantom_bolt_duration: f32,
}

pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::Afterimage {
        phantom_duration,
        phantom_bolt_duration,
    } = *tuning
    else {
        warn!("afterimage::activate called with non-Afterimage tuning");
        return;
    };
    commands.insert_resource(AfterimageConfig {
        phantom_duration,
        phantom_bolt_duration,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        warn_stub.run_if(protocol_active(ProtocolKind::Afterimage)),
    );
}

fn warn_stub(cfg: Option<Res<AfterimageConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    info!("Afterimage activated — phantom-bolt plumbing pending.");
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
                &ProtocolTuning::Afterimage {
                    phantom_duration:      1.5,
                    phantom_bolt_duration: 0.75,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<AfterimageConfig>();
        assert!((cfg.phantom_duration - 1.5).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<AfterimageConfig>().is_none());
    }
}
