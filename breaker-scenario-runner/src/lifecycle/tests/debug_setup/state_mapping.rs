//! Tests for `PreviousGameState`, `map_forced_game_state`, and `map_scenario_breaker_state`.

use super::super::helpers::*;

// -------------------------------------------------------------------------
// apply_debug_setup — sets PreviousGameState from force_previous_game_state
// -------------------------------------------------------------------------

/// When `force_previous_game_state: Some(ForcedGameState::Loading)`,
/// `apply_debug_setup` must set `PreviousGameState.0` to `Some(GameState::Loading)`.
#[test]
fn apply_debug_setup_sets_previous_game_state_from_forced() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        debug_setup: Some(DebugSetup {
            force_previous_game_state: Some(ForcedGameState::Loading),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);
    app.init_resource::<PreviousGameState>();

    app.update();
    app.update();

    let prev = app.world().resource::<PreviousGameState>();
    assert_eq!(
        prev.0,
        Some(GameState::Loading),
        "expected PreviousGameState.0 == Some(GameState::Loading), got {:?}",
        prev.0
    );
}

// -------------------------------------------------------------------------
// map_forced_game_state — maps all variants correctly
// -------------------------------------------------------------------------

/// Each `ForcedGameState` variant must map 1:1 to the corresponding `GameState` variant.
#[test]
fn map_forced_game_state_maps_all_variants_correctly() {
    let mappings = [
        (ForcedGameState::Loading, GameState::Loading),
        (ForcedGameState::MainMenu, GameState::MainMenu),
        (ForcedGameState::RunSetup, GameState::RunSetup),
        (ForcedGameState::Playing, GameState::Playing),
        (ForcedGameState::TransitionOut, GameState::TransitionOut),
        (ForcedGameState::TransitionIn, GameState::TransitionIn),
        (ForcedGameState::ChipSelect, GameState::ChipSelect),
        (ForcedGameState::RunEnd, GameState::RunEnd),
        (ForcedGameState::MetaProgression, GameState::MetaProgression),
    ];
    for (forced, expected) in &mappings {
        let result = map_forced_game_state(*forced);
        assert_eq!(
            result, *expected,
            "map_forced_game_state({forced:?}) must return {expected:?}, got {result:?}"
        );
    }
}

// -------------------------------------------------------------------------
// map_scenario_breaker_state — maps all variants 1:1
// -------------------------------------------------------------------------

/// Each `ScenarioBreakerState` variant must map 1:1 to the corresponding
/// `BreakerState` variant.
#[test]
fn map_scenario_breaker_state_maps_all_variants() {
    let mappings = [
        (ScenarioBreakerState::Idle, BreakerState::Idle),
        (ScenarioBreakerState::Dashing, BreakerState::Dashing),
        (ScenarioBreakerState::Braking, BreakerState::Braking),
        (ScenarioBreakerState::Settling, BreakerState::Settling),
    ];
    for (scenario, expected) in &mappings {
        let result = map_scenario_breaker_state(*scenario);
        assert_eq!(
            result, *expected,
            "map_scenario_breaker_state({scenario:?}) must return {expected:?}, got {result:?}"
        );
    }
}
