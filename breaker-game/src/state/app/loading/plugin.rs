//! Loading screen plugin registration.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use super::{
    components::LoadingScreen,
    systems::{spawn_loading_screen, update_loading_bar},
};
use crate::{
    chips::systems::build_chip_catalog::build_chip_catalog, shared::GameState,
    state::cleanup::cleanup_entities,
};

/// Plugin for the loading screen — UI and config seeding.
pub(crate) struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            build_chip_catalog
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
            cleanup_entities::<LoadingScreen>,
        );
    }
}
