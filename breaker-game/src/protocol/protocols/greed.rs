//! Greed protocol — scaffold.
//!
//! Full behaviour (boost rare-rarity weight per skipped chip-select offer)
//! will hook into `generate_chip_offerings` via the skip counter once
//! counter plumbing lands. Scaffold installs the config only.

use bevy::prelude::*;

use crate::{
    prelude::*,
    protocol::{
        definition::{ProtocolKind, ProtocolTuning},
        resources::protocol_active,
    },
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct GreedConfig {
    pub(crate) rarity_boost_per_skip: f32,
}

pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::Greed {
        rarity_boost_per_skip,
    } = *tuning
    else {
        warn!("greed::activate called with non-Greed tuning");
        return;
    };
    commands.insert_resource(GreedConfig {
        rarity_boost_per_skip,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        warn_stub.run_if(protocol_active(ProtocolKind::Greed)),
    );
}

fn warn_stub(cfg: Option<Res<GreedConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    warn!("Greed activated — rarity weighting hook pending.");
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
                &ProtocolTuning::Greed {
                    rarity_boost_per_skip: 0.05,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<GreedConfig>();
        assert!((cfg.rarity_boost_per_skip - 0.05).abs() < f32::EPSILON);
    }

    #[test]
    fn activate_with_mismatched_tuning_does_nothing() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &ProtocolTuning::DebtCollector {
                    stack_per_bump: 0.1,
                },
                &mut commands,
            );
        });
        app.update();
        assert!(app.world().get_resource::<GreedConfig>().is_none());
    }
}
