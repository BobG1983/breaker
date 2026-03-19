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
