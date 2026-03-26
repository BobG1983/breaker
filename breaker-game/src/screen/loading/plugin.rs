//! Loading screen plugin registration.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use super::{
    components::LoadingScreen,
    systems::{
        seed_breaker_registry, seed_cell_type_registry, seed_chip_registry, seed_difficulty_curve,
        seed_node_layout_registry, spawn_loading_screen, update_loading_bar,
    },
};
use crate::{screen::systems::cleanup_entities, shared::GameState};

/// Plugin for the loading screen — UI and config seeding.
pub(crate) struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                seed_cell_type_registry.track_progress::<GameState>(),
                seed_node_layout_registry.track_progress::<GameState>(),
                seed_breaker_registry.track_progress::<GameState>(),
                seed_chip_registry.track_progress::<GameState>(),
                seed_difficulty_curve.track_progress::<GameState>(),
            )
                .run_if(in_state(GameState::Loading)),
        )
        .add_systems(OnEnter(GameState::Loading), spawn_loading_screen)
        .add_systems(
            Update,
            update_loading_bar.run_if(in_state(GameState::Loading)),
        )
        .add_systems(
            OnExit(GameState::Loading),
            cleanup_entities::<LoadingScreen>,
        );
    }
}
