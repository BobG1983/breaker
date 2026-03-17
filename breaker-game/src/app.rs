//! App construction — builds the Bevy [`App`] with all plugins.

use bevy::{
    camera::ScalingMode,
    core_pipeline::tonemapping::Tonemapping,
    log::{BoxedLayer, LogPlugin},
    post_process::bloom::Bloom,
    prelude::*,
    window::PrimaryWindow,
};

use crate::{game::Game, shared::PlayfieldConfig};

/// Applies dev-only CLI flags parsed from `std::env::args()`.
///
/// Supported flags (all dev-only, not shown in any `--help` output):
/// - `--record [<layout_name>]` — enable input recording, optional layout filter
///
/// `--log` and `--log-level` are parsed inside [`build_app`] because they must
/// configure `LogPlugin` before it is registered.
///
/// Called from `main.rs` after `build_app`, before `app.run()`.
#[cfg(feature = "dev")]
pub fn apply_dev_flags(app: &mut App) {
    use crate::debug::recording::RecordingConfig;

    let args: Vec<String> = std::env::args().collect();

    // --record [<layout_name>]
    if let Some(idx) = args.iter().position(|a| a == "--record") {
        let level_filter = args.get(idx + 1).filter(|a| !a.starts_with('-')).cloned();
        app.insert_resource(RecordingConfig {
            enabled: true,
            level_filter,
        });
    }
}

/// Reads `--log` and `--log-level` dev flags from process args.
///
/// Returns `(filter_string, file_logging_enabled)`.
/// In non-dev builds (or when no flags are present) returns the provided default.
fn dev_log_config(default_filter: &str) -> (String, bool) {
    #[cfg(feature = "dev")]
    {
        let args: Vec<String> = std::env::args().collect();

        // --log false  →  disable file logging
        let file_enabled = args
            .iter()
            .position(|a| a == "--log")
            .and_then(|i| args.get(i + 1))
            .is_none_or(|v| v != "false");

        // --log-level <level>  →  override filter
        let filter = args
            .iter()
            .position(|a| a == "--log-level")
            .and_then(|i| args.get(i + 1))
            .map_or_else(
                || default_filter.to_owned(),
                |level| format!("breaker={level},bevy=warn"),
            );

        (filter, file_enabled)
    }

    #[cfg(not(feature = "dev"))]
    (default_filter.to_owned(), true)
}

/// `LogPlugin::custom_layer` factory — attaches a daily rolling file appender.
///
/// Writes to `logs/breaker.log` (rolling daily). Returns `None` if the `logs/`
/// directory cannot be created (non-fatal — falls back to console logging only).
fn file_log_layer(_app: &mut App) -> Option<BoxedLayer> {
    use tracing_appender::rolling;
    use tracing_subscriber::Layer;

    std::fs::create_dir_all("logs").ok()?;
    let file_appender = rolling::daily("logs", "breaker.log");
    Some(
        tracing_subscriber::fmt::layer()
            .with_writer(file_appender)
            .with_ansi(false)
            .boxed(),
    )
}

/// Constructs and returns the configured Bevy [`App`].
///
/// Sets up the window, camera, and all game plugins via [`Game`].
///
/// In dev builds, also reads `--log <true|false>` and `--log-level <level>`
/// from `std::env::args()` to configure `LogPlugin` before it is registered.
pub fn build_app() -> App {
    let mut app = App::new();

    let default_filter = if cfg!(debug_assertions) {
        "breaker=debug,bevy=warn"
    } else {
        "breaker=warn,bevy=error"
    };

    let (log_filter, use_file_log) = dev_log_config(default_filter);

    let custom_layer: fn(&mut App) -> Option<BoxedLayer> = if use_file_log {
        file_log_layer
    } else {
        |_| None
    };

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Brickbreaker".into(),
                    ..default()
                }),
                ..default()
            })
            .set(LogPlugin {
                filter: log_filter,
                custom_layer,
                ..default()
            }),
    );

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
