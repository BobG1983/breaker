use bevy::prelude::*;

use super::super::system::bolt_lost;
use crate::{
    bolt::{
        components::{BoltBaseSpeed, BoltRadius, BoltRespawnAngleSpread, BoltRespawnOffsetY},
        messages::BoltLost,
        resources::BoltConfig,
    },
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

pub(super) fn bolt_lost_bundle() -> (
    BoltBaseSpeed,
    BoltRadius,
    BoltRespawnOffsetY,
    BoltRespawnAngleSpread,
) {
    let config = BoltConfig::default();
    (
        BoltBaseSpeed(config.base_speed),
        BoltRadius(config.radius),
        BoltRespawnOffsetY(config.respawn_offset_y),
        BoltRespawnAngleSpread(config.respawn_angle_spread),
    )
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
