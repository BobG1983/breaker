use bevy::prelude::*;

use super::super::system::reset_run_state;
use crate::{
    chips::inventory::ChipInventory,
    prelude::*,
    shared::{RunSeed, rng::EffectBaseSeed},
    state::run::resources::{HighlightTracker, NodeOutcome, NodeResult, RunProgress, TierConfig},
};

pub(super) fn test_app() -> App {
    TestAppBuilder::new()
        .insert_resource(NodeOutcome {
            node_index: 5,
            result: NodeResult::Won,
            ..default()
        })
        .with_resource::<GameRng>()
        .with_resource::<RunSeed>()
        .with_resource::<ChipInventory>()
        .with_resource::<RunStats>()
        .with_resource::<HighlightTracker>()
        .with_resource::<EffectBaseSeed>()
        .with_resource::<crate::shared::rng::ChipSelectCount>()
        .with_resource::<crate::mutators::protocols::resources::ActiveProtocols>()
        .with_resource::<crate::mutators::protocols::resources::ProtocolOffer>()
        .with_resource::<crate::mutators::protocols::resources::ProtocolOfferingCount>()
        .with_resource::<crate::mutators::hazards::resources::ActiveHazards>()
        .with_resource::<crate::mutators::protocols::greed::GreedStacks>()
        .with_resource::<crate::mutators::protocols::siphon::SiphonStreak>()
        .with_resource::<RunProgress>()
        .with_resource::<TierConfig>()
        .with_system(Update, reset_run_state)
        .build()
}
