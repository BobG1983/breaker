use bevy::prelude::*;

use super::super::super::shared::ScreenSize;
use crate::transition::messages::{TransitionOver, TransitionReady, TransitionRunComplete};

pub(super) fn effect_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<TransitionReady>();
    app.add_message::<TransitionRunComplete>();
    app.add_message::<TransitionOver>();
    app.insert_resource(ScreenSize::default());
    app
}
