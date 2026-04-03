use super::helpers::*;

// -------------------------------------------------------------------------
// bypass_menu_to_playing -- sets RunSeed from scenario config
// -------------------------------------------------------------------------

/// `bypass_menu_to_playing` must set `RunSeed` to `Some(0)` when the
/// scenario definition has `seed: None` (default 0 for determinism).
#[test]
fn bypass_menu_to_playing_sets_run_seed_default_zero() {
    let mut app = bypass_app(make_scenario(100));

    app.update();

    let seed = app.world().resource::<breaker::shared::RunSeed>();
    assert_eq!(
        seed.0,
        Some(0),
        "expected RunSeed(Some(0)) when scenario seed is None, got {:?}",
        seed.0
    );
}

/// `bypass_menu_to_playing` must set `RunSeed` to `Some(42)` when the
/// scenario definition has `seed: Some(42)`.
#[test]
fn bypass_menu_to_playing_sets_run_seed_from_scenario() {
    let mut definition = make_scenario(100);
    definition.seed = Some(42);

    let mut app = bypass_app(definition);

    app.update();

    let seed = app.world().resource::<breaker::shared::RunSeed>();
    assert_eq!(
        seed.0,
        Some(42),
        "expected RunSeed(Some(42)) from scenario seed, got {:?}",
        seed.0
    );
}

// -------------------------------------------------------------------------
// restart_run_on_end -- transitions from RunEnd back through teardown chain
// -------------------------------------------------------------------------

/// `restart_run_on_end` must set `RunPhase` to Teardown, which triggers
/// the routing chain back to menus.
#[test]
fn restart_run_on_end_transitions_through_teardown() {
    use breaker::state::types::RunEndState;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<AppState>()
        .add_sub_state::<GameState>()
        .add_sub_state::<MenuState>()
        .add_sub_state::<RunPhase>()
        .add_sub_state::<NodeState>()
        .add_sub_state::<ChipSelectState>()
        .add_sub_state::<RunEndState>()
        .add_systems(OnEnter(RunEndState::Active), restart_run_on_end);

    // Navigate to RunEndState::Active
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Game);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Run);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<RunPhase>>()
        .set(RunPhase::RunEnd);
    app.update();
    // RunEndState defaults to Loading -- need to get to Active
    app.world_mut()
        .resource_mut::<NextState<RunEndState>>()
        .set(RunEndState::Active);
    // OnEnter(RunEndState::Active) fires and sets RunPhase to Teardown.
    app.update();

    // One more update to process RunPhase transition
    app.update();

    let state = app.world().resource::<State<RunPhase>>();
    assert_eq!(
        **state,
        RunPhase::Teardown,
        "expected restart_run_on_end to set RunPhase::Teardown"
    );
}
