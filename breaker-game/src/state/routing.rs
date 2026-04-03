//! Pass-through and teardown routing systems for the hierarchical state machine.
//!
//! Pass-throughs auto-advance states that don't have real content yet
//! (e.g. `AnimateIn` → `Playing`). Teardown routers run cleanup and
//! determine the next parent state based on game context.

use bevy::prelude::*;
use tracing::info;

use super::types::{ChipSelectState, GamePhase, MenuState, NodeState, RunEndState, RunPhase};
use crate::state::run::resources::{RunOutcome, RunState};

// ──────────────────────────────────────────────────────────────
//  Pass-through routing — auto-advance states without real content
// ──────────────────────────────────────────────────────────────

/// `MenuState::Loading` → `MenuState::Main`
pub(crate) fn menu_loading_to_main(mut next: ResMut<NextState<MenuState>>) {
    next.set(MenuState::Main);
}

/// `RunPhase::Loading` → `RunPhase::Setup` (after run init systems run)
pub(crate) fn run_loading_to_setup(mut next: ResMut<NextState<RunPhase>>) {
    next.set(RunPhase::Setup);
}

/// `NodeState::AnimateIn` → `NodeState::Playing`
pub(crate) fn node_animate_in_to_playing(mut next: ResMut<NextState<NodeState>>) {
    next.set(NodeState::Playing);
}

/// `NodeState::AnimateOut` → `NodeState::Teardown`
pub(crate) fn node_animate_out_to_teardown(mut next: ResMut<NextState<NodeState>>) {
    next.set(NodeState::Teardown);
}

/// `ChipSelectState::Loading` → `ChipSelectState::AnimateIn`
pub(crate) fn chip_select_loading_to_animate_in(mut next: ResMut<NextState<ChipSelectState>>) {
    next.set(ChipSelectState::AnimateIn);
}

/// `ChipSelectState::AnimateIn` → `ChipSelectState::Selecting`
pub(crate) fn chip_select_animate_in_to_selecting(mut next: ResMut<NextState<ChipSelectState>>) {
    next.set(ChipSelectState::Selecting);
}

/// `ChipSelectState::AnimateOut` → `ChipSelectState::Teardown`
pub(crate) fn chip_select_animate_out_to_teardown(mut next: ResMut<NextState<ChipSelectState>>) {
    next.set(ChipSelectState::Teardown);
}

/// `RunEndState::Loading` → `RunEndState::AnimateIn`
pub(crate) fn run_end_loading_to_animate_in(mut next: ResMut<NextState<RunEndState>>) {
    next.set(RunEndState::AnimateIn);
}

/// `RunEndState::AnimateIn` → `RunEndState::Active`
pub(crate) fn run_end_animate_in_to_active(mut next: ResMut<NextState<RunEndState>>) {
    next.set(RunEndState::Active);
}

/// `RunEndState::AnimateOut` → `RunEndState::Teardown`
pub(crate) fn run_end_animate_out_to_teardown(mut next: ResMut<NextState<RunEndState>>) {
    next.set(RunEndState::Teardown);
}

// ──────────────────────────────────────────────────────────────
//  Teardown routing — cleanup + determine next parent state
// ──────────────────────────────────────────────────────────────

/// `NodeState::Teardown` — route to `ChipSelect` or `RunEnd`.
///
/// Cleanup is handled by `cleanup_on_exit::<NodeState>` chained before this.
/// Reads [`RunOutcome`] from [`RunState`] to decide:
/// - `InProgress` → `RunPhase::ChipSelect` (mid-run, next node)
/// - `Won` / `TimerExpired` / `LivesDepleted` → `RunPhase::RunEnd`
pub(crate) fn node_teardown_router(
    run_state: Res<RunState>,
    mut next: ResMut<NextState<RunPhase>>,
) {
    match run_state.outcome {
        RunOutcome::InProgress => {
            info!("Node teardown → ChipSelect (run in progress)");
            next.set(RunPhase::ChipSelect);
        }
        outcome => {
            info!("Node teardown → RunEnd (outcome: {outcome:?})");
            next.set(RunPhase::RunEnd);
        }
    }
}

/// `ChipSelectState::Teardown` — advance to next node.
pub(crate) fn chip_select_teardown_router(mut next: ResMut<NextState<RunPhase>>) {
    info!("ChipSelect teardown → Node");
    next.set(RunPhase::Node);
}

/// `RunEndState::Teardown` — signal run complete.
pub(crate) fn run_end_teardown_router(mut next: ResMut<NextState<RunPhase>>) {
    info!("RunEnd teardown → Run Teardown");
    next.set(RunPhase::Teardown);
}

