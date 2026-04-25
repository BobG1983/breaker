//! Protocol domain plugin.

use bevy::{ecs::schedule::ApplyDeferred, prelude::*};

use super::{
    messages::ProtocolSelected,
    register,
    resources::{ActiveProtocols, ProtocolOffer, UnlockedProtocols},
    systems::{dispatch_protocol_selection, generate_protocol_offering},
};
use crate::{prelude::*, state::run::chip_select::sets::ChipSelectSystems};

/// Registers protocol resources, messages, and systems. Per-protocol modules
/// register themselves here from their own `register(app)` functions as they
/// land.
pub(crate) struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        register(app);
        app.init_resource::<ActiveProtocols>()
            .init_resource::<UnlockedProtocols>()
            .init_resource::<ProtocolOffer>()
            .init_resource::<crate::mutators::protocols::greed::GreedStacks>()
            .init_resource::<crate::mutators::protocols::siphon::SiphonStreak>()
            .init_resource::<crate::mutators::protocols::fission::FissionCounter>()
            .init_resource::<crate::mutators::protocols::reckless_dash::RecklessDashDoubledBolts>()
            .add_message::<ProtocolSelected>()
            .add_systems(
                OnEnter(ChipSelectState::Selecting),
                (generate_protocol_offering, ApplyDeferred)
                    .chain()
                    .after(ChipSelectSystems::GenerateOfferings)
                    .before(ChipSelectSystems::SpawnScreen),
            )
            .add_systems(
                Update,
                dispatch_protocol_selection
                    .after(ChipSelectSystems::HandleInput)
                    .run_if(in_state(ChipSelectState::Selecting)),
            );
    }
}
