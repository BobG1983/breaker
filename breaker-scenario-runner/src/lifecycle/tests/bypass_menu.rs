use super::helpers::*;

// -------------------------------------------------------------------------
// bypass_menu_to_playing — sets RunSeed from scenario config
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
// restart_run_on_end — transitions from RunEnd to MainMenu
// -------------------------------------------------------------------------

/// `restart_run_on_end` must set the next state to `MainMenu` so
/// `bypass_menu_to_playing` can restart the run.
#[test]
fn restart_run_on_end_transitions_to_main_menu() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<GameState>()
        .add_systems(OnEnter(GameState::RunEnd), restart_run_on_end);

    // Drive into RunEnd
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::RunEnd);
    app.update();

    // OnEnter(RunEnd) fires and sets next state to MainMenu.
    // One more update applies the transition.
    app.update();

    let state = app.world().resource::<State<GameState>>();
    assert_eq!(
        **state,
        GameState::MainMenu,
        "expected restart_run_on_end to transition to MainMenu"
    );
}

