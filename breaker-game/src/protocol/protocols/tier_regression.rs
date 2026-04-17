//! `TierRegression` protocol — scaffold only.
//!
//! Full behaviour (re-enter the previous tier's nodes on boss clear) is
//! blocked on the node-sequencing refactor (`docs/todos/TODO.md` follow-up
//! item). This module stands up the config resource, the activation hook,
//! and a diagnostic system so the protocol can be selected today without
//! the game panicking or silently dropping the selection.

use bevy::prelude::*;

use crate::protocol::{
    definition::{ProtocolKind, ProtocolTuning},
    resources::protocol_active,
};

/// Tuning extracted from `ProtocolTuning::TierRegression` at activation time.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct TierRegressionConfig {
    /// How many tiers to regress on each boss clear.
    pub(crate) tiers_back: u32,
}

/// Inserted when the protocol is activated; consumed by the future node
/// sequencing rewrite. Kept as a marker so the scheduler has a place to
/// hang the eventual regression logic without touching this module's shape.
#[derive(Resource, Debug, Default)]
pub(crate) struct TierRegressionPending;

/// Activation entry: called by `dispatch_protocol_selection` on the frame
/// the player confirms a `TierRegression` selection.
pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::TierRegression { tiers_back } = *tuning else {
        warn!("tier_regression::activate called with non-TierRegression tuning");
        return;
    };
    commands.insert_resource(TierRegressionConfig { tiers_back });
    commands.insert_resource(TierRegressionPending);
}

/// Register runtime systems. For Wave 8 this is a single diagnostic system
/// — real regression logic lands with the node-sequencing refactor.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        warn_pending_tier_regression.run_if(protocol_active(ProtocolKind::TierRegression)),
    );
}

fn warn_pending_tier_regression(
    pending: Option<Res<TierRegressionPending>>,
    mut seen: Local<bool>,
) {
    if *seen {
        return;
    }
    if pending.is_some() {
        warn!(
            "TierRegression activated — awaiting node sequencing refactor; \
             regression behaviour is a no-op for now."
        );
        *seen = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::TestAppBuilder;

    #[test]
    fn activate_with_matching_tuning_inserts_config_and_marker() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &ProtocolTuning::TierRegression { tiers_back: 2 },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<TierRegressionConfig>();
        assert_eq!(cfg.tiers_back, 2);
        assert!(
            app.world()
                .get_resource::<TierRegressionPending>()
                .is_some(),
            "TierRegressionPending marker must be inserted"
        );
    }

    #[test]
    fn activate_with_mismatched_tuning_does_not_insert_resources() {
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

        assert!(
            app.world().get_resource::<TierRegressionConfig>().is_none(),
            "mismatched tuning must not insert config"
        );
        assert!(
            app.world()
                .get_resource::<TierRegressionPending>()
                .is_none(),
            "mismatched tuning must not insert pending marker"
        );
    }
}
