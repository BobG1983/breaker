//! Chips plugin registration.

use bevy::prelude::*;

use super::{inventory::ChipInventory, systems::dispatch_chip_effects};
use crate::state::types::ChipSelectState;

/// Plugin for the chips domain.
///
/// Owns chip application, stacking, and registry resources.
/// Registers per-effect observers and the thin dispatcher system.
pub(crate) struct ChipsPlugin;

impl Plugin for ChipsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChipInventory>()
            // ChipSelected message is registered by UiPlugin.
            // Only run during ChipSelect — messages can only arrive in that state.
            .add_systems(
                Update,
                dispatch_chip_effects.run_if(in_state(ChipSelectState::Selecting)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        use crate::state::{
            run::chip_select::messages::ChipSelected,
            types::{AppState, GamePhase, RunPhase},
        };
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<AppState>()
            .add_sub_state::<GamePhase>()
            .add_sub_state::<RunPhase>()
            .add_sub_state::<ChipSelectState>()
            // ChipSelected must be registered before ChipsPlugin (normally by UiPlugin)
            .add_message::<ChipSelected>()
            .add_plugins(ChipsPlugin)
            .update();
    }
}
