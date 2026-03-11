//! Screen plugin registration.

use bevy::prelude::*;

use crate::shared::{GameState, PlayingState};

use super::systems::{cleanup_on_node_exit, cleanup_on_run_end, finish_loading};

/// Plugin for screen state management.
///
/// Registers the game state machine, sub-states, and cleanup systems
/// that run on state transitions.
pub struct ScreenPlugin;

impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>();
        app.add_sub_state::<PlayingState>();
        app.add_systems(OnEnter(GameState::Loading), finish_loading);
        app.add_systems(OnExit(GameState::Playing), cleanup_on_node_exit);
        app.add_systems(OnExit(GameState::RunEnd), cleanup_on_run_end);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .add_plugins(ScreenPlugin)
            .update();
    }
}
