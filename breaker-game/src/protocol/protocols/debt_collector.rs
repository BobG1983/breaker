//! `DebtCollector` protocol — scaffold.
//!
//! Full behaviour (accumulate bump-grade stacks, cash out on next cell kill
//! as bonus damage) requires bump-grade event plumbing from the bolt domain.
//! This module installs the config + a diagnostic stub so the protocol is
//! selectable today.

use bevy::prelude::*;

use crate::{
    prelude::*,
    protocol::{
        definition::{ProtocolKind, ProtocolTuning},
        resources::protocol_active,
    },
};

/// Tuning extracted from `ProtocolTuning::DebtCollector` at activation time.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct DebtCollectorConfig {
    /// Damage added to the "debt" per successful bump.
    pub(crate) stack_per_bump: f32,
}

/// Activation entry.
pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::DebtCollector { stack_per_bump } = *tuning else {
        warn!("debt_collector::activate called with non-DebtCollector tuning");
        return;
    };
    commands.insert_resource(DebtCollectorConfig { stack_per_bump });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        warn_stub.run_if(protocol_active(ProtocolKind::DebtCollector)),
    );
}

fn warn_stub(cfg: Option<Res<DebtCollectorConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    warn!("DebtCollector activated — runtime behaviour pending bump-grade plumbing.");
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
                &ProtocolTuning::DebtCollector {
                    stack_per_bump: 0.1,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<DebtCollectorConfig>();
        assert!((cfg.stack_per_bump - 0.1).abs() < f32::EPSILON);
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

        assert!(app.world().get_resource::<DebtCollectorConfig>().is_none());
    }
}
