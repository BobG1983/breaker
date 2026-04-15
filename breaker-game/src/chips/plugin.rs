//! Chips plugin registration.

use bevy::prelude::*;

use super::{inventory::ChipInventory, systems::dispatch_chip_effects};
use crate::prelude::*;

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
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            // ChipSelected must be registered before ChipsPlugin (normally by UiPlugin)
            .with_message::<ChipSelected>()
            .build();
        app.add_plugins(ChipsPlugin);
        app.update();
    }
}
