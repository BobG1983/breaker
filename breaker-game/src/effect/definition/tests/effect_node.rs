use super::super::*;

// =========================================================================
// C7 Wave 1 Part A: EffectNode construction (behaviors 1-6)
// =========================================================================

#[test]
fn effect_node_when_wraps_trigger_and_children() {
    let node = EffectNode::When {
        trigger: Trigger::PerfectBump,
        then: vec![EffectNode::Do(Effect::Shockwave {
            base_range: 64.0,
            range_per_level: 0.0,
            stacks: 1,
            speed: 400.0,
        })],
    };
    match &node {
        EffectNode::When {
            trigger: Trigger::PerfectBump,
            then,
        } => {
            assert_eq!(then.len(), 1);
        }
        other => panic!("expected When(PerfectBump, _), got {other:?}"),
    }
}

#[test]
fn effect_node_when_empty_then_is_valid() {
    let node = EffectNode::When {
        trigger: Trigger::Bump,
        then: vec![],
    };
    match &node {
        EffectNode::When {
            trigger: Trigger::Bump,
            then,
        } => {
            assert!(then.is_empty());
        }
        other => panic!("expected When(Bump, []), got {other:?}"),
    }
}

#[test]
fn effect_node_do_wraps_effect_leaf() {
    let node = EffectNode::Do(Effect::LoseLife);
    assert!(matches!(node, EffectNode::Do(Effect::LoseLife)));
}

#[test]
fn effect_node_do_wraps_spawn_bolts() {
    let node = EffectNode::Do(Effect::SpawnBolts {
        count: 1,
        lifespan: None,
        inherit: false,
    });
    assert!(matches!(
        node,
        EffectNode::Do(Effect::SpawnBolts { count: 1, .. })
    ));
}

#[test]
fn effect_node_until_wraps_trigger_and_children() {
    let node = EffectNode::Until {
        until: Trigger::TimeExpires(3.0),
        then: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
    };
    match &node {
        EffectNode::Until { until, then } => {
            assert_eq!(*until, Trigger::TimeExpires(3.0));
            assert_eq!(then.len(), 1);
        }
        other => panic!("expected Until(TimeExpires(3.0), _), got {other:?}"),
    }
}

#[test]
fn effect_node_until_with_impact_breaker_removal() {
    let node = EffectNode::Until {
        until: Trigger::Impact(ImpactTarget::Breaker),
        then: vec![],
    };
    assert!(matches!(
        node,
        EffectNode::Until {
            until: Trigger::Impact(ImpactTarget::Breaker),
            ..
        }
    ));
}

#[test]
fn effect_node_once_wraps_children() {
    let node = EffectNode::Once(vec![EffectNode::Do(Effect::SecondWind {
        invuln_secs: 3.0,
    })]);
    match &node {
        EffectNode::Once(children) => {
            assert_eq!(children.len(), 1);
        }
        other => panic!("expected Once(_), got {other:?}"),
    }
}

#[test]
fn effect_node_once_empty_is_valid() {
    let node = EffectNode::Once(vec![]);
    match &node {
        EffectNode::Once(children) => {
            assert!(children.is_empty());
        }
        other => panic!("expected Once([]), got {other:?}"),
    }
}

#[test]
fn effect_node_nests_when_inside_when_two_deep() {
    let node = EffectNode::When {
        trigger: Trigger::PerfectBump,
        then: vec![EffectNode::When {
            trigger: Trigger::Impact(ImpactTarget::Cell),
            then: vec![EffectNode::Do(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
        }],
    };
    match &node {
        EffectNode::When {
            trigger: Trigger::PerfectBump,
            then,
        } => match &then[0] {
            EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                then: inner,
            } => {
                assert!(matches!(inner[0], EffectNode::Do(Effect::Shockwave { .. })));
            }
            other => panic!("expected inner When(Impact(Cell), _), got {other:?}"),
        },
        other => panic!("expected outer When(PerfectBump, _), got {other:?}"),
    }
}

#[test]
fn effect_node_nests_three_deep() {
    let node = EffectNode::When {
        trigger: Trigger::PerfectBump,
        then: vec![EffectNode::When {
            trigger: Trigger::Impact(ImpactTarget::Cell),
            then: vec![EffectNode::When {
                trigger: Trigger::CellDestroyed,
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            }],
        }],
    };
    assert!(matches!(
        node,
        EffectNode::When {
            trigger: Trigger::PerfectBump,
            ..
        }
    ));
}

#[test]
fn effect_node_when_containing_until_with_do_leaves() {
    let node = EffectNode::When {
        trigger: Trigger::BumpWhiff,
        then: vec![EffectNode::When {
            trigger: Trigger::Impact(ImpactTarget::Cell),
            then: vec![EffectNode::Until {
                until: Trigger::Impact(ImpactTarget::Breaker),
                then: vec![
                    EffectNode::Do(Effect::DamageBoost(1.5)),
                    EffectNode::Do(Effect::Shockwave {
                        base_range: 64.0,
                        range_per_level: 0.0,
                        stacks: 1,
                        speed: 400.0,
                    }),
                ],
            }],
        }],
    };
    // Verify the nested structure has 2 Do leaves inside Until
    match &node {
        EffectNode::When { then, .. } => match &then[0] {
            EffectNode::When { then: inner, .. } => match &inner[0] {
                EffectNode::Until {
                    then: until_kids, ..
                } => {
                    assert_eq!(until_kids.len(), 2, "Until node should contain 2 Do leaves");
                }
                other => panic!("expected Until, got {other:?}"),
            },
            other => panic!("expected inner When, got {other:?}"),
        },
        other => panic!("expected outer When, got {other:?}"),
    }
}

// =========================================================================
// C7 Wave 1 Part A: EffectNode RON deserialization (behaviors 7-10)
// =========================================================================

#[test]
fn effect_node_ron_when_with_do_leaf() {
    let ron_str = "When(trigger: OnPerfectBump, then: [Do(Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0))])";
    let node: EffectNode = ron::de::from_str(ron_str).expect("EffectNode When+Do RON should parse");
    assert_eq!(
        node,
        EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })]
        }
    );
}

