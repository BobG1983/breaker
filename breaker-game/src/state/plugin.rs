//! State plugin registration.

use std::sync::Arc;

use bevy::prelude::*;
use iyes_progress::prelude::*;
use rantzsoft_defaults::prelude::*;
use rantzsoft_lifecycle::{
    FadeIn, FadeOut, RantzLifecyclePlugin, Route, RoutingTableAppExt, TransitionType,
    cleanup_on_exit,
};

use super::{
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
            .with_transition(TransitionType::In(Arc::new(FadeIn::default())))
            .when(|_| true),
    );
    app.add_route(
        Route::from(GameState::Menu)
            .to(GameState::Run)
            .with_transition(TransitionType::OutIn {
                out_e: Arc::new(FadeOut::default()),
                in_e: Arc::new(FadeIn::default()),
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
            .with_transition(TransitionType::OutIn {
                out_e: Arc::new(FadeOut::default()),
                in_e: Arc::new(FadeIn::default()),
            })
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
            .with_transition(TransitionType::OutIn {
                out_e: Arc::new(FadeOut::default()),
                in_e: Arc::new(FadeIn::default()),
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
            .with_transition(TransitionType::In(Arc::new(FadeIn::default())))
            .when(|_| true),
    );
    // Node → dynamic (ChipSelect, RunEnd, or Teardown based on NodeOutcome)
    app.add_route(
        Route::from(RunState::Node)
            .to_dynamic(resolve_node_next_state)
            .with_transition(TransitionType::OutIn {
                out_e: Arc::new(FadeOut::default()),
                in_e: Arc::new(FadeIn::default()),
            })
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
            .with_transition(TransitionType::OutIn {
                out_e: Arc::new(FadeOut::default()),
                in_e: Arc::new(FadeIn::default()),
            })
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
fn register_cleanup(app: &mut App) {
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
    // Safety net: also clean node-scoped entities when the run tears down
    // (covers quit-from-pause where NodeState may not reach its own Teardown)
    app.add_systems(OnEnter(RunState::Teardown), cleanup_on_exit::<NodeState>);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::run::resources::{NodeOutcome, NodeResult};

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

    // ── Behavior 7a: Quit routes to RunState::Teardown ──────────────────

    #[test]
    fn resolve_node_next_state_quit_returns_teardown() {
        let mut world = World::new();
        world.insert_resource(NodeOutcome {
            result: NodeResult::Quit,
            node_index: 0,
            transition_queued: false,
        });

        let next = resolve_node_next_state(&world);
        assert_eq!(
            next,
            RunState::Teardown,
            "NodeResult::Quit should route to RunState::Teardown"
        );
    }

    #[test]
    fn resolve_node_next_state_quit_ignores_node_index_and_transition_queued() {
        let mut world = World::new();
        world.insert_resource(NodeOutcome {
            result: NodeResult::Quit,
            node_index: 99,
            transition_queued: true,
        });

        let next = resolve_node_next_state(&world);
        assert_eq!(
            next,
            RunState::Teardown,
            "NodeResult::Quit should route to Teardown regardless of node_index or transition_queued"
        );
    }

    // ── Behavior 7b: InProgress routes to ChipSelect ────────────────────

    #[test]
    fn resolve_node_next_state_in_progress_returns_chip_select() {
        let mut world = World::new();
        world.insert_resource(NodeOutcome {
            result: NodeResult::InProgress,
            node_index: 0,
            transition_queued: false,
        });

        let next = resolve_node_next_state(&world);
        assert_eq!(next, RunState::ChipSelect);
    }

    // ── Behavior 7c: Won routes to RunEnd ───────────────────────────────

    #[test]
    fn resolve_node_next_state_won_returns_run_end() {
        let mut world = World::new();
        world.insert_resource(NodeOutcome {
            result: NodeResult::Won,
            node_index: 8,
            transition_queued: false,
        });

        let next = resolve_node_next_state(&world);
        assert_eq!(next, RunState::RunEnd);
    }

    // ── Behavior 7d: TimerExpired routes to RunEnd ──────────────────────

    #[test]
    fn resolve_node_next_state_timer_expired_returns_run_end() {
        let mut world = World::new();
        world.insert_resource(NodeOutcome {
            result: NodeResult::TimerExpired,
            node_index: 3,
            transition_queued: false,
        });

        let next = resolve_node_next_state(&world);
        assert_eq!(next, RunState::RunEnd);
    }

    // ── Behavior 7e: LivesDepleted routes to RunEnd ─────────────────────

    #[test]
    fn resolve_node_next_state_lives_depleted_returns_run_end() {
        let mut world = World::new();
        world.insert_resource(NodeOutcome {
            result: NodeResult::LivesDepleted,
            node_index: 1,
            transition_queued: false,
        });

        let next = resolve_node_next_state(&world);
        assert_eq!(next, RunState::RunEnd);
    }

    // ── Behavior 8: CleanupOnExit<NodeState> on OnEnter(RunState::Teardown) ────

    #[test]
    fn cleanup_on_node_exit_runs_on_enter_run_state_teardown() {
        use bevy::state::app::StatesPlugin;
        use rantzsoft_lifecycle::CleanupOnExit;

        use crate::state::types::{AppState, GameState};

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<AppState>()
            .add_sub_state::<GameState>()
            .add_sub_state::<RunState>()
            .add_sub_state::<NodeState>()
            .add_sub_state::<ChipSelectState>()
            .add_sub_state::<RunEndState>();

        // Use the real register_cleanup to verify it includes
        // cleanup_on_exit::<NodeState> on OnEnter(RunState::Teardown).
        register_cleanup(&mut app);

        // Navigate to RunState::Node first
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Run);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<RunState>>()
            .set(RunState::Node);
        app.update();

        // Spawn entities with cleanup markers
        let node_exit_entity = app
            .world_mut()
            .spawn(CleanupOnExit::<NodeState>::default())
            .id();
        let both_markers_entity = app
            .world_mut()
            .spawn((
                CleanupOnExit::<NodeState>::default(),
                CleanupOnExit::<RunState>::default(),
            ))
            .id();
        let unrelated_entity = app.world_mut().spawn_empty().id();

        // Transition directly to RunState::Teardown (bypassing NodeState teardown)
        app.world_mut()
            .resource_mut::<NextState<RunState>>()
            .set(RunState::Teardown);
        app.update();

        assert!(
            app.world().get_entity(node_exit_entity).is_err(),
            "CleanupOnExit<NodeState> entity should be despawned on RunState::Teardown"
        );
        assert!(
            app.world().get_entity(both_markers_entity).is_err(),
            "Entity with both CleanupOnExit<NodeState> and CleanupOnExit<RunState> should be despawned"
        );
        assert!(
            app.world().get_entity(unrelated_entity).is_ok(),
            "Entity without cleanup markers should survive"
        );
    }
}
