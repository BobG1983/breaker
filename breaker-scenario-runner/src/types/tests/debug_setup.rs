use super::super::*;

// -------------------------------------------------------------------------
// DebugSetup -- partial fields
// -------------------------------------------------------------------------

#[test]
fn debug_setup_with_bolt_position_only_parses() {
    let ron = "(bolt_position: Some((0.0, -500.0)))";
    let result: DebugSetup =
        ron::de::from_str(ron).expect("DebugSetup with bolt_position should parse");
    assert_eq!(result.bolt_position, Some((0.0_f32, -500.0_f32)));
    assert!(
        !result.disable_physics,
        "disable_physics should default to false"
    );
}

#[test]
fn debug_setup_default_has_no_overrides() {
    let default = DebugSetup::default();
    assert!(default.bolt_position.is_none());
    assert!(!default.disable_physics);
}

// -------------------------------------------------------------------------
// DebugSetup -- bolt_velocity field
// -------------------------------------------------------------------------

#[test]
fn debug_setup_with_bolt_velocity_parses_from_ron() {
    let ron = "(bolt_position: None, bolt_velocity: Some((0.0, 2000.0)))";
    let result: DebugSetup =
        ron::de::from_str(ron).expect("DebugSetup with bolt_velocity should parse");
    assert_eq!(
        result.bolt_velocity,
        Some((0.0_f32, 2000.0_f32)),
        "bolt_velocity must be Some((0.0, 2000.0))"
    );
    assert!(
        result.bolt_position.is_none(),
        "bolt_position must default to None"
    );
    assert!(
        result.breaker_position.is_none(),
        "breaker_position must default to None"
    );
    assert!(
        !result.disable_physics,
        "disable_physics must default to false"
    );
    assert!(
        result.extra_tagged_bolts.is_none(),
        "extra_tagged_bolts must default to None"
    );
    assert!(
        result.node_timer_remaining.is_none(),
        "node_timer_remaining must default to None"
    );
    assert!(
        result.force_previous_game_state.is_none(),
        "force_previous_game_state must default to None"
    );
}

// -------------------------------------------------------------------------
// DebugSetup -- extra_tagged_bolts field
// -------------------------------------------------------------------------

#[test]
fn debug_setup_with_extra_tagged_bolts_parses_from_ron() {
    let ron = "(bolt_position: None, extra_tagged_bolts: Some(12))";
    let result: DebugSetup =
        ron::de::from_str(ron).expect("DebugSetup with extra_tagged_bolts should parse");
    assert_eq!(
        result.extra_tagged_bolts,
        Some(12),
        "extra_tagged_bolts must be Some(12)"
    );
}

// -------------------------------------------------------------------------
// DebugSetup -- node_timer_remaining field
// -------------------------------------------------------------------------

#[test]
fn debug_setup_with_node_timer_remaining_parses_from_ron() {
    let ron = "(bolt_position: None, node_timer_remaining: Some(-1.0))";
    let result: DebugSetup =
        ron::de::from_str(ron).expect("DebugSetup with node_timer_remaining should parse");
    assert_eq!(
        result.node_timer_remaining,
        Some(-1.0_f32),
        "node_timer_remaining must be Some(-1.0)"
    );
}

// -------------------------------------------------------------------------
// ForcedGameState -- RON parsing
// -------------------------------------------------------------------------

#[test]
fn forced_game_state_loading_parses_from_ron() {
    let result: ForcedGameState =
        ron::de::from_str("Loading").expect("ForcedGameState::Loading should parse");
    assert_eq!(result, ForcedGameState::Loading);
}

#[test]
fn forced_game_state_all_variants_parse_from_ron() {
    let variants = [
        ("Loading", ForcedGameState::Loading),
        ("MainMenu", ForcedGameState::MainMenu),
        ("Playing", ForcedGameState::Playing),
        ("RunSetup", ForcedGameState::RunSetup),
        ("TransitionOut", ForcedGameState::TransitionOut),
        ("TransitionIn", ForcedGameState::TransitionIn),
        ("ChipSelect", ForcedGameState::ChipSelect),
        ("RunEnd", ForcedGameState::RunEnd),
        ("MetaProgression", ForcedGameState::MetaProgression),
    ];
    for (ron_str, expected) in &variants {
        let result: ForcedGameState = ron::de::from_str(ron_str)
            .unwrap_or_else(|e| panic!("ForcedGameState::{ron_str} should parse: {e}"));
        assert_eq!(
            result, *expected,
            "ForcedGameState::{ron_str} must parse to {expected:?}"
        );
    }
}

// -------------------------------------------------------------------------
// DebugSetup -- force_previous_game_state field
// -------------------------------------------------------------------------

#[test]
fn debug_setup_with_force_previous_game_state_parses_from_ron() {
    let ron = "(bolt_position: None, force_previous_game_state: Some(Loading))";
    let result: DebugSetup =
        ron::de::from_str(ron).expect("DebugSetup with force_previous_game_state should parse");
    assert_eq!(
        result.force_previous_game_state,
        Some(ForcedGameState::Loading),
        "force_previous_game_state must be Some(ForcedGameState::Loading)"
    );
}

// -------------------------------------------------------------------------
// DebugSetup -- default has all new fields as None
// -------------------------------------------------------------------------

#[test]
fn debug_setup_default_has_all_new_fields_as_none() {
    let default = DebugSetup::default();
    assert!(
        default.bolt_velocity.is_none(),
        "bolt_velocity must default to None"
    );
    assert!(
        default.extra_tagged_bolts.is_none(),
        "extra_tagged_bolts must default to None"
    );
    assert!(
        default.node_timer_remaining.is_none(),
        "node_timer_remaining must default to None"
    );
    assert!(
        default.force_previous_game_state.is_none(),
        "force_previous_game_state must default to None"
    );
}

// -------------------------------------------------------------------------
// DebugSetup -- extra_tagged_bolts zero is valid
// -------------------------------------------------------------------------

#[test]
fn debug_setup_extra_tagged_bolts_zero_parses_from_ron() {
    let ron = "(bolt_position: None, extra_tagged_bolts: Some(0))";
    let result: DebugSetup =
        ron::de::from_str(ron).expect("DebugSetup with extra_tagged_bolts 0 should parse");
    assert_eq!(
        result.extra_tagged_bolts,
        Some(0),
        "extra_tagged_bolts must be Some(0) -- means spawn zero extras"
    );
}
