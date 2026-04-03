//! State plugin registration.

use bevy::prelude::*;
use iyes_progress::prelude::*;
use rantzsoft_defaults::prelude::*;

use super::{
    app::loading::LoadingPlugin,
    cleanup::{cleanup_entities, cleanup_on_exit},
    menu::{
        main::{MainMenuDefaults, MainMenuPlugin},
        start_game::RunSetupPlugin,
    },
    pause::PauseMenuPlugin,
    routing,
    run::{
        RunPlugin,
        chip_select::{ChipSelectDefaults, ChipSelectPlugin},
        node::hud::TimerUiDefaults,
        resources::DifficultyCurveDefaults,
        run_end::RunEndPlugin,
    },
};
use crate::{
    cells::CellDefaults,
    input::InputDefaults,
    shared::{CleanupOnNodeExit, CleanupOnRunEnd, PlayfieldConfig, PlayfieldDefaults},
    state::types::{
        AppState, ChipSelectState, GameState, MenuState, NodeState, RunEndState, RunPhase,
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
            // Hierarchical state machine
            .init_state::<AppState>()
            .add_sub_state::<GameState>()
            .add_sub_state::<MenuState>()
            .add_sub_state::<RunPhase>()
            .add_sub_state::<NodeState>()
            .add_sub_state::<ChipSelectState>()
            .add_sub_state::<RunEndState>()
            // Defaults + progress
            .add_plugins(defaults_plugin())
            .add_plugins(
                ProgressPlugin::<AppState>::new()
                    .with_state_transition(AppState::Loading, AppState::Game),
            )
            // Sub-domain plugins
            .add_plugins((
                LoadingPlugin,
                MainMenuPlugin,
                RunSetupPlugin,
                PauseMenuPlugin,
                ChipSelectPlugin,
                RunEndPlugin,
                RunPlugin,
            ));
        register_routing(app);
    }
}

/// Builds the defaults plugin with all config types and registries.
fn defaults_plugin() -> impl Plugin {
    RantzDefaultsPluginBuilder::<AppState>::new(AppState::Loading)
        .add_config::<PlayfieldDefaults>()
        .add_config::<CellDefaults>()
        .add_config::<InputDefaults>()
        .add_config::<MainMenuDefaults>()
        .add_config::<TimerUiDefaults>()
        .add_config::<ChipSelectDefaults>()
        .add_config::<DifficultyCurveDefaults>()
        .add_registry::<crate::cells::CellTypeRegistry>()
        .add_registry::<crate::breaker::BreakerRegistry>()
        .add_registry::<crate::bolt::BoltRegistry>()
        .add_registry::<crate::state::run::NodeLayoutRegistry>()
        .add_registry::<crate::chips::ChipTemplateRegistry>()
        .add_registry::<crate::chips::EvolutionTemplateRegistry>()
        .add_registry::<crate::walls::WallRegistry>()
        .build()
}

/// Registers all pass-through, teardown, and cleanup routing systems.
pub(crate) fn register_routing(app: &mut App) {
    // Pass-through routing
    app.add_systems(OnEnter(MenuState::Loading), routing::menu_loading_to_main)
        .add_systems(OnEnter(RunPhase::Loading), routing::run_loading_to_setup)
        .add_systems(
            OnEnter(NodeState::AnimateIn),
            routing::node_animate_in_to_playing,
        )
        .add_systems(
            OnEnter(NodeState::AnimateOut),
            routing::node_animate_out_to_teardown,
        )
        .add_systems(
            OnEnter(ChipSelectState::Loading),
            routing::chip_select_loading_to_animate_in,
        )
        .add_systems(
            OnEnter(ChipSelectState::AnimateIn),
            routing::chip_select_animate_in_to_selecting,
        )
        .add_systems(
            OnEnter(ChipSelectState::AnimateOut),
            routing::chip_select_animate_out_to_teardown,
        )
        .add_systems(
            OnEnter(RunEndState::Loading),
            routing::run_end_loading_to_animate_in,
        )
        .add_systems(
            OnEnter(RunEndState::AnimateIn),
            routing::run_end_animate_in_to_active,
        )
        .add_systems(
            OnEnter(RunEndState::AnimateOut),
            routing::run_end_animate_out_to_teardown,
        );
    // Teardown routing (cleanup runs first, then router decides next parent state)
    app.add_systems(
        OnEnter(NodeState::Teardown),
        (cleanup_on_exit::<NodeState>, routing::node_teardown_router).chain(),
    )
    .add_systems(
        OnEnter(ChipSelectState::Teardown),
        (
            cleanup_on_exit::<ChipSelectState>,
            routing::chip_select_teardown_router,
        )
            .chain(),
    )
    .add_systems(
        OnEnter(RunEndState::Teardown),
        (
            cleanup_on_exit::<RunEndState>,
            routing::run_end_teardown_router,
        )
            .chain(),
    )
    .add_systems(
        OnEnter(RunPhase::Teardown),
        (cleanup_on_exit::<RunPhase>, routing::run_teardown_router).chain(),
    )
    .add_systems(OnEnter(MenuState::Teardown), routing::menu_teardown_router);
    // Old cleanup markers — still used until entities migrate to CleanupOnExit<S>
    app.add_systems(
        OnEnter(NodeState::Teardown),
        cleanup_entities::<CleanupOnNodeExit>,
    )
    .add_systems(
        OnEnter(RunPhase::Teardown),
        cleanup_entities::<CleanupOnRunEnd>,
    );
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
