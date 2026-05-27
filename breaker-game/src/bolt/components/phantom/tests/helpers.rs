//! Shared test helpers for phantom component tests.

use bevy::prelude::*;

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app
}
