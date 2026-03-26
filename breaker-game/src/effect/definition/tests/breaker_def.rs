use super::super::*;

// =========================================================================
// C7 Wave 1 Part I: BreakerDefinition migration (behaviors 44-46)
// =========================================================================

#[test]
fn breaker_definition_fields_use_effect_node() {
    let def = BreakerDefinition {
        name: "Aegis".to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: Some(3),
        effects: vec![
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(Effect::LoseLife)],
                }],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.5 })],
                }],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::EarlyBumped,
                    then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.1 })],
                }],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::LateBumped,
                    then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.1 })],
                }],
            },
        ],
    };
    // Verify BoltLost effect is present
    assert!(matches!(
        &def.effects[0],
        RootEffect::On { target: Target::Breaker, then } if matches!(&then[0], EffectNode::When { trigger: Trigger::BoltLost, then: inner } if matches!(&inner[0], EffectNode::Do(Effect::LoseLife)))
    ));
    // Verify PerfectBumped SpeedBoost is present
    assert!(matches!(
        &def.effects[1],
        RootEffect::On { target: Target::Bolt, then } if matches!(&then[0], EffectNode::When { trigger: Trigger::PerfectBumped, then: inner } if matches!(&inner[0], EffectNode::Do(Effect::SpeedBoost { .. })))
    ));
}

#[test]
fn breaker_definition_none_early_late_bump_is_valid() {
    let def = BreakerDefinition {
        name: "Prism".to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(Effect::SpawnBolts {
                    count: 1,
                    lifespan: None,
                    inherit: false,
                })],
            }],
        }],
    };
    // Only one effect — no early/late bump entries
    assert_eq!(def.effects.len(), 1);
}

#[test]
fn breaker_definition_chains_holds_nested_when_tree() {
    let def = BreakerDefinition {
        name: "Test".to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::Do(Effect::Shockwave {
                        base_range: 64.0,
                        range_per_level: 0.0,
                        stacks: 1,
                        speed: 400.0,
                    })],
                }],
            }],
        }],
    };
    assert_eq!(def.effects.len(), 1);
}

#[test]
fn breaker_definition_empty_chains_is_valid() {
    let def = BreakerDefinition {
        name: "Test".to_owned(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: None,
        effects: vec![],
    };
    assert!(def.effects.is_empty());
}

#[test]
fn breaker_definition_ron_with_effect_node_syntax() {
    let ron_str = r#"
    (
        name: "Aegis",
        stat_overrides: (),
        life_pool: Some(3),
        effects: [
            On(target: Breaker, then: [When(trigger: OnBoltLost, then: [Do(LoseLife)])]),
            On(target: Bolt, then: [When(trigger: PerfectBumped, then: [Do(SpeedBoost(multiplier: 1.5))])]),
            On(target: Bolt, then: [When(trigger: EarlyBumped, then: [Do(SpeedBoost(multiplier: 1.1))])]),
            On(target: Bolt, then: [When(trigger: LateBumped, then: [Do(SpeedBoost(multiplier: 1.1))])]),
        ],
    )
    "#;
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("BreakerDefinition with EffectNode RON should parse");
    assert_eq!(def.name, "Aegis");
    assert_eq!(def.effects.len(), 4);
}

#[test]
fn breaker_definition_ron_prism_style_none_early_late() {
    let ron_str = r#"
    (
        name: "Prism",
        stat_overrides: (),
        life_pool: None,
        effects: [
            On(target: Breaker, then: [When(trigger: OnBoltLost, then: [Do(TimePenalty(seconds: 7.0))])]),
            On(target: Breaker, then: [When(trigger: OnPerfectBump, then: [Do(SpawnBolts())])]),
        ],
    )
    "#;
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("Prism-style BreakerDefinition should parse");
    assert_eq!(def.name, "Prism");
    assert_eq!(def.effects.len(), 2);
}
