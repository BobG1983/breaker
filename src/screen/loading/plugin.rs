//! Loading screen plugin registration.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use super::{
    components::LoadingScreen,
    systems::{seed_configs_from_defaults, spawn_loading_screen, update_loading_bar},
};
use crate::shared::GameState;

/// Plugin for the loading screen — UI and config seeding.
pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            seed_configs_from_defaults
                .track_progress::<GameState>()
                .run_if(in_state(GameState::Loading)),
        )
        .add_systems(OnEnter(GameState::Loading), spawn_loading_screen)
        .add_systems(
            Update,
            update_loading_bar.run_if(in_state(GameState::Loading)),
        )
        .add_systems(
            OnExit(GameState::Loading),
            crate::screen::systems::cleanup_entities::<LoadingScreen>,
        );
    }
}
