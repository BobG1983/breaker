//! State plugin registration.

use bevy::prelude::*;
use iyes_progress::prelude::*;
use rantzsoft_defaults::prelude::*;

use super::{
    app::loading::LoadingPlugin,
    cleanup::cleanup_entities,
    menu::{
        main::{MainMenuDefaults, MainMenuPlugin},
        start_game::RunSetupPlugin,
    },
    pause::PauseMenuPlugin,
    run::{
        RunPlugin,
        chip_select::{ChipSelectDefaults, ChipSelectPlugin},
        node::hud::{HudPlugin, TimerUiDefaults},
        resources::DifficultyCurveDefaults,
        run_end::RunEndPlugin,
    },
};
use crate::{
    cells::CellDefaults,
    input::InputDefaults,
    shared::{
        CleanupOnNodeExit, CleanupOnRunEnd, GameState, PlayfieldConfig, PlayfieldDefaults,
        PlayingState,
    },
};

/// Plugin for state lifecycle management.
///
/// Registers the game state machine, sub-states, asset loading pipeline,
/// and cleanup systems that run on state transitions.
pub(crate) struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayfieldConfig>()
            // State machine
            .init_state::<GameState>()
            .add_sub_state::<PlayingState>()
            // Defaults plugin — registers loaders, startup handles, and seed
            // systems for simple config types.
            .add_plugins(
                RantzDefaultsPluginBuilder::<GameState>::new(GameState::Loading)
                    .add_config::<PlayfieldDefaults>()
                    .add_config::<CellDefaults>()
                    .add_config::<InputDefaults>()
                    .add_config::<MainMenuDefaults>()
                    .add_config::<TimerUiDefaults>()
                    .add_config::<ChipSelectDefaults>()
                    .add_config::<DifficultyCurveDefaults>()
                    // Registries
                    .add_registry::<crate::cells::CellTypeRegistry>()
                    .add_registry::<crate::breaker::BreakerRegistry>()
                    .add_registry::<crate::bolt::BoltRegistry>()
                    .add_registry::<crate::state::run::NodeLayoutRegistry>()
                    .add_registry::<crate::chips::ChipTemplateRegistry>()
                    .add_registry::<crate::chips::EvolutionTemplateRegistry>()
                    .add_registry::<crate::walls::WallRegistry>()
                    .build(),
            )
            // Progress plugin drives Loading → MainMenu transition.
            .add_plugins(
                ProgressPlugin::<GameState>::new()
                    .with_state_transition(GameState::Loading, GameState::MainMenu),
            )
            // Sub-domain plugins
            .add_plugins((
                LoadingPlugin,
                MainMenuPlugin,
                RunSetupPlugin,
                PauseMenuPlugin,
                ChipSelectPlugin,
                RunEndPlugin,
                HudPlugin,
                RunPlugin,
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
            .add_plugins(StatePlugin)
            .update();
    }
}
