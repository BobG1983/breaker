//! Screen plugin registration.

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use iyes_progress::prelude::*;

use crate::{
    bolt::BoltDefaults,
    breaker::BreakerDefaults,
    cells::CellDefaults,
    physics::PhysicsDefaults,
    shared::{
        CleanupOnNodeExit, CleanupOnRunEnd, GameState, PlayfieldConfig, PlayfieldDefaults,
        PlayingState,
    },
};

use super::{
    components::{LoadingScreen, MainMenuScreen},
    resources::{DefaultsCollection, MainMenuDefaults},
    systems::{
        cleanup_entities, cleanup_main_menu, handle_main_menu_input, seed_configs_from_defaults,
        spawn_loading_screen, spawn_main_menu, update_loading_bar, update_menu_colors,
    },
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
            RonAssetPlugin::<MainMenuDefaults>::new(&["mainmenu.ron"]),
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
        app.add_systems(
            OnExit(GameState::Loading),
            cleanup_entities::<LoadingScreen>,
        );

        // Main menu
        app.add_systems(OnEnter(GameState::MainMenu), spawn_main_menu);
        app.add_systems(
            Update,
            (handle_main_menu_input, update_menu_colors)
                .chain()
                .run_if(in_state(GameState::MainMenu)),
        );
        app.add_systems(
            OnExit(GameState::MainMenu),
            (cleanup_entities::<MainMenuScreen>, cleanup_main_menu),
        );

        // Cleanup
        app.add_systems(
            OnExit(GameState::Playing),
            cleanup_entities::<CleanupOnNodeExit>,
        );
        app.add_systems(
            OnExit(GameState::RunEnd),
            cleanup_entities::<CleanupOnRunEnd>,
        );
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
