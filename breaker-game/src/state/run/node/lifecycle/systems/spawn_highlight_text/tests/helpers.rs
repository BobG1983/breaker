use bevy::prelude::*;

use crate::{
    prelude::*,
    shared::rng::FxRng,
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
        .with_resource::<FxRng>()
        .with_system(
            Update,
            (
                enqueue_highlights.before(spawn_highlight_text),
                spawn_highlight_text,
            ),
        )
        .build()
}

// B7 — test_app() registers FxRng and NOT GameRng
#[test]
fn test_app_registers_fx_rng_not_game_rng() {
    let app = test_app();
    assert!(
        app.world().contains_resource::<FxRng>(),
        "test_app must register FxRng"
    );
    assert!(
        !app.world().contains_resource::<GameRng>(),
        "test_app must NOT register GameRng"
    );
}
