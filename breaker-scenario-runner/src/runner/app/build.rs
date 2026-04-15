//! Bevy `App` construction for scenario runs — headless and visual variants.

use std::time::Duration;

use bevy::{log::LogPlugin, prelude::*, time::TimeUpdateStrategy};
use breaker::game::Game;

use super::window::{apply_tile_layout, sync_ui_scale};
use crate::{log_capture::scenario_log_layer_factory, runner::tiling};

/// Bevy's default fixed timestep frequency (Hz).
const FIXED_TIMESTEP_HZ: f64 = 64.0;

/// Speed multiplier for visual mode — each rendered frame advances virtual
/// time by this many fixed timesteps.
const VISUAL_SPEED_MULTIPLIER: f64 = 10.0;

/// Scenario runner log plugin — captures `WARN`-and-above logs via [`scenario_log_layer_factory`].
fn scenario_log_plugin() -> LogPlugin {
    LogPlugin {
        level: bevy::log::Level::WARN,
        filter: "warn,bevy_egui=error".to_owned(),
        custom_layer: scenario_log_layer_factory,
        ..default()
    }
}

/// Builds a Bevy app configured for scenario running.
///
/// In headless mode, uses [`MinimalPlugins`] with only the specific Bevy
/// plugins the game needs (states, assets, input). This avoids pulling in the
/// full render pipeline, winit event loop, and GPU initialization — none of
/// which are needed when running scenarios at CPU speed on CI. Asset types
/// for headless spawn systems (`Mesh`, `ColorMaterial`, `Font`) are registered by
/// [`Game::headless()`].
///
/// In visual mode, uses [`DefaultPlugins`] for full windowed rendering.
///
/// On the first run (headless or visual), installs `LogPlugin` with a custom
/// tracing layer. On subsequent runs, skips `LogPlugin` (headless: omits it;
/// visual: disables it from `DefaultPlugins`) to avoid the "global logger
/// already set" error — the shared `LogBuffer` is inserted by `run_scenario`
/// instead.
pub(crate) fn build_app(headless: bool, first_run: bool) -> App {
    let mut app = App::new();

    // Point to the game crate's assets directory so scenarios
    // load real RON config files rather than code defaults.
    let game_asset_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../breaker-game/assets").to_owned();

    if headless {
        // Minimal plugin set — no render pipeline, no window, no GPU.
        // Asset types needed by game spawn systems (Mesh, ColorMaterial, Font)
        // are registered by HeadlessAssetsPlugin inside Game::headless().
        app.add_plugins((
            MinimalPlugins,
            bevy::state::app::StatesPlugin,
            bevy::asset::AssetPlugin {
                file_path: game_asset_path,
                ..default()
            },
            bevy::input::InputPlugin,
        ));

        if first_run {
            app.add_plugins(scenario_log_plugin());
        }

        // Advance simulated time by exactly one fixed timestep per Update tick.
        // Without this, Time<Fixed> accumulates based on real wall-clock elapsed
        // time, so a 20k-frame scenario would take ~5 minutes. With ManualDuration,
        // each Update tick instantly advances virtual time by 1/64 s, and all
        // Fixed ticks execute in sequence at CPU speed.
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / FIXED_TIMESTEP_HZ,
        )))
        .add_plugins(Game::headless());
    } else {
        // Visual mode — full DefaultPlugins for windowed rendering.
        let window = Window {
            title: "Scenario Runner".into(),
            ..default()
        };

        let mut defaults = DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(window),
                ..default()
            })
            .set(bevy::asset::AssetPlugin {
                file_path: game_asset_path,
                ..default()
            });

        if first_run {
            defaults = defaults.set(scenario_log_plugin());
        } else {
            defaults = defaults.disable::<LogPlugin>();
        }

        app.add_plugins(defaults)
            // Visual mode runs at 10x speed to avoid 5+ minute waits for
            // 20,000-frame scenarios. Each Update tick advances virtual time
            // by 10 fixed timesteps (10/64 s).
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                VISUAL_SPEED_MULTIPLIER / FIXED_TIMESTEP_HZ,
            )))
            .add_plugins(Game::default())
            .add_systems(Update, (apply_tile_layout, sync_ui_scale));

        // Insert TileConfig resource if environment variables are set.
        if let Some(tile_config) = tiling::read_tile_config() {
            app.insert_resource(tile_config);
        }
    }

    app
}
