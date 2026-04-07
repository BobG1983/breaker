use bevy::prelude::*;

use crate::{
    shared::{GameRng, PlayfieldConfig},
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
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<HighlightTriggered>()
        .init_resource::<HighlightConfig>()
        .init_resource::<PlayfieldConfig>()
        .init_resource::<GameRng>()
        .add_systems(
            Update,
            (
                enqueue_highlights.before(spawn_highlight_text),
                spawn_highlight_text,
            ),
        );
    app
}
