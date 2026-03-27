//! Chips plugin registration.

use bevy::prelude::*;

use super::{inventory::ChipInventory, systems::dispatch_chip_effects};
use crate::shared::GameState;

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
                dispatch_chip_effects.run_if(in_state(GameState::ChipSelect)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        use crate::{shared::GameState, ui::messages::ChipSelected};
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<GameState>()
            // ChipSelected must be registered before ChipsPlugin (normally by UiPlugin)
            .add_message::<ChipSelected>()
            .add_plugins(ChipsPlugin)
            .update();
    }

    // =========================================================================
    // B12d Behavior 22: ChipsPlugin does NOT register passive handler observers
    // =========================================================================

    /// After B12d, passive handlers (`handle_piercing`, `handle_damage_boost`, etc.)
    /// are moved to `EffectPlugin`. `ChipsPlugin` should NOT register them.
    /// Triggering `PiercingApplied` with only `ChipsPlugin` should NOT insert
    /// `Piercing` on a bolt.
    #[test]
    fn chips_plugin_does_not_register_passive_handler_observers() {
        use crate::{
            bolt::components::Bolt, chips::components::Piercing,
            effect::typed_events::PiercingApplied, shared::GameState, ui::messages::ChipSelected,
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<GameState>()
            .add_message::<ChipSelected>()
            .add_plugins(ChipsPlugin);

        let bolt = app.world_mut().spawn(Bolt).id();

        app.world_mut().commands().trigger(PiercingApplied {
            per_stack: 1,
            max_stacks: 3,
        });
        app.world_mut().flush();

        assert!(
            app.world().entity(bolt).get::<Piercing>().is_none(),
            "ChipsPlugin should NOT register handle_piercing — bolt should NOT have Piercing component. \
             Passive handler observers are now registered by EffectPlugin."
        );
    }
}
