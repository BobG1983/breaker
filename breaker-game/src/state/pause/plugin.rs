//! Pause menu plugin registration.

use bevy::prelude::*;

use super::{
    PauseMenuScreen,
    systems::{handle_pause_input, spawn_pause_menu, toggle_pause},
};
use crate::state::{cleanup::cleanup_entities, types::NodeState};

/// Run condition: `Time<Virtual>` is paused.
fn is_time_paused(time: Res<Time<Virtual>>) -> bool {
    time.is_paused()
}

/// Plugin for the pause menu overlay.
///
/// Uses `Time<Virtual>::pause()/unpause()` instead of a dedicated pause state.
/// `FixedUpdate` (gameplay) freezes when paused; `Update` (UI) keeps running.
pub(crate) struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // Toggle pause on Escape (only during active node gameplay)
            .add_systems(Update, toggle_pause.run_if(in_state(NodeState::Playing)))
            // Spawn pause menu when paused and no menu exists yet
            .add_systems(
                Update,
                spawn_pause_menu
                    .run_if(is_time_paused.and(not(any_with_component::<PauseMenuScreen>))),
            )
            // Handle pause menu input when paused
            .add_systems(Update, handle_pause_input.run_if(is_time_paused))
            // Cleanup pause menu when unpaused
            .add_systems(
                Update,
                cleanup_entities::<PauseMenuScreen>
                    .run_if(not(is_time_paused).and(any_with_component::<PauseMenuScreen>)),
            );
    }
}
