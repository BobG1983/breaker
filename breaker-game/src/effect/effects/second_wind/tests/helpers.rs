//! Shared test helpers for `SecondWind` effect tests.

use bevy::prelude::*;

use crate::{bolt::messages::BoltImpactWall, effect::effects::second_wind::system::*};

pub(super) fn despawn_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BoltImpactWall>()
        .add_systems(FixedUpdate, despawn_second_wind_on_contact);
    app
}

pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

#[derive(Resource, Default)]
pub(super) struct TestBoltImpactWallMessages(pub(super) Vec<BoltImpactWall>);

pub(super) fn enqueue_bolt_impact_wall(
    msg_res: Res<TestBoltImpactWallMessages>,
    mut writer: MessageWriter<BoltImpactWall>,
) {
    for msg in &msg_res.0 {
        writer.write(msg.clone());
    }
}
