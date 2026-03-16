//! Loading screen plugin registration.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use super::{
    components::LoadingScreen,
    systems::{
        seed_archetype_registry, seed_bolt_config, seed_breaker_config, seed_cell_config,
        seed_cell_type_registry, seed_input_config, seed_main_menu_config,
        seed_node_layout_registry, seed_playfield_config, seed_timer_ui_config,
        seed_upgrade_select_config, spawn_loading_screen, update_loading_bar,
    },
};
use crate::shared::GameState;

/// Plugin for the loading screen — UI and config seeding.
pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                seed_playfield_config.track_progress::<GameState>(),
                seed_bolt_config.track_progress::<GameState>(),
                seed_breaker_config.track_progress::<GameState>(),
                seed_cell_config.track_progress::<GameState>(),
                seed_input_config.track_progress::<GameState>(),
                seed_main_menu_config.track_progress::<GameState>(),
                seed_cell_type_registry.track_progress::<GameState>(),
                seed_node_layout_registry.track_progress::<GameState>(),
                seed_timer_ui_config.track_progress::<GameState>(),
                seed_archetype_registry.track_progress::<GameState>(),
                seed_upgrade_select_config.track_progress::<GameState>(),
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
            crate::screen::systems::cleanup_entities::<LoadingScreen>,
        );
    }
}
