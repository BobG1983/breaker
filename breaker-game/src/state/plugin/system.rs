//! State plugin registration.

use std::sync::Arc;

use bevy::prelude::*;
use iyes_progress::prelude::*;
use rantzsoft_defaults::prelude::*;
use rantzsoft_stateflow::{
    FadeIn, FadeOut, RantzStateflowPlugin, Route, RoutingTableAppExt, TransitionType,
    cleanup_on_exit,
};

use crate::{
    bolt::BoltRegistry,
    breaker::BreakerRegistry,
    cells::{CellDefaults, CellTypeRegistry, ToughnessDefaults},
    chips::{ChipTemplateRegistry, EvolutionTemplateRegistry},
    input::InputDefaults,
    prelude::*,
    shared::PlayfieldDefaults,
    state::{
        app::loading::LoadingPlugin,
        menu::{
            main::{MainMenuDefaults, MainMenuPlugin},
            start_game::RunSetupPlugin,
        },
        pause::PauseMenuPlugin,
        run::{
            NodeLayoutRegistry, RunPlugin,
            chip_select::{ChipSelectDefaults, ChipSelectPlugin},
            node::hud::TimerUiDefaults,
            resources::{DifficultyCurveDefaults, NodeOutcome, NodeResult},
            run_end::RunEndPlugin,
        },
    },
    walls::WallRegistry,
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
                RantzStateflowPlugin::new()
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
        .add_config::<ToughnessDefaults>()
        .add_config::<DifficultyCurveDefaults>()
        .add_registry::<CellTypeRegistry>()
        .add_registry::<BreakerRegistry>()
        .add_registry::<BoltRegistry>()
        .add_registry::<NodeLayoutRegistry>()
        .add_registry::<ChipTemplateRegistry>()
        .add_registry::<EvolutionTemplateRegistry>()
        .add_registry::<WallRegistry>()
        .build()
}

/// Registers declarative routes via the lifecycle crate and cleanup systems.
///
/// Cross-state routes (parent watches child teardown) live here.
/// Leaf-state routes live in their domain plugins (`NodePlugin`, `ChipSelectPlugin`, `RunEndPlugin`).
fn register_routing(app: &mut App) {
    register_app_routes(app);
    register_parent_routes(app);
    register_run_routes(app);
    register_cleanup(app);
}

/// `AppState` routes — top-level app lifecycle.
fn register_app_routes(app: &mut App) {
    app.add_route(
        Route::from(AppState::Game)
            .to(AppState::Teardown)
            .when(|world| {
                world
                    .get_resource::<State<GameState>>()
                    .is_some_and(|s| *s.get() == GameState::Teardown)
            }),
    );
}

/// `GameState` and `MenuState` routes — parent-level routing.
fn register_parent_routes(app: &mut App) {
    // ── GameState routes (parent watches MenuState/RunState) ──────────
    app.add_route(
        Route::from(GameState::Loading)
            .to(GameState::Menu)
            .with_transition(TransitionType::Out(Arc::new(FadeOut {
                duration: 0.6,
                color:    Color::WHITE,
            })))
            .when(|_| true),
    );
    app.add_route(
        Route::from(GameState::Menu)
            .to_dynamic(|world| {
                use crate::state::menu::main::{MainMenuSelection, MenuItem};
                let Some(selection) = world.get_resource::<MainMenuSelection>() else {
                    return GameState::Run;
                };
                match selection.selected {
                    MenuItem::Quit => GameState::Teardown,
                    _ => GameState::Run,
                }
            })
            .with_dynamic_transition(|world| {
                use crate::state::menu::main::{MainMenuSelection, MenuItem};
                let Some(selection) = world.get_resource::<MainMenuSelection>() else {
                    return TransitionType::Out(Arc::new(FadeOut {
                        duration: 0.6,
                        color:    Color::WHITE,
                    }));
                };
                match selection.selected {
                    MenuItem::Quit => TransitionType::None,
                    _ => TransitionType::Out(Arc::new(FadeOut {
                        duration: 0.6,
                        color:    Color::WHITE,
                    })),
                }
            })
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
                color:    Color::WHITE,
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
            .with_transition(TransitionType::In(Arc::new(FadeIn {
                duration: 0.6,
                color:    Color::WHITE,
            })))
            .when(|_| true),
    );
    // Main → dynamic (StartGame/Options/Teardown based on selection)
    app.add_route(
        Route::from(MenuState::Main)
            .to_dynamic(|world| {
                use crate::state::menu::main::{MainMenuSelection, MenuItem};
                let Some(selection) = world.get_resource::<MainMenuSelection>() else {
                    return MenuState::StartGame;
                };
                match selection.selected {
                    MenuItem::Play => MenuState::StartGame,
                    MenuItem::Settings => MenuState::Options,
                    MenuItem::Quit => MenuState::Teardown,
                }
            })
            .with_dynamic_transition(|world| {
                use crate::state::menu::main::{MainMenuSelection, MenuItem};
                let Some(selection) = world.get_resource::<MainMenuSelection>() else {
                    return TransitionType::Out(Arc::new(FadeOut {
                        duration: 0.6,
                        color:    Color::WHITE,
                    }));
                };
                match selection.selected {
                    MenuItem::Quit => TransitionType::None,
                    _ => TransitionType::Out(Arc::new(FadeOut {
                        duration: 0.6,
                        color:    Color::WHITE,
                    })),
                }
            }),
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
                color:    Color::WHITE,
            })))
            .when(|_| true),
    );
    // Node → dynamic (ChipSelect, RunEnd, or Teardown based on NodeOutcome)
    app.add_route(
        Route::from(RunState::Node)
            .to_dynamic(resolve_node_next_state)
            .with_transition(TransitionType::Out(Arc::new(FadeOut {
                duration: 0.6,
                color:    Color::WHITE,
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
                color:    Color::WHITE,
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

/// Cleanup systems for cross-state teardown.
///
/// Leaf-state cleanup lives in domain plugins (`NodePlugin`, `ChipSelectPlugin`, `RunEndPlugin`).
pub(super) fn register_cleanup(app: &mut App) {
    app.add_systems(OnEnter(RunState::Teardown), cleanup_on_exit::<RunState>)
        .add_systems(OnEnter(AppState::Teardown), send_app_exit);
}

/// Sends [`AppExit::Success`] to cleanly shut down the application.
fn send_app_exit(mut writer: MessageWriter<AppExit>) {
    writer.write(AppExit::Success);
}
