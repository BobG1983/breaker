//! Loading screen plugin registration.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use super::{
    components::LoadingScreen,
    systems::{spawn_loading_screen, update_loading_bar},
};
use crate::{
    chips::systems::build_chip_catalog::build_chip_catalog,
    state::{cleanup::cleanup_entities, types::AppState},
};

/// Plugin for the loading screen — UI and config seeding.
pub(crate) struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            build_chip_catalog
                .track_progress::<AppState>()
                .run_if(in_state(AppState::Loading)),
        )
        .add_systems(OnEnter(AppState::Loading), spawn_loading_screen)
        .add_systems(
            Update,
            update_loading_bar.run_if(in_state(AppState::Loading)),
        )
        .add_systems(OnExit(AppState::Loading), cleanup_entities::<LoadingScreen>);
    }
}
