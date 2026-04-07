use crate::types::*;

// -------------------------------------------------------------------------
// InputStrategy — Chaos
// -------------------------------------------------------------------------

#[test]
fn chaos_input_strategy_parses_from_ron() {
    // RON newtype-variant syntax: Chaos((field: val, ...))
    let ron = "Chaos((action_prob: 0.3))";
    let result: InputStrategy = ron::de::from_str(ron).expect("Chaos should parse");
    assert_eq!(
        result,
        InputStrategy::Chaos(ChaosParams { action_prob: 0.3 })
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
    let ron = "Hybrid((scripted_frames: 100, action_prob: 0.5))";
    let result: InputStrategy = ron::de::from_str(ron).expect("Hybrid should parse");
    assert_eq!(
        result,
        InputStrategy::Hybrid(HybridParams {
            scripted_frames: 100,
            action_prob: 0.5,
        })
    );
}

// -------------------------------------------------------------------------
// ChaosParams — seed removed
// -------------------------------------------------------------------------

#[test]
fn chaos_params_without_seed_parses() {
    let ron = "(action_prob: 0.3)";
    let result: ChaosParams =
        ron::de::from_str(ron).expect("ChaosParams without seed should parse");
    assert_eq!(
        result,
        ChaosParams { action_prob: 0.3 },
        "ChaosParams must contain only action_prob"
    );
}

// -------------------------------------------------------------------------
// HybridParams — seed removed
// -------------------------------------------------------------------------

#[test]
fn hybrid_params_without_seed_parses() {
    let ron = "(scripted_frames: 100, action_prob: 0.5)";
    let result: HybridParams =
        ron::de::from_str(ron).expect("HybridParams without seed should parse");
    assert_eq!(
        result,
        HybridParams {
            scripted_frames: 100,
            action_prob: 0.5,
        },
        "HybridParams must contain only scripted_frames and action_prob"
    );
}

// -------------------------------------------------------------------------
// InputStrategy — Perfect variant
// -------------------------------------------------------------------------

#[test]
fn input_strategy_perfect_parses() {
    let ron = "Perfect(AlwaysPerfect)";
    let result: InputStrategy =
        ron::de::from_str(ron).expect("InputStrategy::Perfect(AlwaysPerfect) should parse");
    assert_eq!(
        result,
        InputStrategy::Perfect(BumpMode::AlwaysPerfect),
        "must parse to Perfect(AlwaysPerfect)"
    );
}

// -------------------------------------------------------------------------
// BumpMode — all variants
// -------------------------------------------------------------------------

#[test]
fn bump_mode_all_variants_parse() {
    let variants = [
        ("AlwaysPerfect", BumpMode::AlwaysPerfect),
        ("AlwaysEarly", BumpMode::AlwaysEarly),
        ("AlwaysLate", BumpMode::AlwaysLate),
        ("AlwaysWhiff", BumpMode::AlwaysWhiff),
        ("NeverBump", BumpMode::NeverBump),
        ("Random", BumpMode::Random),
    ];
    for (ron_str, expected) in &variants {
        let result: BumpMode = ron::de::from_str(ron_str)
            .unwrap_or_else(|e| panic!("BumpMode::{ron_str} should parse: {e}"));
        assert_eq!(
            result, *expected,
            "BumpMode::{ron_str} must parse to {expected:?}"
        );
    }
}
