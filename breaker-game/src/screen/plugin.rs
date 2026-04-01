//! Screen plugin registration.

use bevy::prelude::*;
use iyes_progress::prelude::*;
use rantzsoft_defaults::prelude::*;

use super::{
    chip_select::{ChipSelectDefaults, ChipSelectPlugin},
    loading::LoadingPlugin,
    main_menu::{MainMenuDefaults, MainMenuPlugin},
    pause_menu::PauseMenuPlugin,
    run_end::RunEndPlugin,
    run_setup::RunSetupPlugin,
    systems::cleanup_entities,
};
use crate::{
    bolt::BoltDefaults,
    breaker::BreakerDefaults,
    cells::CellDefaults,
    input::InputDefaults,
    run::resources::DifficultyCurveDefaults,
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
                RantzDefaultsPluginBuilder::<GameState>::new(GameState::Loading)
                    .add_config::<PlayfieldDefaults>()
                    .add_config::<BoltDefaults>()
                    .add_config::<BreakerDefaults>()
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
                    .add_registry::<crate::run::NodeLayoutRegistry>()
                    .add_registry::<crate::chips::ChipTemplateRegistry>()
                    .add_registry::<crate::chips::EvolutionTemplateRegistry>()
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
