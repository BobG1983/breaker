//! Shared test helpers for `spawn_walls` tests.

use bevy::prelude::*;

use super::super::*;
use crate::{shared::PlayfieldConfig, wall::messages::WallsSpawned};

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<WallsSpawned>()
        .init_resource::<PlayfieldConfig>()
        .add_systems(Update, spawn_walls);
    app
}
