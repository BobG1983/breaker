use bevy::prelude::*;

use crate::transition::messages::{TransitionOver, TransitionReady, TransitionRunComplete};

pub(super) fn effect_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<TransitionReady>();
    app.add_message::<TransitionRunComplete>();
    app.add_message::<TransitionOver>();
    // Spawn a Camera2d entity — target for TransitionEffect insertion.
    // No ScreenSize resource — shader handles screen dimensions internally.
    app.world_mut().spawn(Camera2d);
    app
}
