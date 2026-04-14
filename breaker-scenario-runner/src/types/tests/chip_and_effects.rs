use crate::types::*;

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
    use breaker::effect_v3::{
        effects::PiercingConfig,
        types::{EffectType, RootNode, StampTarget, Tree},
    };

    let ron = r#"(
        breaker: "aegis",
        layout: "corridor",
        input: Chaos((action_prob: 0.1)),
        max_frames: 1000,
        disallowed_failures: [],
        allowed_failures: None,
        debug_setup: None,
        initial_effects: Some([Stamp(Bolt, Fire(Piercing((charges: 1))))]),
    )"#;
    let result: ScenarioDefinition =
        ron::de::from_str(ron).expect("ScenarioDefinition with initial_effects should parse");
    let effects = result
        .initial_effects
        .expect("initial_effects must be Some");
    assert_eq!(effects.len(), 1, "expected 1 root effect");
    assert_eq!(
        effects[0],
        RootNode::Stamp(
            StampTarget::Bolt,
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
        ),
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

// -------------------------------------------------------------------------
// Scenario file parse coverage — TetherBeam width field (behaviors 17 & 18)
// -------------------------------------------------------------------------

/// Behavior 17: `tether_beam_stress.scenario.ron` parses successfully, its
/// `initial_effects` contains a `Fire(TetherBeam(TetherBeamConfig))` node
/// with the expected shape, and `TetherBeamConfig` parses with
/// `damage_mult = 1.5`, `chain = false`, and `width = 10.0`.
#[test]
fn tether_beam_stress_scenario_parses_with_tether_beam_config_width_field() {
    use std::path::PathBuf;

    use breaker::effect_v3::types::{EffectType, RootNode, StampTarget, Tree, Trigger};

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scenarios/stress/tether_beam_stress.scenario.ron");
    let content =
        std::fs::read_to_string(&path).expect("tether_beam_stress.scenario.ron must be readable");
    let result: ScenarioDefinition = ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
        .from_str(&content)
        .expect("tether_beam_stress.scenario.ron must parse as ScenarioDefinition after Wave 6");

    let effects = result
        .initial_effects
        .expect("tether_beam_stress must declare initial_effects");
    assert_eq!(
        effects.len(),
        1,
        "expected 1 root effect in initial_effects"
    );

    // Expected tree shape: Stamp(Bolt, When(Bumped, Fire(TetherBeam(..))))
    let RootNode::Stamp(stamp_target, tree) = &effects[0] else {
        panic!("expected RootNode::Stamp(..), got {:?}", effects[0]);
    };
    assert_eq!(
        *stamp_target,
        StampTarget::Bolt,
        "tether_beam_stress must stamp on Bolt"
    );
    let Tree::When(trigger, inner) = tree else {
        panic!("expected Tree::When(..), got {tree:?}");
    };
    assert_eq!(
        *trigger,
        Trigger::Bumped,
        "tether_beam_stress must trigger on Bumped"
    );
    let Tree::Fire(EffectType::TetherBeam(parsed_cfg)) = inner.as_ref() else {
        panic!("expected Fire(TetherBeam(..)), got {inner:?}");
    };

    // Verify each field on the parsed TetherBeamConfig directly.
    assert!(
        (parsed_cfg.damage_mult.0 - 1.5).abs() < 1e-6,
        "parsed damage_mult must be 1.5, got {}",
        parsed_cfg.damage_mult.0,
    );
    assert!(
        !parsed_cfg.chain,
        "parsed chain must be false (fire_spawn branch), got {}",
        parsed_cfg.chain,
    );
    assert!(
        (parsed_cfg.width.0 - 10.0).abs() < 1e-6,
        "parsed width must be 10.0, got {}",
        parsed_cfg.width.0,
    );
}

/// Behavior 18: `tether_chain_chaos.scenario.ron` (chip-selection mode)
/// continues to parse as a `ScenarioDefinition` after Wave 6. No RON edit
/// is needed for this file — it uses `chip_selections: ["Arcwelder"]` and
/// does not declare an inline `TetherBeamConfig` literal. The
/// `arcwelder.evolution.ron` update is what propagates the width field at
/// runtime; this test simply locks that the scenario RON envelope still
/// loads.
#[test]
fn tether_chain_chaos_scenario_still_parses_after_wave_6() {
    use std::path::PathBuf;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scenarios/stress/tether_chain_chaos.scenario.ron");
    let content =
        std::fs::read_to_string(&path).expect("tether_chain_chaos.scenario.ron must be readable");
    let result: ScenarioDefinition = ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
        .from_str(&content)
        .expect("tether_chain_chaos.scenario.ron must parse as ScenarioDefinition");

    assert_eq!(
        result.chip_selections,
        Some(vec!["Arcwelder".to_owned()]),
        "tether_chain_chaos must request Arcwelder via chip_selections"
    );
}
