use bevy::prelude::*;

pub(super) use crate::effect::core::*;
use crate::effect::triggers::until::system::*;

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    register(&mut app);
    app
}

pub(super) use crate::shared::test_utils::tick;
