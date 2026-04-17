//! Siphon protocol — scaffold.

use bevy::prelude::*;

use crate::{
    prelude::*,
    protocol::{
        definition::{ProtocolKind, ProtocolTuning},
        resources::protocol_active,
    },
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct SiphonConfig {
    pub(crate) streak_window: f32,
    pub(crate) time_per_kill: f32,
}

pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::Siphon {
        streak_window,
        time_per_kill,
    } = *tuning
    else {
        warn!("siphon::activate called with non-Siphon tuning");
        return;
    };
    commands.insert_resource(SiphonConfig {
        streak_window,
        time_per_kill,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        warn_stub.run_if(protocol_active(ProtocolKind::Siphon)),
    );
}

fn warn_stub(cfg: Option<Res<SiphonConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    warn!("Siphon activated — runtime behaviour pending kill-streak plumbing.");
    *seen = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activate_with_matching_tuning_inserts_config() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &ProtocolTuning::Siphon {
                    streak_window: 2.0,
                    time_per_kill: 0.25,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<SiphonConfig>();
        assert!((cfg.streak_window - 2.0).abs() < f32::EPSILON);
        assert!((cfg.time_per_kill - 0.25).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<SiphonConfig>().is_none());
    }
}
