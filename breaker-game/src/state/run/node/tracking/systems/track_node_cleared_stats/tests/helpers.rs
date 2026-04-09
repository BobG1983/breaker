use bevy::prelude::*;

use crate::state::run::{
    definition::HighlightConfig,
    messages::HighlightTriggered,
    node::{
        messages::NodeCleared, resources::NodeTimer,
        tracking::systems::track_node_cleared_stats::system::track_node_cleared_stats,
    },
    resources::{HighlightTracker, NodeOutcome, RunStats},
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
        .init_resource::<NodeOutcome>()
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

pub(super) use crate::shared::test_utils::tick;
