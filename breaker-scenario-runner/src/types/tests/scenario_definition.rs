use super::super::*;

// -------------------------------------------------------------------------
// ScenarioDefinition — allowed_failures field
// -------------------------------------------------------------------------

#[test]
fn scenario_definition_allowed_failures_some_parses() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((action_prob: 0.1)),
        max_frames: 1000,
        disallowed_failures: [BoltInBounds, NoNaN],
        allowed_failures: Some([BoltInBounds, NoNaN]),
        debug_setup: None,
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition with Some violations should parse");
    assert_eq!(
        result.allowed_failures,
        Some(vec![InvariantKind::BoltInBounds, InvariantKind::NoNaN])
    );
}

#[test]
fn scenario_definition_allowed_failures_none_parses() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((action_prob: 0.1)),
        max_frames: 1000,
        disallowed_failures: [],
        allowed_failures: None,
        debug_setup: None,
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition with None violations should parse");
    assert!(result.allowed_failures.is_none());
}

// -------------------------------------------------------------------------
// ScenarioDefinition — full round-trip
// -------------------------------------------------------------------------

#[test]
fn full_scenario_definition_parses_all_fields() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((action_prob: 0.25)),
        max_frames: 20000,
        disallowed_failures: [BoltInBounds, NoNaN],
        allowed_failures: None,
        debug_setup: None,
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("full ScenarioDefinition should parse");

    assert_eq!(result.breaker, "aegis");
    assert_eq!(result.layout, "corridor");
    assert_eq!(
        result.input,
        InputStrategy::Chaos(ChaosParams { action_prob: 0.25 })
    );
    assert_eq!(result.max_frames, 20_000);
    assert_eq!(
        result.disallowed_failures,
        vec![InvariantKind::BoltInBounds, InvariantKind::NoNaN]
    );
    assert!(result.allowed_failures.is_none());
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
        input: Chaos((action_prob: 0.1)),
        max_frames: 1000,
        disallowed_failures: [],
        allowed_failures: None,
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
// ScenarioDefinition — seed field
// -------------------------------------------------------------------------

#[test]
fn scenario_definition_seed_defaults_to_none_when_omitted() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((action_prob: 0.1)),
        max_frames: 1000,
        disallowed_failures: [],
        allowed_failures: None,
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
        input: Chaos((action_prob: 0.1)),
        max_frames: 1000,
        disallowed_failures: [],
        allowed_failures: None,
        debug_setup: None,
        seed: Some(42),
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition with seed Some(42) should parse");
    assert_eq!(result.seed, Some(42));
}

// -------------------------------------------------------------------------
// ScenarioDefinition — stress field
// -------------------------------------------------------------------------

#[test]
fn scenario_definition_stress_field_defaults_to_none_when_omitted() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((action_prob: 0.1)),
        max_frames: 1000,
        disallowed_failures: [],
        allowed_failures: None,
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
        input: Chaos((action_prob: 0.1)),
        max_frames: 1000,
        disallowed_failures: [],
        allowed_failures: None,
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
        input: Chaos((action_prob: 0.1)),
        max_frames: 1000,
        disallowed_failures: [],
        allowed_failures: None,
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
// ScenarioDefinition — initial_chips field
// -------------------------------------------------------------------------

#[test]
fn scenario_definition_initial_chips_single_parses() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((action_prob: 0.1)),
        max_frames: 1000,
        disallowed_failures: [],
        allowed_failures: None,
        debug_setup: None,
        initial_chips: Some(["Surge"]),
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition with initial_chips should parse");
    let chips = result.initial_chips.expect("initial_chips must be Some");
    assert_eq!(chips.len(), 1, "expected 1 chip name");
    assert_eq!(chips[0], "Surge", "chip name must be 'Surge'");
}

#[test]
fn scenario_definition_initial_chips_multiple_parses() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((action_prob: 0.1)),
        max_frames: 1000,
        disallowed_failures: [],
        allowed_failures: None,
        debug_setup: None,
        initial_chips: Some(["Surge", "Piercing Shot"]),
    )"#;
    let result: ScenarioDefinition = ron::de::from_str(ron)
        .expect("ScenarioDefinition with multiple initial_chips should parse");
    let chips = result.initial_chips.expect("initial_chips must be Some");
    assert_eq!(chips.len(), 2, "expected 2 chip names");
    assert_eq!(chips[0], "Surge", "first chip must be 'Surge'");
    assert_eq!(
        chips[1], "Piercing Shot",
        "second chip must be 'Piercing Shot'"
    );
}

#[test]
fn scenario_definition_initial_chips_defaults_to_none() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((action_prob: 0.1)),
        max_frames: 1000,
        disallowed_failures: [],
        allowed_failures: None,
        debug_setup: None,
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition without initial_chips should parse");
    assert!(
        result.initial_chips.is_none(),
        "initial_chips must be None when omitted from RON"
    );
}

// -------------------------------------------------------------------------
// ScenarioDefinition — frame_mutations field
// -------------------------------------------------------------------------

#[test]
fn scenario_definition_with_frame_mutations_parses_from_ron() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((action_prob: 0.1)),
        max_frames: 1000,
        disallowed_failures: [],
        allowed_failures: None,
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
        input: Chaos((action_prob: 0.1)),
        max_frames: 1000,
        disallowed_failures: [],
        allowed_failures: None,
        debug_setup: None,
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition without frame_mutations should parse");
    assert!(
        result.frame_mutations.is_none(),
        "frame_mutations must be None when omitted from RON"
    );
}
