//! Run end screen plugin registration.

use bevy::prelude::*;

use super::{
    RunEndScreen,
    systems::{handle_run_end_input, spawn_run_end_screen},
};
use crate::state::{cleanup::cleanup_entities, types::RunEndState};

/// Plugin for the run-end screen.
pub(crate) struct RunEndPlugin;

impl Plugin for RunEndPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(RunEndState::Active), spawn_run_end_screen)
            .add_systems(
                Update,
                handle_run_end_input.run_if(in_state(RunEndState::Active)),
            )
            .add_systems(
                OnExit(RunEndState::Active),
                cleanup_entities::<RunEndScreen>,
            );
    }
}
