use super::types::*;

#[test]
fn from_root_effect_on_for_effect_node_sets_permanent_false() {
    let root = RootEffect::On {
        target: Target::Bolt,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
    };

    let node = EffectNode::from(root);

    assert_eq!(
        node,
        EffectNode::On {
            target: Target::Bolt,
            permanent: false,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
        },
        "From<RootEffect> should produce On with permanent=false"
    );
}

#[test]
fn from_root_effect_on_all_cells_with_empty_then() {
    let root = RootEffect::On {
        target: Target::AllCells,
        then: vec![],
    };

    let node = EffectNode::from(root);

    assert_eq!(
        node,
        EffectNode::On {
            target: Target::AllCells,
            permanent: false,
            then: vec![],
        },
        "From<RootEffect> with AllCells and empty then should produce On with permanent=false"
    );
}

#[test]
fn from_root_effect_on_breaker_with_nested_children_preserves_structure() {
    let nested_children = vec![EffectNode::When {
        trigger: Trigger::Bump,
        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
    }];
    let root = RootEffect::On {
        target: Target::Breaker,
        then: nested_children.clone(),
    };

    let node = EffectNode::from(root);

    assert_eq!(
        node,
        EffectNode::On {
            target: Target::Breaker,
            permanent: false,
            then: nested_children,
        },
        "From<RootEffect> with Breaker should preserve nested children with permanent=false"
    );
}

// -- TetherBeam chain field serde tests --

#[test]
fn tether_beam_serde_with_chain_true() {
    let ron_str = "TetherBeam(damage_mult: 1.5, chain: true)";
    let effect: EffectKind =
        ron::from_str(ron_str).expect("should deserialize TetherBeam with chain: true");

    match &effect {
        EffectKind::TetherBeam { damage_mult, chain } => {
            assert!(
                (*damage_mult - 1.5).abs() < f32::EPSILON,
                "expected damage_mult 1.5, got {damage_mult}"
            );
            assert!(*chain, "expected chain true, got {chain}");
        }
        other => panic!("expected TetherBeam variant, got {other:?}"),
    }
}

#[test]
fn tether_beam_serde_defaults_chain_to_false_when_omitted() {
    let ron_str = "TetherBeam(damage_mult: 2.0)";
    let effect: EffectKind =
        ron::from_str(ron_str).expect("should deserialize TetherBeam with omitted chain");

    match &effect {
        EffectKind::TetherBeam { damage_mult, chain } => {
            assert!(
                (*damage_mult - 2.0).abs() < f32::EPSILON,
                "expected damage_mult 2.0, got {damage_mult}"
            );
            assert!(!*chain, "expected chain false (serde default), got {chain}");
        }
        other => panic!("expected TetherBeam variant, got {other:?}"),
    }
}
