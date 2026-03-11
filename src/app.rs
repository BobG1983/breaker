//! App construction — builds the Bevy [`App`] with all plugins.

use bevy::{core_pipeline::tonemapping::Tonemapping, post_process::bloom::Bloom, prelude::*};

use crate::{game::Game, shared::PlayfieldConfig};

/// Constructs and returns the configured Bevy [`App`].
///
/// Sets up the window, camera, and all game plugins via [`Game`].
pub fn build_app() -> App {
    let mut app = App::new();

    let playfield = PlayfieldConfig::default();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let window_width = playfield.width as u32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let window_height = playfield.height as u32;

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Brickbreaker".into(),
            resolution: bevy::window::WindowResolution::new(window_width, window_height),
            ..default()
        }),
        ..default()
    }));

    app.insert_resource(ClearColor(playfield.background_color()));
    app.add_plugins(Game);
    app.add_systems(Startup, spawn_camera);

    app
}

/// Spawns the 2D camera centered on the playfield with HDR bloom.
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera::default(),
        Tonemapping::AcesFitted,
        Bloom::default(),
    ));
}

#[cfg(all(test, not(target_os = "macos")))]
mod tests {
    use super::*;
    use crate::game::Game;

    fn headless_app() -> App {
        let mut app = App::new();
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    ..default()
                })
                .set(bevy::asset::AssetPlugin {
                    file_path: "assets".into(),
                    ..default()
                }),
        );
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
            .query::<(&Camera2d, &Camera)>()
            .iter(app.world())
            .count();
        assert_eq!(camera_count, 1);
    }
}
