//! Pause menu plugin registration.

use bevy::prelude::*;
use rantzsoft_lifecycle::ActiveTransition;

use super::{
    PauseMenuScreen,
    systems::{handle_pause_input, spawn_pause_menu, toggle_pause, update_pause_menu_colors},
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
        // Pause systems are gated on NOT ActiveTransition — transitions
        // pause Time<Virtual> but should not trigger the pause menu.
        let not_in_transition = not(resource_exists::<ActiveTransition>);

        app
            // Toggle pause on Escape (only during active node gameplay, not during transitions)
            .add_systems(
                Update,
                toggle_pause.run_if(in_state(NodeState::Playing).and(not_in_transition.clone())),
            )
            // Spawn pause menu when paused during active gameplay (not during transitions or loading)
            .add_systems(
                Update,
                spawn_pause_menu.run_if(
                    in_state(NodeState::Playing)
                        .and(is_time_paused)
                        .and(not(any_with_component::<PauseMenuScreen>))
                        .and(not_in_transition.clone()),
                ),
            )
            // Handle pause menu input when paused AND menu exists (not during transitions)
            .add_systems(
                Update,
                handle_pause_input.run_if(
                    is_time_paused
                        .and(any_with_component::<PauseMenuScreen>)
                        .and(not_in_transition.clone()),
                ),
            )
            // Update pause menu item colors based on current selection
            // Must run after handle_pause_input so color reflects this frame's selection.
            .add_systems(
                Update,
                update_pause_menu_colors.after(handle_pause_input).run_if(
                    is_time_paused
                        .and(any_with_component::<PauseMenuScreen>)
                        .and(not_in_transition.clone()),
                ),
            )
            // Cleanup pause menu when unpaused
            .add_systems(
                Update,
                cleanup_entities::<PauseMenuScreen>
                    .run_if(not(is_time_paused).and(any_with_component::<PauseMenuScreen>)),
            );
    }
}
