use bevy::prelude::*;

use crate::{
    shared::test_utils::TestAppBuilder,
    state::run::{
        definition::HighlightConfig,
        messages::HighlightTriggered,
        node::{
            messages::NodeCleared, resources::NodeTimer,
            tracking::systems::track_node_cleared_stats::system::track_node_cleared_stats,
        },
        resources::{HighlightTracker, NodeOutcome, RunStats},
    },
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
    TestAppBuilder::new()
        .with_message::<NodeCleared>()
        .with_message::<HighlightTriggered>()
        .with_resource::<RunStats>()
        .with_resource::<HighlightTracker>()
        .with_resource::<NodeOutcome>()
        .insert_resource(HighlightConfig::default())
        .insert_resource(NodeTimer {
            remaining: 15.0,
            total:     30.0,
        })
        .with_system(
            FixedUpdate,
            (enqueue_node_cleared, track_node_cleared_stats).chain(),
        )
        .build()
}

pub(super) use crate::shared::test_utils::tick;
