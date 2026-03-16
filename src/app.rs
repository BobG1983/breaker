//! App construction — builds the Bevy [`App`] with all plugins.

use bevy::{
    camera::ScalingMode, core_pipeline::tonemapping::Tonemapping, post_process::bloom::Bloom,
    prelude::*, window::PrimaryWindow,
};

use crate::{game::Game, shared::PlayfieldConfig};

/// Constructs and returns the configured Bevy [`App`].
///
/// Sets up the window, camera, and all game plugins via [`Game`].
pub fn build_app() -> App {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Brickbreaker".into(),
            ..default()
        }),
        ..default()
    }));

    app.insert_resource(ClearColor(PlayfieldConfig::default().background_color()));
    app.add_plugins(Game);
    app.add_systems(Startup, (spawn_camera, maximize_window));

    app
}

/// Spawns the 2D camera with a fixed 1920×1080 canvas and HDR bloom.
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 1920.0,
                min_height: 1080.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        Tonemapping::AcesFitted,
        Bloom::default(),
    ));
}

/// Maximizes the primary window on startup.
fn maximize_window(mut query: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = query.single_mut() {
        window.set_maximized(true);
    }
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
