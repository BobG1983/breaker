pub(super) use bevy::prelude::*;

pub(super) use crate::{effect::effects::shield::system::*, shared::PlayfieldConfig};

pub(super) fn test_world() -> World {
    let mut world = World::new();
    world.insert_resource(PlayfieldConfig::default());
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<ColorMaterial>>();
    world
}

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<crate::bolt::messages::BoltImpactWall>();
    app
}

pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}
