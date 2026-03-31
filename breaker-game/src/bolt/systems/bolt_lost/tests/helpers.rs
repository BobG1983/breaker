use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::super::system::bolt_lost;
use crate::{
    bolt::{components::Bolt, messages::BoltLost, resources::BoltConfig},
    shared::{GameRng, PlayfieldConfig},
};

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<PlayfieldConfig>()
        .init_resource::<GameRng>()
        .add_message::<BoltLost>()
        .add_systems(FixedUpdate, bolt_lost);
    app
}

pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

/// Spawns a bolt at the given position with the given velocity using the builder.
pub(super) fn spawn_bolt(app: &mut App, pos: Vec2, vel: Vec2) -> Entity {
    Bolt::builder()
        .at_position(pos)
        .config(&BoltConfig::default())
        .with_velocity(Velocity2D(vel))
        .primary()
        .spawn(app.world_mut())
}

/// Spawns a bolt with a custom `BoltConfig` (e.g. zero `respawn_angle_spread`).
pub(super) fn spawn_bolt_with_config(
    app: &mut App,
    pos: Vec2,
    vel: Vec2,
    config: &BoltConfig,
) -> Entity {
    Bolt::builder()
        .at_position(pos)
        .config(config)
        .with_velocity(Velocity2D(vel))
        .primary()
        .spawn(app.world_mut())
}

#[derive(Resource, Default)]
pub(super) struct BoltLostCount(pub(super) u32);

pub(super) fn count_bolt_lost(
    mut reader: MessageReader<BoltLost>,
    mut count: ResMut<BoltLostCount>,
) {
    for _msg in reader.read() {
        count.0 += 1;
    }
}

#[derive(Resource, Default)]
pub(super) struct CapturedRequestBoltDestroyed(
    pub(super) Vec<crate::bolt::messages::RequestBoltDestroyed>,
);

pub(super) fn capture_request_bolt_destroyed(
    mut reader: MessageReader<crate::bolt::messages::RequestBoltDestroyed>,
    mut captured: ResMut<CapturedRequestBoltDestroyed>,
) {
    for msg in reader.read() {
        captured.0.push(msg.clone());
    }
}
