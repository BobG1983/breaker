//! Screen plugin registration.

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use iyes_progress::prelude::*;

use crate::shared::{GameState, PlayfieldConfig, PlayingState};

use super::defaults::{
    BoltDefaults, BreakerDefaults, CellDefaults, PhysicsDefaults, PlayfieldDefaults,
};
use super::systems::{
    DefaultsCollection, cleanup_loading_screen, cleanup_on_node_exit, cleanup_on_run_end,
    seed_configs_from_defaults, spawn_loading_screen, start_game_on_input, update_loading_bar,
};

/// Plugin for screen state management.
///
/// Registers the game state machine, sub-states, asset loading pipeline,
/// and cleanup systems that run on state transitions.
pub struct ScreenPlugin;

impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        // Provide default PlayfieldConfig (domain plugins provide their own configs)
        app.init_resource::<PlayfieldConfig>();

        // State machine
        app.init_state::<GameState>();
        app.add_sub_state::<PlayingState>();

        // RON asset plugins — each type gets a unique extension to avoid
        // bevy_common_assets trying every loader on every file.
        app.add_plugins((
            RonAssetPlugin::<PlayfieldDefaults>::new(&["playfield.ron"]),
            RonAssetPlugin::<BoltDefaults>::new(&["bolt.ron"]),
            RonAssetPlugin::<BreakerDefaults>::new(&["breaker.ron"]),
            RonAssetPlugin::<CellDefaults>::new(&["cells.ron"]),
            RonAssetPlugin::<PhysicsDefaults>::new(&["physics.ron"]),
        ));

        // Progress plugin drives Loading → MainMenu transition.
        // Must be added BEFORE add_loading_state.
        app.add_plugins(
            ProgressPlugin::<GameState>::new()
                .with_state_transition(GameState::Loading, GameState::MainMenu),
        );

        // Asset loader: load all defaults RON files during Loading state
        app.add_loading_state(
            LoadingState::new(GameState::Loading).load_collection::<DefaultsCollection>(),
        );

        // Seeding system — tracked as progress entry, blocks transition until done
        app.add_systems(
            Update,
            seed_configs_from_defaults
                .track_progress::<GameState>()
                .run_if(in_state(GameState::Loading)),
        );

        // Loading screen UI
        app.add_systems(OnEnter(GameState::Loading), spawn_loading_screen);
        app.add_systems(
            Update,
            update_loading_bar.run_if(in_state(GameState::Loading)),
        );
        app.add_systems(OnExit(GameState::Loading), cleanup_loading_screen);

        // Game start
        app.add_systems(
            Update,
            start_game_on_input.run_if(in_state(GameState::MainMenu)),
        );

        // Cleanup
        app.add_systems(OnExit(GameState::Playing), cleanup_on_node_exit);
        app.add_systems(OnExit(GameState::RunEnd), cleanup_on_run_end);
    }
}

#[cfg(all(test, not(target_os = "macos")))]
mod tests {
    use super::*;

    // ScreenPlugin now requires AssetPlugin for RON loading,
    // which is not available with MinimalPlugins.
    // This test uses DefaultPlugins in headless mode.
    // On macOS, event loop creation fails in parallel test threads.
    #[test]
    fn plugin_builds() {
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
        app.add_plugins(ScreenPlugin);
        app.update();
    }
}
