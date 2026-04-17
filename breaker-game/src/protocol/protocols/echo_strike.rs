//! `EchoStrike` protocol — scaffold.

use bevy::prelude::*;

use crate::{
    prelude::*,
    protocol::{
        definition::{ProtocolKind, ProtocolTuning},
        resources::protocol_active,
    },
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct EchoStrikeConfig {
    pub(crate) max_echoes:      u32,
    pub(crate) newest_fraction: f32,
    pub(crate) middle_fraction: f32,
    pub(crate) oldest_fraction: f32,
}

pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::EchoStrike {
        max_echoes,
        newest_fraction,
        middle_fraction,
        oldest_fraction,
    } = *tuning
    else {
        warn!("echo_strike::activate called with non-EchoStrike tuning");
        return;
    };
    commands.insert_resource(EchoStrikeConfig {
        max_echoes,
        newest_fraction,
        middle_fraction,
        oldest_fraction,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        warn_stub.run_if(protocol_active(ProtocolKind::EchoStrike)),
    );
}

fn warn_stub(cfg: Option<Res<EchoStrikeConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    warn!("EchoStrike activated — runtime behaviour pending echo-hit plumbing.");
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
                &ProtocolTuning::EchoStrike {
                    max_echoes:      3,
                    newest_fraction: 0.5,
                    middle_fraction: 0.25,
                    oldest_fraction: 0.125,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<EchoStrikeConfig>();
        assert_eq!(cfg.max_echoes, 3);
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
        assert!(app.world().get_resource::<EchoStrikeConfig>().is_none());
    }
}