/// `RunPhase::Teardown` — return to menu.
pub(crate) fn run_teardown_router(mut next: ResMut<NextState<GamePhase>>) {
    info!("Run teardown → Menu");
    next.set(GamePhase::Menu);
}

/// `MenuState::Teardown` — start a run.
pub(crate) fn menu_teardown_router(mut next: ResMut<NextState<GamePhase>>) {
    info!("Menu teardown → Run");
    next.set(GamePhase::Run);
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::state::types::AppState;

    /// Helper to create a test app with all state types registered.
    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(StatesPlugin)
            .init_state::<AppState>()
            .add_sub_state::<GamePhase>()
            .add_sub_state::<MenuState>()
            .add_sub_state::<RunPhase>()
            .add_sub_state::<NodeState>()
            .add_sub_state::<ChipSelectState>()
            .add_sub_state::<RunEndState>()
            .init_resource::<RunState>();
        app
    }

    // --- Pass-through tests ---

    #[test]
    fn menu_loading_passes_to_main() {
        let mut app = test_app();
        // Register system BEFORE navigation so OnEnter(MenuState::Loading) fires
        app.add_systems(OnEnter(MenuState::Loading), menu_loading_to_main);

        // Navigate to GamePhase::Menu to enable MenuState
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GamePhase>>()
            .set(GamePhase::Menu);
        // MenuState defaults to Loading — OnEnter fires here, sets NextState<MenuState>::Main
        app.update();
        // Extra update to process the NextState set by the OnEnter system
        app.update();

        let state = app.world().resource::<State<MenuState>>();
        assert_eq!(**state, MenuState::Main);
    }

    #[test]
    fn node_animate_in_passes_to_playing() {
        let mut app = test_app();
        app.add_systems(OnEnter(NodeState::AnimateIn), node_animate_in_to_playing);

        // Navigate to NodeState
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GamePhase>>()
            .set(GamePhase::Run);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<RunPhase>>()
            .set(RunPhase::Node);
        app.update();

        // NodeState defaults to Loading → set AnimateIn
        app.world_mut()
            .resource_mut::<NextState<NodeState>>()
            .set(NodeState::AnimateIn);
        // OnEnter(AnimateIn) fires here, sets NextState<NodeState>::Playing
        app.update();
        // Extra update to process the NextState set by the OnEnter system
        app.update();

        let state = app.world().resource::<State<NodeState>>();
        assert_eq!(**state, NodeState::Playing);
    }

    #[test]
    fn node_animate_out_passes_to_teardown() {
        let mut app = test_app();
        app.add_systems(OnEnter(NodeState::AnimateOut), node_animate_out_to_teardown);

        // Navigate to NodeState::Playing first
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GamePhase>>()
            .set(GamePhase::Run);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<RunPhase>>()
            .set(RunPhase::Node);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<NodeState>>()
            .set(NodeState::AnimateOut);
        // OnEnter(AnimateOut) fires here, sets NextState<NodeState>::Teardown
        app.update();
        // Extra update to process the NextState set by the OnEnter system
        app.update();

        let state = app.world().resource::<State<NodeState>>();
        assert_eq!(**state, NodeState::Teardown);
    }

    // --- Teardown routing tests ---

    #[test]
    fn node_teardown_routes_to_chip_select_when_in_progress() {
        let mut app = test_app();
        app.add_systems(OnEnter(NodeState::Teardown), node_teardown_router);

        // Set outcome to InProgress (default)
        assert_eq!(
            app.world().resource::<RunState>().outcome,
            RunOutcome::InProgress
        );

        // Navigate to NodeState::Teardown
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GamePhase>>()
            .set(GamePhase::Run);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<RunPhase>>()
            .set(RunPhase::Node);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<NodeState>>()
            .set(NodeState::Teardown);
        // OnEnter(Teardown) fires here, sets NextState<RunPhase>::ChipSelect
        app.update();
        // Extra update to process the NextState set by the OnEnter system
        app.update();

        let state = app.world().resource::<State<RunPhase>>();
        assert_eq!(**state, RunPhase::ChipSelect);
    }

    #[test]
    fn node_teardown_routes_to_run_end_when_won() {
        let mut app = test_app();
        app.add_systems(OnEnter(NodeState::Teardown), node_teardown_router);

        app.world_mut().resource_mut::<RunState>().outcome = RunOutcome::Won;

        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GamePhase>>()
            .set(GamePhase::Run);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<RunPhase>>()
            .set(RunPhase::Node);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<NodeState>>()
            .set(NodeState::Teardown);
        // OnEnter(Teardown) fires here, sets NextState<RunPhase>::RunEnd
        app.update();
        // Extra update to process the NextState set by the OnEnter system
        app.update();

        let state = app.world().resource::<State<RunPhase>>();
        assert_eq!(**state, RunPhase::RunEnd);
    }

    #[test]
    fn node_teardown_routes_to_run_end_when_timer_expired() {
        let mut app = test_app();
        app.add_systems(OnEnter(NodeState::Teardown), node_teardown_router);

        app.world_mut().resource_mut::<RunState>().outcome = RunOutcome::TimerExpired;

        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GamePhase>>()
            .set(GamePhase::Run);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<RunPhase>>()
            .set(RunPhase::Node);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<NodeState>>()
            .set(NodeState::Teardown);
        // OnEnter(Teardown) fires here, sets NextState<RunPhase>::RunEnd
        app.update();
        // Extra update to process the NextState set by the OnEnter system
        app.update();

        let state = app.world().resource::<State<RunPhase>>();
        assert_eq!(**state, RunPhase::RunEnd);
    }

    #[test]
    fn node_teardown_routes_to_run_end_when_lives_depleted() {
        let mut app = test_app();
        app.add_systems(OnEnter(NodeState::Teardown), node_teardown_router);

        app.world_mut().resource_mut::<RunState>().outcome = RunOutcome::LivesDepleted;

        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GamePhase>>()
            .set(GamePhase::Run);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<RunPhase>>()
            .set(RunPhase::Node);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<NodeState>>()
            .set(NodeState::Teardown);
        // OnEnter(Teardown) fires here, sets NextState<RunPhase>::RunEnd
        app.update();
        // Extra update to process the NextState set by the OnEnter system
        app.update();

        let state = app.world().resource::<State<RunPhase>>();
        assert_eq!(**state, RunPhase::RunEnd);
    }

    #[test]
    fn chip_select_teardown_routes_to_node() {
        let mut app = test_app();
        app.add_systems(
            OnEnter(ChipSelectState::Teardown),
            chip_select_teardown_router,
        );

        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GamePhase>>()
            .set(GamePhase::Run);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<RunPhase>>()
            .set(RunPhase::ChipSelect);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<ChipSelectState>>()
            .set(ChipSelectState::Teardown);
        // OnEnter(Teardown) fires here, sets NextState<RunPhase>::Node
        app.update();
        // Extra update to process the NextState set by the OnEnter system
        app.update();

        let state = app.world().resource::<State<RunPhase>>();
        assert_eq!(**state, RunPhase::Node);
    }

    #[test]
    fn run_end_teardown_routes_to_run_teardown() {
        let mut app = test_app();
        app.add_systems(OnEnter(RunEndState::Teardown), run_end_teardown_router);

        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GamePhase>>()
            .set(GamePhase::Run);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<RunPhase>>()
            .set(RunPhase::RunEnd);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<RunEndState>>()
            .set(RunEndState::Teardown);
        // OnEnter(Teardown) fires here, sets NextState<RunPhase>::Teardown
        app.update();
        // Extra update to process the NextState set by the OnEnter system
        app.update();

        let state = app.world().resource::<State<RunPhase>>();
        assert_eq!(**state, RunPhase::Teardown);
    }

    #[test]
    fn menu_teardown_routes_to_run() {
        let mut app = test_app();
        app.add_systems(OnEnter(MenuState::Teardown), menu_teardown_router);

        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GamePhase>>()
            .set(GamePhase::Menu);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<MenuState>>()
            .set(MenuState::Teardown);
        // OnEnter(Teardown) fires here, sets NextState<GamePhase>::Run
        app.update();
        // Extra update to process the NextState set by the OnEnter system
        app.update();

        let state = app.world().resource::<State<GamePhase>>();
        assert_eq!(**state, GamePhase::Run);
    }

    #[test]
    fn run_teardown_routes_to_menu() {
        let mut app = test_app();
        app.add_systems(OnEnter(RunPhase::Teardown), run_teardown_router);

        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GamePhase>>()
            .set(GamePhase::Run);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<RunPhase>>()
            .set(RunPhase::Teardown);
        // OnEnter(Teardown) fires here, sets NextState<GamePhase>::Menu
        app.update();
        // Extra update to process the NextState set by the OnEnter system
        app.update();

        let state = app.world().resource::<State<GamePhase>>();
        assert_eq!(**state, GamePhase::Menu);
    }
}
