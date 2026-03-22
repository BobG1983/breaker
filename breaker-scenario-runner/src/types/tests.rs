use super::*;

// -------------------------------------------------------------------------
// InputStrategy — Chaos
// -------------------------------------------------------------------------

#[test]
fn chaos_input_strategy_parses_from_ron() {
    // RON newtype-variant syntax: Chaos((field: val, ...))
    let ron = "Chaos((seed: 42, action_prob: 0.3))";
    let result: InputStrategy = ron::de::from_str(ron).expect("Chaos should parse");
    assert_eq!(
        result,
        InputStrategy::Chaos(ChaosParams {
            seed: 42,
            action_prob: 0.3,
        })
    );
}

// -------------------------------------------------------------------------
// InputStrategy — Scripted
// -------------------------------------------------------------------------

#[test]
fn scripted_input_strategy_parses_from_ron() {
    let ron = r"Scripted((actions: [
        (frame: 10, actions: [MoveLeft]),
        (frame: 20, actions: [Bump, MoveRight]),
    ]))";

    let result: InputStrategy = ron::de::from_str(ron).expect("Scripted should parse");
    match result {
        InputStrategy::Scripted(params) => {
            assert_eq!(params.actions.len(), 2, "expected 2 scripted entries");
            assert_eq!(params.actions[0].frame, 10);
            assert_eq!(params.actions[0].actions, vec![GameAction::MoveLeft]);
            assert_eq!(params.actions[1].frame, 20);
            assert_eq!(
                params.actions[1].actions,
                vec![GameAction::Bump, GameAction::MoveRight]
            );
        }
        other => panic!("expected Scripted variant, got {other:?}"),
    }
}

#[test]
fn scripted_input_strategy_empty_actions_list_parses() {
    let ron = "Scripted((actions: []))";
    let result: InputStrategy = ron::de::from_str(ron).expect("empty Scripted should parse");
    assert_eq!(
        result,
        InputStrategy::Scripted(ScriptedParams { actions: vec![] })
    );
}

// -------------------------------------------------------------------------
// InputStrategy — Hybrid
// -------------------------------------------------------------------------

#[test]
fn hybrid_input_strategy_parses_from_ron() {
    let ron = "Hybrid((scripted_frames: 100, seed: 7, action_prob: 0.5))";
    let result: InputStrategy = ron::de::from_str(ron).expect("Hybrid should parse");
    assert_eq!(
        result,
        InputStrategy::Hybrid(HybridParams {
            scripted_frames: 100,
            seed: 7,
            action_prob: 0.5,
        })
    );
}

// -------------------------------------------------------------------------
// InvariantKind — all variants
// -------------------------------------------------------------------------

#[test]
fn invariant_kind_bolt_in_bounds_parses() {
    let result: InvariantKind =
        ron::de::from_str("BoltInBounds").expect("BoltInBounds should parse");
    assert_eq!(result, InvariantKind::BoltInBounds);
}

#[test]
fn invariant_kind_breaker_in_bounds_parses() {
    let result: InvariantKind =
        ron::de::from_str("BreakerInBounds").expect("BreakerInBounds should parse");
    assert_eq!(result, InvariantKind::BreakerInBounds);
}

#[test]
fn invariant_kind_no_entity_leaks_parses() {
    let result: InvariantKind =
        ron::de::from_str("NoEntityLeaks").expect("NoEntityLeaks should parse");
    assert_eq!(result, InvariantKind::NoEntityLeaks);
}

#[test]
fn invariant_kind_no_nan_parses() {
    let result: InvariantKind = ron::de::from_str("NoNaN").expect("NoNaN should parse");
    assert_eq!(result, InvariantKind::NoNaN);
}

#[test]
fn invariant_kind_valid_state_transitions_parses() {
    let result: InvariantKind =
        ron::de::from_str("ValidStateTransitions").expect("ValidStateTransitions should parse");
    assert_eq!(result, InvariantKind::ValidStateTransitions);
}

// -------------------------------------------------------------------------
// ScenarioDefinition — expected_violations field
// -------------------------------------------------------------------------

#[test]
fn scenario_definition_expected_violations_some_parses() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((seed: 1, action_prob: 0.1)),
        max_frames: 1000,
        invariants: [BoltInBounds, NoNaN],
        expected_violations: Some([BoltInBounds, NoNaN]),
        debug_setup: None,
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition with Some violations should parse");
    assert_eq!(
        result.expected_violations,
        Some(vec![InvariantKind::BoltInBounds, InvariantKind::NoNaN])
    );
}

