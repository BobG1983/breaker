use bevy::prelude::*;

use super::super::system::*;
use crate::{
    breaker::{messages::BreakerSpawned, resources::BreakerConfig},
    shared::PlayfieldConfig,
};

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BreakerSpawned>()
        .init_resource::<BreakerConfig>()
        .init_resource::<PlayfieldConfig>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_systems(Startup, spawn_breaker);
    app
}
