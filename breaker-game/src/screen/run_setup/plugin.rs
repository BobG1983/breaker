//! Run setup screen plugin registration.

use bevy::prelude::*;

use super::{
    RunSetupScreen,
    systems::{handle_run_setup_input, spawn_run_setup, update_run_setup_colors},
};
use crate::shared::GameState;

/// Plugin for the breaker selection screen.
pub(crate) struct RunSetupPlugin;

impl Plugin for RunSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::RunSetup), spawn_run_setup)
            .add_systems(
                Update,
                (handle_run_setup_input, update_run_setup_colors)
                    .chain()
                    .run_if(in_state(GameState::RunSetup)),
            )
            .add_systems(
                OnExit(GameState::RunSetup),
                crate::screen::systems::cleanup_entities::<RunSetupScreen>,
            );
    }
}