#[test]
fn scenario_definition_expected_violations_none_parses() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((seed: 1, action_prob: 0.1)),
        max_frames: 1000,
        invariants: [],
        expected_violations: None,
        debug_setup: None,
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition with None violations should parse");
    assert!(result.expected_violations.is_none());
}

// -------------------------------------------------------------------------
// DebugSetup — partial fields
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
// ScenarioDefinition — full round-trip
// -------------------------------------------------------------------------

#[test]
fn full_scenario_definition_parses_all_fields() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((seed: 99, action_prob: 0.25)),
        max_frames: 20000,
        invariants: [BoltInBounds, NoNaN],
        expected_violations: None,
        debug_setup: None,
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("full ScenarioDefinition should parse");

    assert_eq!(result.breaker, "aegis");
    assert_eq!(result.layout, "corridor");
    assert_eq!(
        result.input,
        InputStrategy::Chaos(ChaosParams {
            seed: 99,
            action_prob: 0.25,
        })
    );
    assert_eq!(result.max_frames, 20_000);
    assert_eq!(
        result.invariants,
        vec![InvariantKind::BoltInBounds, InvariantKind::NoNaN]
    );
    assert!(result.expected_violations.is_none());
    assert!(result.debug_setup.is_none());
}

// -------------------------------------------------------------------------
// ScenarioDefinition — allow_early_end defaults to true
// -------------------------------------------------------------------------

#[test]
fn scenario_definition_allow_early_end_defaults_to_true() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((seed: 1, action_prob: 0.1)),
        max_frames: 1000,
        invariants: [],
        expected_violations: None,
        debug_setup: None,
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition without allow_early_end should parse");
    assert!(
        result.allow_early_end,
        "allow_early_end must default to true when omitted"
    );
}

// -------------------------------------------------------------------------
// InvariantKind::fail_reason — each variant returns non-empty string
// -------------------------------------------------------------------------

#[test]
fn fail_reason_returns_non_empty_string_for_every_variant() {
    for variant in InvariantKind::ALL {
        let reason = variant.fail_reason();
        assert!(
            !reason.is_empty(),
            "fail_reason() for {variant:?} must not be empty"
        );
    }
}

#[test]
fn fail_reason_returns_distinct_strings_for_each_variant() {
    let reasons: Vec<&str> = InvariantKind::ALL
        .iter()
        .map(InvariantKind::fail_reason)
        .collect();
    let unique: std::collections::HashSet<&str> = reasons.iter().copied().collect();
    assert_eq!(
        reasons.len(),
        unique.len(),
        "fail_reason() must return distinct strings — found duplicates in: {reasons:?}"
    );
}

#[test]
fn all_variants_covered_by_invariant_kind_all() {
    // If a new variant is added to InvariantKind, fail_reason()'s exhaustive
    // match forces a compile error. This test ensures ALL has the right count.
    let unique: std::collections::HashSet<InvariantKind> =
        InvariantKind::ALL.iter().copied().collect();
    assert_eq!(
        InvariantKind::ALL.len(),
        unique.len(),
        "InvariantKind::ALL must not contain duplicates"
    );
}

// -------------------------------------------------------------------------
// ScenarioDefinition — seed field
// -------------------------------------------------------------------------

#[test]
fn scenario_definition_seed_defaults_to_none_when_omitted() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((seed: 1, action_prob: 0.1)),
        max_frames: 1000,
        invariants: [],
        expected_violations: None,
        debug_setup: None,
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition without seed should parse");
    assert!(
        result.seed.is_none(),
        "seed must be None when omitted from RON"
    );
}

#[test]
fn scenario_definition_seed_some_value_parses() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((seed: 1, action_prob: 0.1)),
        max_frames: 1000,
        invariants: [],
        expected_violations: None,
        debug_setup: None,
        seed: Some(42),
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition with seed Some(42) should parse");
    assert_eq!(result.seed, Some(42));
}

// -------------------------------------------------------------------------
// StressConfig — serde deserialization
// -------------------------------------------------------------------------

#[test]
fn stress_config_parses_full_ron() {
    let ron = "(runs: 64, parallelism: 8)";
    let result: StressConfig = ron::de::from_str(ron).expect("StressConfig full should parse");
    assert_eq!(
        result,
        StressConfig {
            runs: 64,
            parallelism: 8,
        }
    );
}

#[test]
fn stress_config_defaults_both_fields_from_empty_struct() {
    let ron = "()";
    let result: StressConfig =
        ron::de::from_str(ron).expect("StressConfig empty struct should parse");
    assert_eq!(
        result,
        StressConfig {
            runs: 32,
            parallelism: 32,
        }
    );
}

