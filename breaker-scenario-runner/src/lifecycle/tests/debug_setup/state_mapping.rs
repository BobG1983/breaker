//! Tests for `map_forced_game_state` and `map_scenario_dash_state`.

use crate::lifecycle::tests::helpers::*;

// -------------------------------------------------------------------------
// map_forced_game_state -- maps all variants correctly
// -------------------------------------------------------------------------

/// Each `ForcedGameState` variant must map to the corresponding `GameState` variant
/// in the new hierarchical state machine.
#[test]
fn map_forced_game_state_maps_all_variants_correctly() {
    let mappings = [
        (ForcedGameState::Loading, GameState::Loading),
        (ForcedGameState::MainMenu, GameState::Menu),
        (ForcedGameState::RunSetup, GameState::Run),
        (ForcedGameState::Playing, GameState::Run),
        (ForcedGameState::TransitionOut, GameState::Run),
        (ForcedGameState::TransitionIn, GameState::Run),
        (ForcedGameState::ChipSelect, GameState::Run),
        (ForcedGameState::RunEnd, GameState::Run),
        (ForcedGameState::MetaProgression, GameState::Menu),
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
// map_scenario_dash_state -- maps all variants 1:1
// -------------------------------------------------------------------------

/// Each `ScenarioDashState` variant must map 1:1 to the corresponding
/// `DashState` variant.
#[test]
fn map_scenario_dash_state_maps_all_variants() {
    let mappings = [
        (ScenarioDashState::Idle, DashState::Idle),
        (ScenarioDashState::Dashing, DashState::Dashing),
        (ScenarioDashState::Braking, DashState::Braking),
        (ScenarioDashState::Settling, DashState::Settling),
    ];
    for (scenario, expected) in &mappings {
        let result = map_scenario_dash_state(*scenario);
        assert_eq!(
            result, *expected,
            "map_scenario_dash_state({scenario:?}) must return {expected:?}, got {result:?}"
        );
    }
}
