//! App construction — builds the Bevy [`App`] with all plugins.

use bevy::prelude::*;

use crate::game::Game;
use crate::shared::{PLAYFIELD_HEIGHT, PLAYFIELD_WIDTH};

/// Window width in pixels, derived from the playfield dimensions.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
const WINDOW_WIDTH: u32 = PLAYFIELD_WIDTH as u32;

/// Window height in pixels, derived from the playfield dimensions.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
const WINDOW_HEIGHT: u32 = PLAYFIELD_HEIGHT as u32;

/// Constructs and returns the configured Bevy [`App`].
///
/// Sets up the window, camera, and all game plugins via [`Game`].
pub fn build_app() -> App {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Brickbreaker".into(),
            resolution: bevy::window::WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            ..default()
        }),
        ..default()
    }));

    app.add_plugins(Game);
    app.add_systems(Startup, spawn_camera);

    app
}

/// Spawns the 2D camera centered on the playfield.
fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Game;

    fn headless_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.add_plugins(Game.build().disable::<crate::debug::DebugPlugin>());
        app.add_systems(Startup, spawn_camera);
        app
    }

    #[test]
    fn headless_app_builds() {
        let mut app = headless_app();
        app.update();
    }

    #[test]
    fn camera_spawns() {
        let mut app = headless_app();
        app.update();

        let camera_count = app
            .world_mut()
            .query::<&Camera2d>()
            .iter(app.world())
            .count();
        assert_eq!(camera_count, 1);
    }
}