#[test]
fn stress_config_partial_override_only_runs() {
    let ron = "(runs: 64)";
    let result: StressConfig = ron::de::from_str(ron).expect("StressConfig runs-only should parse");
    assert_eq!(
        result,
        StressConfig {
            runs: 64,
            parallelism: 32,
        }
    );
}

#[test]
fn stress_config_partial_override_only_parallelism() {
    let ron = "(parallelism: 4)";
    let result: StressConfig =
        ron::de::from_str(ron).expect("StressConfig parallelism-only should parse");
    assert_eq!(
        result,
        StressConfig {
            runs: 32,
            parallelism: 4,
        }
    );
}

// -------------------------------------------------------------------------
// ScenarioDefinition — stress field
// -------------------------------------------------------------------------

#[test]
fn scenario_definition_stress_field_defaults_to_none_when_omitted() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((seed: 1, action_prob: 0.1)),
        max_frames: 1000,
        invariants: [],
        expected_violations: None,
        debug_setup: None,
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition without stress should parse");
    assert!(
        result.stress.is_none(),
        "stress must be None when omitted from RON"
    );
}

#[test]
fn scenario_definition_stress_some_with_explicit_values_parses() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((seed: 1, action_prob: 0.1)),
        max_frames: 1000,
        invariants: [],
        expected_violations: None,
        debug_setup: None,
        stress: Some((runs: 64, parallelism: 4)),
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition with stress Some should parse");
    assert_eq!(
        result.stress,
        Some(StressConfig {
            runs: 64,
            parallelism: 4,
        })
    );
}

#[test]
fn scenario_definition_stress_some_empty_uses_defaults() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((seed: 1, action_prob: 0.1)),
        max_frames: 1000,
        invariants: [],
        expected_violations: None,
        debug_setup: None,
        stress: Some(()),
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition with stress Some(()) should parse");
    assert_eq!(
        result.stress,
        Some(StressConfig {
            runs: 32,
            parallelism: 32,
        })
    );
}

// -------------------------------------------------------------------------
// ScenarioDefinition — initial_overclocks field
// -------------------------------------------------------------------------

#[test]
fn scenario_definition_initial_overclocks_single_surge_chain_parses() {
    use breaker::chips::{ImpactTarget, TriggerChain};

    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((seed: 1, action_prob: 0.1)),
        max_frames: 1000,
        invariants: [],
        expected_violations: None,
        debug_setup: None,
        initial_overclocks: Some([OnPerfectBump(OnImpact(Cell, Shockwave(base_range: 64.0, range_per_level: 32.0, stacks: 1)))]),
    )"#;
    let result: ScenarioDefinition = ron::de::from_str(ron)
        .expect("ScenarioDefinition with initial_overclocks surge chain should parse");
    let overclocks = result
        .initial_overclocks
        .expect("initial_overclocks must be Some");
    assert_eq!(overclocks.len(), 1, "expected 1 overclock chain");
    assert_eq!(
        overclocks[0],
        TriggerChain::OnPerfectBump(Box::new(TriggerChain::OnImpact(
            ImpactTarget::Cell,
            Box::new(TriggerChain::Shockwave {
                base_range: 64.0,
                range_per_level: 32.0,
                stacks: 1
            })
        ))),
        "overclock chain must match OnPerfectBump(OnImpact(Cell, Shockwave))"
    );
}

#[test]
fn scenario_definition_initial_overclocks_multiple_parses() {
    use breaker::chips::TriggerChain;

    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((seed: 1, action_prob: 0.1)),
        max_frames: 1000,
        invariants: [],
        expected_violations: None,
        debug_setup: None,
        initial_overclocks: Some([Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1), MultiBolt(base_count: 3, count_per_level: 0, stacks: 1)]),
    )"#;
    let result: ScenarioDefinition = ron::de::from_str(ron)
        .expect("ScenarioDefinition with multiple initial_overclocks should parse");
    let overclocks = result
        .initial_overclocks
        .expect("initial_overclocks must be Some");
    assert_eq!(overclocks.len(), 2, "expected 2 overclock chains");
    assert_eq!(
        overclocks[0],
        TriggerChain::Shockwave {
            base_range: 64.0,
            range_per_level: 0.0,
            stacks: 1
        },
        "first overclock must be Shockwave"
    );
    assert_eq!(
        overclocks[1],
        TriggerChain::MultiBolt {
            base_count: 3,
            count_per_level: 0,
            stacks: 1
        },
        "second overclock must be MultiBolt"
    );
}

#[test]
fn scenario_definition_initial_overclocks_defaults_to_none() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((seed: 1, action_prob: 0.1)),
        max_frames: 1000,
        invariants: [],
        expected_violations: None,
        debug_setup: None,
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition without initial_overclocks should parse");
    assert!(
        result.initial_overclocks.is_none(),
        "initial_overclocks must be None when omitted from RON"
    );
}