#[test]
fn effect_node_ron_bare_do_lose_life() {
    let ron_str = "Do(LoseLife)";
    let node: EffectNode = ron::de::from_str(ron_str).expect("bare Do(LoseLife) should parse");
    assert_eq!(node, EffectNode::Do(Effect::LoseLife));
}

#[test]
fn effect_node_ron_until_node() {
    let ron_str = "Until(until: TimeExpires(3.0), then: [Do(DamageBoost(2.0))])";
    let node: EffectNode = ron::de::from_str(ron_str).expect("Until node RON should parse");
    assert_eq!(
        node,
        EffectNode::Until {
            until: Trigger::TimeExpires(3.0),
            then: vec![EffectNode::Do(Effect::DamageBoost(2.0))]
        }
    );
}

#[test]
fn effect_node_ron_until_empty_then() {
    let ron_str = "Until(until: OnImpact(Breaker), then: [])";
    let node: EffectNode = ron::de::from_str(ron_str).expect("Until with empty then should parse");
    assert_eq!(
        node,
        EffectNode::Until {
            until: Trigger::Impact(ImpactTarget::Breaker),
            then: vec![]
        }
    );
}

#[test]
fn effect_node_ron_once_node() {
    let ron_str = "Once([Do(SecondWind(invuln_secs: 3.0))])";
    let node: EffectNode = ron::de::from_str(ron_str).expect("Once node RON should parse");
    assert_eq!(
        node,
        EffectNode::Once(vec![EffectNode::Do(Effect::SecondWind {
            invuln_secs: 3.0
        })])
    );
}

#[test]
fn effect_node_ron_once_empty() {
    let ron_str = "Once([])";
    let node: EffectNode = ron::de::from_str(ron_str).expect("Once([]) should parse");
    assert_eq!(node, EffectNode::Once(vec![]));
}

#[test]
fn effect_node_ron_nested_when_until_do_combo() {
    let ron_str = "When(trigger: OnBumpWhiff, then: [When(trigger: OnImpact(Cell), then: [Until(until: OnImpact(Breaker), then: [Do(DamageBoost(1.5)), Do(Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0))])])])";
    let node: EffectNode =
        ron::de::from_str(ron_str).expect("nested When/Until/Do RON should parse");
    // Verify outer When
    match &node {
        EffectNode::When {
            trigger: Trigger::BumpWhiff,
            then,
        } => match &then[0] {
            EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                then: inner,
            } => match &inner[0] {
                EffectNode::Until {
                    until: Trigger::Impact(ImpactTarget::Breaker),
                    then: until_kids,
                } => {
                    assert_eq!(until_kids.len(), 2, "Until should have 2 Do children");
                }
                other => panic!("expected Until, got {other:?}"),
            },
            other => panic!("expected inner When(Impact(Cell)), got {other:?}"),
        },
        other => panic!("expected outer When(BumpWhiff), got {other:?}"),
    }
}

// =========================================================================
// C7 Wave 1 Part A: trigger_leaf helper (behavior 11)
// =========================================================================

#[test]
fn effect_node_trigger_leaf_builds_when_do() {
    let node = EffectNode::trigger_leaf(Trigger::BoltLost, Effect::LoseLife);
    assert_eq!(
        node,
        EffectNode::When {
            trigger: Trigger::BoltLost,
            then: vec![EffectNode::Do(Effect::LoseLife)]
        }
    );
}

#[test]
fn effect_node_trigger_leaf_on_perfect_bump() {
    let node = EffectNode::trigger_leaf(Trigger::PerfectBump, Effect::Piercing(1));
    assert_eq!(
        node,
        EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(Effect::Piercing(1))]
        }
    );
}

// =========================================================================
// EffectNode::On — construction and serde (Part A)
// =========================================================================

#[test]
fn effect_node_on_wraps_target_and_children() {
    let node = EffectNode::On {
        target: Target::Bolt,
        then: vec![EffectNode::Do(Effect::LoseLife)],
    };
    match &node {
        EffectNode::On { target, then } => {
            assert_eq!(*target, Target::Bolt);
            assert_eq!(then.len(), 1);
            assert!(matches!(then[0], EffectNode::Do(Effect::LoseLife)));
        }
        other => panic!("expected On(Bolt, _), got {other:?}"),
    }
}

#[test]
fn effect_node_on_deserializes_from_ron() {
    let ron_str = "On(target: Bolt, then: [Do(LoseLife)])";
    let node: EffectNode = ron::de::from_str(ron_str).expect("EffectNode On RON should parse");
    assert_eq!(
        node,
        EffectNode::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(Effect::LoseLife)],
        }
    );
}

#[test]
fn effect_node_on_converts_from_root_effect() {
    let root = RootEffect::On {
        target: Target::Breaker,
        then: vec![EffectNode::When {
            trigger: Trigger::BoltLost,
            then: vec![EffectNode::Do(Effect::LoseLife)],
        }],
    };
    let node: EffectNode = root.into();
    assert_eq!(
        node,
        EffectNode::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::BoltLost,
                then: vec![EffectNode::Do(Effect::LoseLife)],
            }],
        }
    );
}
