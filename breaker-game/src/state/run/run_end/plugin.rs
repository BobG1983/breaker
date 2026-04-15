//! Run end screen plugin registration.

use bevy::prelude::*;
use rantzsoft_stateflow::{Route, RoutingTableAppExt, cleanup_on_exit};

use super::{
    RunEndScreen,
    systems::{handle_run_end_input, spawn_run_end_screen},
};
use crate::{prelude::*, state::cleanup::cleanup_entities};

/// Plugin for the run-end screen.
pub(crate) struct RunEndPlugin;

impl Plugin for RunEndPlugin {
    fn build(&self, app: &mut App) {
        // RunEndState routes — run end lifecycle
        app.add_route(
            Route::from(RunEndState::Loading)
                .to(RunEndState::AnimateIn)
                .when(|_| true),
        );
        app.add_route(
            Route::from(RunEndState::AnimateIn)
                .to(RunEndState::Active)
                .when(|_| true),
        );
        // Active → AnimateOut: message-triggered (handle_run_end_input)
        app.add_route(Route::from(RunEndState::Active).to(RunEndState::AnimateOut));
        app.add_route(
            Route::from(RunEndState::AnimateOut)
                .to(RunEndState::Teardown)
                .when(|_| true),
        );
        app.add_systems(
            OnEnter(RunEndState::Teardown),
            cleanup_on_exit::<RunEndState>,
        );

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
