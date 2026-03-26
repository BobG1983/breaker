//! Run end screen plugin registration.

use bevy::prelude::*;

use super::{
    RunEndScreen,
    systems::{handle_run_end_input, spawn_run_end_screen},
};
use crate::{screen::systems::cleanup_entities, shared::GameState};

/// Plugin for the run-end screen.
pub(crate) struct RunEndPlugin;

impl Plugin for RunEndPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::RunEnd), spawn_run_end_screen)
            .add_systems(
                Update,
                handle_run_end_input.run_if(in_state(GameState::RunEnd)),
            )
            .add_systems(OnExit(GameState::RunEnd), cleanup_entities::<RunEndScreen>);
    }
}
