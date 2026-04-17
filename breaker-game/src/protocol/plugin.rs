//! Protocol domain plugin.

use bevy::prelude::*;

use super::{
    messages::ProtocolSelected,
    resources::{ActiveProtocols, ProtocolOffer, UnlockedProtocols},
};

/// Registers protocol resources and messages. Per-protocol modules register
/// themselves here from their own `register(app)` functions as they land.
pub(crate) struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveProtocols>()
            .init_resource::<UnlockedProtocols>()
            .init_resource::<ProtocolOffer>()
            .add_message::<ProtocolSelected>();
    }
}
