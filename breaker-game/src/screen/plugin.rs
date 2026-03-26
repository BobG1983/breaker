//! Screen plugin registration.

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use iyes_progress::prelude::*;
use rantzsoft_defaults::prelude::*;

use super::{
    chip_select::{ChipSelectDefaults, ChipSelectPlugin},
    loading::{LoadingPlugin, resources::DefaultsCollection},
    main_menu::{MainMenuDefaults, MainMenuPlugin},
    pause_menu::PauseMenuPlugin,
    run_end::RunEndPlugin,
    run_setup::RunSetupPlugin,
    systems::cleanup_entities,
};
use crate::{
    bolt::BoltDefaults,
    breaker::BreakerDefaults,
    cells::{CellDefaults, CellTypeDefinition},
    chips::{ChipDefinition, definition::ChipTemplate},
    effect::BreakerDefinition,
    input::InputDefaults,
    run::{NodeLayout, definition::DifficultyCurveDefaults},
    shared::{
        CleanupOnNodeExit, CleanupOnRunEnd, GameState, PlayfieldConfig, PlayfieldDefaults,
        PlayingState,
    },
    ui::TimerUiDefaults,
};

/// Plugin for screen state management.
///
/// Registers the game state machine, sub-states, asset loading pipeline,
/// and cleanup systems that run on state transitions.
pub(crate) struct ScreenPlugin;

impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayfieldConfig>()
            // State machine
            .init_state::<GameState>()
            .add_sub_state::<PlayingState>()
            // Defaults plugin — registers loaders, startup handles, and seed
            // systems for simple config types.
            .add_plugins(
                RantzDefaultsPluginBuilder::new()
                    .register_config::<PlayfieldDefaults>()
                    .register_config::<BoltDefaults>()
                    .register_config::<BreakerDefaults>()
                    .register_config::<CellDefaults>()
                    .register_config::<InputDefaults>()
                    .register_config::<MainMenuDefaults>()
                    .register_config::<TimerUiDefaults>()
                    .register_config::<ChipSelectDefaults>()
                    .build(),
            )
            // RON asset plugins for registry/collection types (not simple configs)
            .add_plugins((
                RonAssetPlugin::<CellTypeDefinition>::new(&["cell.ron"]),
                RonAssetPlugin::<NodeLayout>::new(&["node.ron"]),
                RonAssetPlugin::<BreakerDefinition>::new(&["bdef.ron"]),
                RonAssetPlugin::<ChipDefinition>::new(&["evolution.ron"]),
                RonAssetPlugin::<ChipTemplate>::new(&["chip.ron"]),
                RonAssetPlugin::<DifficultyCurveDefaults>::new(&["difficulty.ron"]),
            ))
            // Progress plugin drives Loading → MainMenu transition.
            // Must be added BEFORE add_loading_state.
            .add_plugins(
                ProgressPlugin::<GameState>::new()
                    .with_state_transition(GameState::Loading, GameState::MainMenu),
            )
            // Asset loader: load all defaults RON files during Loading state
            .add_loading_state(
                LoadingState::new(GameState::Loading).load_collection::<DefaultsCollection>(),
            )
            // Sub-domain plugins
            .add_plugins((
                LoadingPlugin,
                MainMenuPlugin,
                RunSetupPlugin,
                PauseMenuPlugin,
                ChipSelectPlugin,
                RunEndPlugin,
            ))
            // Cleanup
            .add_systems(
                OnExit(GameState::Playing),
                cleanup_entities::<CleanupOnNodeExit>,
            )
            .add_systems(
                OnExit(GameState::RunEnd),
                cleanup_entities::<CleanupOnRunEnd>,
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins((
                MinimalPlugins,
                bevy::state::app::StatesPlugin,
                bevy::asset::AssetPlugin::default(),
            ))
            .add_plugins(ScreenPlugin)
            .update();
    }
}
