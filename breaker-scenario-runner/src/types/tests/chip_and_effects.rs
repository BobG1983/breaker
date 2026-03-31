use super::super::*;

// -------------------------------------------------------------------------
// ScenarioDefinition — chip_selections field
// -------------------------------------------------------------------------

#[test]
fn chip_selections_parses_from_ron() {
    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((action_prob: 0.1)),
        max_frames: 1000,
        disallowed_failures: [],
        allowed_failures: None,
        debug_setup: None,
        chip_selections: Some(["Surge", "Surge"]),
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition with chip_selections should parse");
    assert_eq!(
        result.chip_selections,
        Some(vec!["Surge".to_owned(), "Surge".to_owned()]),
        "chip_selections must be Some([\"Surge\", \"Surge\"])"
    );
}

#[test]
fn chip_selections_defaults_to_none() {
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
        ron::de::from_str(ron).expect("ScenarioDefinition without chip_selections should parse");
    assert!(
        result.chip_selections.is_none(),
        "chip_selections must be None when omitted from RON"
    );
}

// -------------------------------------------------------------------------
// ScenarioDefinition — initial_effects field
// -------------------------------------------------------------------------

#[test]
fn initial_effects_parses_from_ron() {
    use breaker::effect::{EffectKind, EffectNode, RootEffect, Target};

    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((action_prob: 0.1)),
        max_frames: 1000,
        disallowed_failures: [],
        allowed_failures: None,
        debug_setup: None,
        initial_effects: Some([On(target: Bolt, then: [Do(Piercing(1))])]),
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition with initial_effects should parse");
    let effects = result
        .initial_effects
        .expect("initial_effects must be Some");
    assert_eq!(effects.len(), 1, "expected 1 root effect");
    assert_eq!(
        effects[0],
        RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::Piercing(1))],
        },
        "initial_effects must contain On(target: Bolt, then: [Do(Piercing(1))])"
    );
}

#[test]
fn initial_effects_defaults_to_none() {
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
        ron::de::from_str(ron).expect("ScenarioDefinition without initial_effects should parse");
    assert!(
        result.initial_effects.is_none(),
        "initial_effects must be None when omitted from RON"
    );
}
