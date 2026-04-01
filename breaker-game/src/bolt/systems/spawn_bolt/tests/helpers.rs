use bevy::prelude::*;

use crate::{
    bolt::{messages::BoltSpawned, resources::BoltConfig},
    breaker::BreakerConfig,
    run::RunState,
};

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BoltSpawned>()
        .init_resource::<BoltConfig>()
        .init_resource::<BreakerConfig>()
        .init_resource::<RunState>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>();
    app
}
