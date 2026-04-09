use bevy::prelude::*;

use crate::{
    shared::{GameRng, PlayfieldConfig, test_utils::TestAppBuilder},
    state::run::{
        definition::HighlightConfig, messages::HighlightTriggered,
        node::lifecycle::systems::spawn_highlight_text::system::spawn_highlight_text,
    },
};

#[derive(Resource)]
pub(super) struct TestHighlightMsg(pub Vec<HighlightTriggered>);

pub(super) fn enqueue_highlights(
    msg_res: Res<TestHighlightMsg>,
    mut writer: MessageWriter<HighlightTriggered>,
) {
    for msg in &msg_res.0 {
        writer.write(msg.clone());
    }
}

pub(super) fn test_app() -> App {
    TestAppBuilder::new()
        .with_message::<HighlightTriggered>()
        .with_resource::<HighlightConfig>()
        .with_resource::<PlayfieldConfig>()
        .with_resource::<GameRng>()
        .with_system(
            Update,
            (
                enqueue_highlights.before(spawn_highlight_text),
                spawn_highlight_text,
            ),
        )
        .build()
}
