//! State plugin registration.

use std::sync::Arc;

use bevy::prelude::*;
use iyes_progress::prelude::*;
use rantzsoft_defaults::prelude::*;
use rantzsoft_lifecycle::{
    FadeIn, FadeOut, RantzLifecyclePlugin, Route, RoutingTableAppExt, TransitionType,
    cleanup_on_exit,
};

use super::super::{
    app::loading::LoadingPlugin,
    menu::{
        main::{MainMenuDefaults, MainMenuPlugin},
        start_game::RunSetupPlugin,
    },
    pause::PauseMenuPlugin,
    run::{
        RunPlugin,
        chip_select::{ChipSelectDefaults, ChipSelectPlugin},
        node::hud::TimerUiDefaults,
        resources::{DifficultyCurveDefaults, NodeOutcome, NodeResult},
        run_end::RunEndPlugin,
    },
};
use crate::{
    cells::CellDefaults,
    input::InputDefaults,
    shared::{PlayfieldConfig, PlayfieldDefaults},
    state::types::{
        AppState, ChipSelectState, GameState, MenuState, NodeState, RunEndState, RunState,
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
            .add_sub_state::<RunState>()
            .add_sub_state::<NodeState>()
            .add_sub_state::<ChipSelectState>()
            .add_sub_state::<RunEndState>()
            // Lifecycle crate — routing tables + dispatch for all state types
            .add_plugins(
                RantzLifecyclePlugin::new()
                    .register_state::<AppState>()
                    .register_state::<GameState>()
                    .register_state::<MenuState>()
                    .register_state::<RunState>()
                    .register_state::<NodeState>()
                    .register_state::<ChipSelectState>()
                    .register_state::<RunEndState>(),
            )
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

/// Registers declarative routes via the lifecycle crate and cleanup systems.
fn register_routing(app: &mut App) {
    register_parent_routes(app);
    register_run_routes(app);
    register_node_routes(app);
    register_chip_select_routes(app);
    register_run_end_routes(app);
    register_cleanup(app);
}

/// `GameState` and `MenuState` routes — parent-level routing.
fn register_parent_routes(app: &mut App) {
    // ── GameState routes (parent watches MenuState/RunState) ──────────
    app.add_route(
        Route::from(GameState::Loading)
            .to(GameState::Menu)
            .with_transition(TransitionType::In(Arc::new(FadeIn {
                duration: 0.6,
                color: Color::WHITE,
            })))
            .when(|_| true),
    );
    app.add_route(
        Route::from(GameState::Menu)
            .to(GameState::Run)
            .with_transition(TransitionType::Out(Arc::new(FadeOut {
                duration: 0.6,
                color: Color::WHITE,
            })))
            .when(|world| {
                world
                    .get_resource::<State<MenuState>>()
                    .is_some_and(|s| *s.get() == MenuState::Teardown)
            }),
    );
    app.add_route(
        Route::from(GameState::Run)
            .to(GameState::Menu)
            .with_transition(TransitionType::Out(Arc::new(FadeOut {
                duration: 0.6,
                color: Color::WHITE,
            })))
            .when(|world| {
                world
                    .get_resource::<State<RunState>>()
                    .is_some_and(|s| *s.get() == RunState::Teardown)
            }),
    );

    // ── MenuState routes ──────────────────────────────────────────────
    app.add_route(
        Route::from(MenuState::Loading)
            .to(MenuState::Main)
            .when(|_| true),
    );
    // Main → dynamic (StartGame/Options/Meta based on selection)
    app.add_route(
        Route::from(MenuState::Main)
            .to_dynamic(|world| {
                use crate::state::menu::main::MenuItem;
                let selection = world.resource::<crate::state::menu::main::MainMenuSelection>();
                match selection.selected {
                    MenuItem::Play => MenuState::StartGame,
                    MenuItem::Settings => MenuState::Options,
                    MenuItem::Quit => MenuState::Main, // Quit handled via AppExit, not routing
                }
            })
            .with_transition(TransitionType::Out(Arc::new(FadeOut {
                duration: 0.6,
                color: Color::WHITE,
            }))),
    );
    // StartGame → Teardown (message-triggered by handle_run_setup_input)
    app.add_route(Route::from(MenuState::StartGame).to(MenuState::Teardown));
}

/// Resolves the next `RunState` when leaving `RunState::Node`.
///
/// Extracted from the inline `to_dynamic` closure so it can be unit tested.
/// Reads `NodeOutcome.result` to decide routing.
pub(crate) fn resolve_node_next_state(world: &World) -> RunState {
    match world.resource::<NodeOutcome>().result {
        NodeResult::InProgress => RunState::ChipSelect,
        NodeResult::Quit => RunState::Teardown,
        _ => RunState::RunEnd,
    }
}

/// `RunState` routes — run lifecycle with transition effects.
fn register_run_routes(app: &mut App) {
    app.add_route(
        Route::from(RunState::Loading)
            .to(RunState::Setup)
            .when(|_| true),
    );
    app.add_route(
        Route::from(RunState::Setup)
            .to(RunState::Node)
            .with_transition(TransitionType::In(Arc::new(FadeIn {
                duration: 0.6,
                color: Color::WHITE,
            })))
            .when(|_| true),
    );
    // Node → dynamic (ChipSelect, RunEnd, or Teardown based on NodeOutcome)
    app.add_route(
        Route::from(RunState::Node)
            .to_dynamic(resolve_node_next_state)
            .with_transition(TransitionType::Out(Arc::new(FadeOut {
                duration: 0.6,
                color: Color::WHITE,
            })))
            .when(|world| {
                world
                    .get_resource::<State<NodeState>>()
                    .is_some_and(|s| *s.get() == NodeState::Teardown)
            }),
    );
    // ChipSelect → Node (parent watches ChipSelectState teardown)
    app.add_route(
        Route::from(RunState::ChipSelect)
            .to(RunState::Node)
            .with_transition(TransitionType::Out(Arc::new(FadeOut {
                duration: 0.6,
                color: Color::WHITE,
            })))
            .when(|world| {
                world
                    .get_resource::<State<ChipSelectState>>()
                    .is_some_and(|s| *s.get() == ChipSelectState::Teardown)
            }),
    );
    // RunEnd → Teardown (parent watches RunEndState teardown)
    app.add_route(
        Route::from(RunState::RunEnd)
            .to(RunState::Teardown)
            .when(|world| {
                world
                    .get_resource::<State<RunEndState>>()
                    .is_some_and(|s| *s.get() == RunEndState::Teardown)
            }),
    );
}

/// `NodeState` routes — node lifecycle.
fn register_node_routes(app: &mut App) {
    // Loading → AnimateIn: message-triggered (check_spawn_complete sends ChangeState)
    app.add_route(Route::from(NodeState::Loading).to(NodeState::AnimateIn));
    // AnimateIn → Playing: pass-through (gameplay animations later)
    app.add_route(
        Route::from(NodeState::AnimateIn)
            .to(NodeState::Playing)
            .when(|_| true),
    );
    // Playing → AnimateOut: message-triggered (handle_node_cleared etc. send ChangeState)
    app.add_route(Route::from(NodeState::Playing).to(NodeState::AnimateOut));
    // AnimateOut → Teardown: pass-through
    app.add_route(
        Route::from(NodeState::AnimateOut)
            .to(NodeState::Teardown)
            .when(|_| true),
    );
}

/// `ChipSelectState` routes — chip selection lifecycle.
fn register_chip_select_routes(app: &mut App) {
    app.add_route(
        Route::from(ChipSelectState::Loading)
            .to(ChipSelectState::AnimateIn)
            .when(|_| true),
    );
    app.add_route(
        Route::from(ChipSelectState::AnimateIn)
            .to(ChipSelectState::Selecting)
            .when(|_| true),
    );
    // Selecting → AnimateOut: message-triggered (handle_chip_input/tick_chip_timer)
    app.add_route(Route::from(ChipSelectState::Selecting).to(ChipSelectState::AnimateOut));
    app.add_route(
        Route::from(ChipSelectState::AnimateOut)
            .to(ChipSelectState::Teardown)
            .when(|_| true),
    );
}

/// `RunEndState` routes — run end lifecycle.
fn register_run_end_routes(app: &mut App) {
    app.add_route(
        Route::from(RunEndState::Loading)
            .to(RunEndState::AnimateIn)
            .when(|_| true),
    );
    app.add_route(
        Route::from(RunEndState::AnimateIn)
            .to(RunEndState::Active)
            .when(|_| true),
    );
    // Active → AnimateOut: message-triggered (handle_run_end_input)
    app.add_route(Route::from(RunEndState::Active).to(RunEndState::AnimateOut));
    app.add_route(
        Route::from(RunEndState::AnimateOut)
            .to(RunEndState::Teardown)
            .when(|_| true),
    );
}

/// Cleanup systems on teardown entry.
pub(super) fn register_cleanup(app: &mut App) {
    app.add_systems(OnEnter(NodeState::Teardown), cleanup_on_exit::<NodeState>)
        .add_systems(
            OnEnter(ChipSelectState::Teardown),
            cleanup_on_exit::<ChipSelectState>,
        )
        .add_systems(
            OnEnter(RunEndState::Teardown),
            cleanup_on_exit::<RunEndState>,
        )
        .add_systems(OnEnter(RunState::Teardown), cleanup_on_exit::<RunState>);
}
