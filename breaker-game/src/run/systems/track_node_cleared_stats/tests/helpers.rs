use bevy::prelude::*;

use super::super::system::track_node_cleared_stats;
use crate::run::{
    definition::HighlightConfig,
    messages::HighlightTriggered,
    node::{messages::NodeCleared, resources::NodeTimer},
    resources::{HighlightTracker, RunState, RunStats},
};

#[derive(Resource)]
pub(super) struct TestNodeCleared(pub bool);

pub(super) fn enqueue_node_cleared(
    msg_res: Res<TestNodeCleared>,
    mut writer: MessageWriter<NodeCleared>,
) {
    if msg_res.0 {
        writer.write(NodeCleared);
    }
}

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<NodeCleared>()
        .add_message::<HighlightTriggered>()
        .init_resource::<RunStats>()
        .init_resource::<HighlightTracker>()
        .init_resource::<RunState>()
        .insert_resource(HighlightConfig::default())
        .insert_resource(NodeTimer {
            remaining: 15.0,
            total: 30.0,
        })
        .add_systems(
            FixedUpdate,
            (enqueue_node_cleared, track_node_cleared_stats).chain(),
        );
    app
}

pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}