// -------------------------------------------------------------------------
// DebugSetup — bolt_velocity field
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
// DebugSetup — extra_tagged_bolts field
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
// DebugSetup — node_timer_remaining field
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
// ForcedGameState — RON parsing
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
// DebugSetup — force_previous_game_state field
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
// DebugSetup — default has all new fields as None
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
// DebugSetup — extra_tagged_bolts zero is valid
// -------------------------------------------------------------------------

#[test]
fn debug_setup_extra_tagged_bolts_zero_parses_from_ron() {
    let ron = "(bolt_position: None, extra_tagged_bolts: Some(0))";
    let result: DebugSetup =
        ron::de::from_str(ron).expect("DebugSetup with extra_tagged_bolts 0 should parse");
    assert_eq!(
        result.extra_tagged_bolts,
        Some(0),
        "extra_tagged_bolts must be Some(0) — means spawn zero extras"
    );
}

// -------------------------------------------------------------------------
// FrameMutation — RON deserialization
// -------------------------------------------------------------------------

#[test]
fn frame_mutation_set_breaker_state_parses_from_ron() {
    let ron = "(frame: 3, mutation: SetBreakerState(Braking))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation SetBreakerState should parse");
    assert_eq!(result.frame, 3);
    assert_eq!(
        result.mutation,
        MutationKind::SetBreakerState(ScenarioBreakerState::Braking)
    );
}

#[test]
fn frame_mutation_set_timer_remaining_parses_from_ron() {
    let ron = "(frame: 5, mutation: SetTimerRemaining(61.0))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation SetTimerRemaining should parse");
    assert_eq!(result.frame, 5);
    assert_eq!(result.mutation, MutationKind::SetTimerRemaining(61.0));
}

#[test]
fn frame_mutation_spawn_extra_entities_parses_from_ron() {
    let ron = "(frame: 119, mutation: SpawnExtraEntities(200))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation SpawnExtraEntities should parse");
    assert_eq!(result.frame, 119);
    assert_eq!(result.mutation, MutationKind::SpawnExtraEntities(200));
}

#[test]
fn frame_mutation_move_bolt_parses_from_ron() {
    let ron = "(frame: 5, mutation: MoveBolt(999.0, 999.0))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation MoveBolt should parse");
    assert_eq!(result.frame, 5);
    assert_eq!(result.mutation, MutationKind::MoveBolt(999.0, 999.0));
}

#[test]
fn frame_mutation_toggle_pause_parses_from_ron() {
    let ron = "(frame: 3, mutation: TogglePause)";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation TogglePause should parse");
    assert_eq!(result.frame, 3);
    assert_eq!(result.mutation, MutationKind::TogglePause);
}

// -------------------------------------------------------------------------
// ScenarioBreakerState — all variants parse from RON
// -------------------------------------------------------------------------

#[test]
fn scenario_breaker_state_all_variants_parse_from_ron() {
    let variants = [
        ("Idle", ScenarioBreakerState::Idle),
        ("Dashing", ScenarioBreakerState::Dashing),
        ("Braking", ScenarioBreakerState::Braking),
        ("Settling", ScenarioBreakerState::Settling),
    ];
    for (ron_str, expected) in &variants {
        let result: ScenarioBreakerState = ron::de::from_str(ron_str)
            .unwrap_or_else(|e| panic!("ScenarioBreakerState::{ron_str} should parse: {e}"));
        assert_eq!(
            result, *expected,
            "ScenarioBreakerState::{ron_str} must parse to {expected:?}"
        );
    }
}

// -------------------------------------------------------------------------
// ScenarioDefinition — frame_mutations field
// -------------------------------------------------------------------------

#[test]
fn scenario_definition_with_frame_mutations_parses_from_ron() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((seed: 1, action_prob: 0.1)),
        max_frames: 1000,
        invariants: [],
        expected_violations: None,
        debug_setup: None,
        frame_mutations: Some([(frame: 3, mutation: TogglePause)]),
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition with frame_mutations should parse");
    assert_eq!(
        result.frame_mutations,
        Some(vec![FrameMutation {
            frame: 3,
            mutation: MutationKind::TogglePause,
        }])
    );
}

#[test]
fn scenario_definition_without_frame_mutations_defaults_to_none() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((seed: 1, action_prob: 0.1)),
        max_frames: 1000,
        invariants: [],
        expected_violations: None,
        debug_setup: None,
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition without frame_mutations should parse");
    assert!(
        result.frame_mutations.is_none(),
        "frame_mutations must be None when omitted from RON"
    );
}
