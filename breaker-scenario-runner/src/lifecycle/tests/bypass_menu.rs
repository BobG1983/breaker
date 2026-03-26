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

// -------------------------------------------------------------------------
// bypass_menu_to_playing — writes ChipSelected from initial_chips
// -------------------------------------------------------------------------

/// When `initial_chips` is `Some` with one chip name, `bypass_menu_to_playing`
/// must write a [`ChipSelected`] message for that chip.
#[test]
fn bypass_menu_to_playing_writes_chip_selected_for_initial_chips() {
    let mut definition = make_scenario(100);
    definition.initial_chips = Some(vec!["TestChip".to_owned()]);

    let mut app = bypass_app(definition);
    app.init_resource::<CapturedChipSelected>()
        .add_systems(Update, collect_chip_selected.after(bypass_menu_to_playing));

    app.update();

    let captured = app.world().resource::<CapturedChipSelected>();
    assert_eq!(
        captured.0.len(),
        1,
        "expected 1 ChipSelected message when initial_chips is Some, got {}",
        captured.0.len()
    );
    assert_eq!(
        captured.0[0].name, "TestChip",
        "expected ChipSelected name == 'TestChip', got '{}'",
        captured.0[0].name
    );
}

/// When `initial_chips` is `None`, `bypass_menu_to_playing` must not write
/// any [`ChipSelected`] messages.
#[test]
fn bypass_menu_to_playing_no_chip_selected_when_initial_chips_is_none() {
    let mut app = bypass_app(make_scenario(100));
    app.init_resource::<CapturedChipSelected>()
        .add_systems(Update, collect_chip_selected.after(bypass_menu_to_playing));

    app.update();

    let captured = app.world().resource::<CapturedChipSelected>();
    assert!(
        captured.0.is_empty(),
        "expected no ChipSelected messages when initial_chips is None, got {}",
        captured.0.len()
    );
}
