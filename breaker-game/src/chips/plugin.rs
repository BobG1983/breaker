//! Chips plugin registration.

use bevy::prelude::*;

use super::{effects::*, systems::apply_chip_effect};
use crate::shared::GameState;

/// Plugin for the chips domain.
///
/// Owns chip application, stacking, and registry resources.
/// Registers per-effect observers and the thin dispatcher system.
pub(crate) struct ChipsPlugin;

impl Plugin for ChipsPlugin {
    fn build(&self, app: &mut App) {
        // ChipSelected message is registered by UiPlugin.
        // Only run during ChipSelect — messages can only arrive in that state.
        app.add_systems(
            Update,
            apply_chip_effect.run_if(in_state(GameState::ChipSelect)),
        )
        .add_observer(handle_piercing)
        .add_observer(handle_damage_boost)
        .add_observer(handle_bolt_speed_boost)
        .add_observer(handle_chain_hit)
        .add_observer(handle_bolt_size_boost)
        .add_observer(handle_width_boost)
        .add_observer(handle_breaker_speed_boost)
        .add_observer(handle_bump_force_boost)
        .add_observer(handle_tilt_control_boost);
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
}
