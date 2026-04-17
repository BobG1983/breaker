//! Fission protocol — scaffold.

use bevy::prelude::*;

use crate::protocol::{
    definition::{ProtocolKind, ProtocolTuning},
    resources::protocol_active,
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct FissionConfig {
    pub(crate) kills_per_split: u32,
}

pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::Fission { kills_per_split } = *tuning else {
        warn!("fission::activate called with non-Fission tuning");
        return;
    };
    commands.insert_resource(FissionConfig { kills_per_split });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        warn_stub.run_if(protocol_active(ProtocolKind::Fission)),
    );
}

fn warn_stub(cfg: Option<Res<FissionConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    warn!("Fission activated — bolt-split plumbing pending.");
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
                &ProtocolTuning::Fission {
                    kills_per_split: 10,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<FissionConfig>();
        assert_eq!(cfg.kills_per_split, 10);
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
        assert!(app.world().get_resource::<FissionConfig>().is_none());
    }
}
