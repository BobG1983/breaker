use super::types::*;
use crate::effect::{EffectKind, EffectNode, ImpactTarget, RootEffect, Target, Trigger};

// =========================================================================
// ChipDefinition with Vec<RootEffect>
// =========================================================================

#[test]
fn chip_definition_effects_is_vec_root_effect() {
    let def = ChipDefinition {
        name: "Test".to_owned(),
        description: String::new(),
        rarity: Rarity::Common,
        max_stacks: 3,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::Piercing(1))],
        }],
        ingredients: None,
        template_name: None,
    };
    assert!(matches!(
        def.effects[0],
        RootEffect::On {
            target: Target::Bolt,
            ..
        }
    ));
}

#[test]
fn chip_definition_empty_effects_is_valid() {
    let def = ChipDefinition {
        name: "Empty".to_owned(),
        description: String::new(),
        rarity: Rarity::Common,
        max_stacks: 1,
        effects: vec![],
        ingredients: None,
        template_name: None,
    };
    assert!(def.effects.is_empty());
}

// =========================================================================
// RaritySlot with Vec<RootEffect>
// =========================================================================

#[test]
fn rarity_slot_effects_is_vec_root_effect() {
    let slot = RaritySlot {
        prefix: "Basic".to_owned(),
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 })],
        }],
    };
    assert_eq!(slot.effects.len(), 1);
    assert!(matches!(
        slot.effects[0],
        RootEffect::On {
            target: Target::Bolt,
            ..
        }
    ));
}

// =========================================================================
// ChipTemplate with Vec<RootEffect>
// =========================================================================

#[test]
fn chip_template_ron_with_root_effect() {
    let ron_str = r#"(name: "Surge", max_taken: 3, common: Some((prefix: "Basic", effects: [On(target: Bolt, then: [When(trigger: PerfectBumped, then: [Do(SpeedBoost(multiplier: 1.2))])])])), uncommon: None, rare: None, legendary: None)"#;
    let template: ChipTemplate =
        ron::de::from_str(ron_str).expect("ChipTemplate with RootEffect RON should parse");
    assert_eq!(template.name, "Surge");
    let common = template.common.unwrap();
    assert!(matches!(
        common.effects[0],
        RootEffect::On {
            target: Target::Bolt,
            ..
        }
    ));
}

#[test]
fn expand_chip_template_produces_root_effect() {
    let template = ChipTemplate {
        name: "Surge".to_owned(),
        max_taken: 3,
        common: Some(RaritySlot {
            prefix: "Basic".to_owned(),
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 })],
                }],
            }],
        }),
        uncommon: None,
        rare: None,
        legendary: None,
    };
    let defs = expand_chip_template(&template);
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].name, "Basic Surge");
    assert_eq!(defs[0].rarity, Rarity::Common);
    assert_eq!(defs[0].max_stacks, 3);
    assert!(matches!(
        defs[0].effects[0],
        RootEffect::On {
            target: Target::Bolt,
            ..
        }
    ));
}

#[test]
fn expand_chip_template_preserves_target() {
    let template = ChipTemplate {
        name: "Wide".to_owned(),
        max_taken: 3,
        common: Some(RaritySlot {
            prefix: "Basic".to_owned(),
            effects: vec![RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::Do(EffectKind::SizeBoost(20.0))],
            }],
        }),
        uncommon: None,
        rare: None,
        legendary: None,
    };
    let defs = expand_chip_template(&template);
    assert_eq!(defs.len(), 1);
    assert!(
        matches!(
            defs[0].effects[0],
            RootEffect::On {
                target: Target::Breaker,
                ..
            }
        ),
        "expand_chip_template should preserve Target::Breaker from slot"
    );
}

#[test]
fn expanded_defs_have_correct_rarities_with_root_effect() {
    let make_slot = |prefix: &str, val: u32| RaritySlot {
        prefix: prefix.to_owned(),
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::Piercing(val))],
        }],
    };
    let template = ChipTemplate {
        name: "AllSlots".to_owned(),
        max_taken: 5,
        common: Some(make_slot("C", 1)),
        uncommon: Some(make_slot("U", 2)),
        rare: Some(make_slot("R", 3)),
        legendary: Some(make_slot("L", 4)),
    };
    let defs = expand_chip_template(&template);
    assert_eq!(defs.len(), 4);
    assert_eq!(defs[0].rarity, Rarity::Common);
    assert_eq!(defs[1].rarity, Rarity::Uncommon);
    assert_eq!(defs[2].rarity, Rarity::Rare);
    assert_eq!(defs[3].rarity, Rarity::Legendary);
}

// =========================================================================
// Test constructors
// =========================================================================

#[test]
fn test_constructor_wraps_in_root_effect() {
    let def = ChipDefinition::test("P", EffectNode::Do(EffectKind::Piercing(1)), 3);
    assert_eq!(def.name, "P");
    assert_eq!(def.max_stacks, 3);
    assert_eq!(def.effects.len(), 1);
    assert!(
        matches!(
            &def.effects[0],
            RootEffect::On {
                target: Target::Bolt,
                ..
            }
        ),
        "test() should wrap effect in RootEffect::On(Bolt)"
    );
}

#[test]
fn test_simple_wraps_in_root_effect() {
    let def = ChipDefinition::test_simple("T");
    assert_eq!(def.name, "T");
    assert_eq!(def.max_stacks, 1);
    assert_eq!(def.effects.len(), 1);
    assert!(
        matches!(
            &def.effects[0],
            RootEffect::On {
                target: Target::Bolt,
                ..
            }
        ),
        "test_simple() should wrap effect in RootEffect::On(Bolt)"
    );
}

#[test]
fn test_on_uses_specified_target() {
    let def = ChipDefinition::test_on(
        "W",
        Target::Breaker,
        EffectNode::Do(EffectKind::SizeBoost(20.0)),
        3,
    );
    assert_eq!(def.name, "W");
    assert_eq!(def.max_stacks, 3);
    assert_eq!(def.effects.len(), 1);
    assert!(
        matches!(
            &def.effects[0],
            RootEffect::On {
                target: Target::Breaker,
                ..
            }
        ),
        "test_on() with Target::Breaker should create RootEffect::On(Breaker)"
    );
}

// =========================================================================
// Preserved tests: Rarity deserialization
// =========================================================================

#[test]
fn rarity_deserializes_common() {
    let r: Rarity = ron::de::from_str("Common").expect("should parse Common");
    assert_eq!(r, Rarity::Common);
}

#[test]
fn rarity_deserializes_uncommon() {
    let r: Rarity = ron::de::from_str("Uncommon").expect("should parse Uncommon");
    assert_eq!(r, Rarity::Uncommon);
}

#[test]
fn rarity_deserializes_rare() {
    let r: Rarity = ron::de::from_str("Rare").expect("should parse Rare");
    assert_eq!(r, Rarity::Rare);
}

#[test]
fn rarity_deserializes_legendary() {
    let r: Rarity = ron::de::from_str("Legendary").expect("should parse Legendary");
    assert_eq!(r, Rarity::Legendary);
}

// =========================================================================
// Preserved tests: ImpactTarget deserialization
// =========================================================================

#[test]
fn impact_target_deserializes_cell() {
    let t: ImpactTarget = ron::de::from_str("Cell").expect("should parse Cell");
    assert_eq!(t, ImpactTarget::Cell);
}

#[test]
fn impact_target_deserializes_breaker() {
    let t: ImpactTarget = ron::de::from_str("Breaker").expect("should parse Breaker");
    assert_eq!(t, ImpactTarget::Breaker);
}

#[test]
fn impact_target_deserializes_wall() {
    let t: ImpactTarget = ron::de::from_str("Wall").expect("should parse Wall");
    assert_eq!(t, ImpactTarget::Wall);
}

// =========================================================================
// Preserved tests: Target deserialization
// =========================================================================

#[test]
fn target_deserializes_bolt() {
    let t: Target = ron::de::from_str("Bolt").expect("should parse Bolt");
    assert_eq!(t, Target::Bolt);
}

#[test]
fn target_deserializes_breaker() {
    let t: Target = ron::de::from_str("Breaker").expect("should parse Breaker");
    assert_eq!(t, Target::Breaker);
}

#[test]
fn target_deserializes_all_bolts() {
    let t: Target = ron::de::from_str("AllBolts").expect("should parse AllBolts");
    assert_eq!(t, Target::AllBolts);
}

#[test]
fn target_cell_is_valid_variant() {
    let result = ron::de::from_str::<Target>("Cell");
    assert!(result.is_ok(), "Target::Cell should be a valid variant");
    assert_eq!(result.unwrap(), Target::Cell);
}

// =========================================================================
// Preserved tests: EvolutionIngredient
// =========================================================================

#[test]
fn evolution_ingredient_deserializes_from_ron() {
    let ron_str = r#"(chip_name: "Piercing Shot", stacks_required: 2)"#;
    let ingredient: EvolutionIngredient =
        ron::de::from_str(ron_str).expect("should parse EvolutionIngredient");
    assert_eq!(ingredient.chip_name, "Piercing Shot");
    assert_eq!(ingredient.stacks_required, 2);
}

// =========================================================================
// Preserved tests: expand_chip_template mechanics
// =========================================================================

#[test]
fn expand_chip_template_all_none_returns_empty() {
    let template = ChipTemplate {
        name: "Empty".to_owned(),
        max_taken: 1,
        common: None,
        uncommon: None,
        rare: None,
        legendary: None,
    };
    let defs = expand_chip_template(&template);
    assert!(defs.is_empty());
}

#[test]
fn expanded_chip_empty_prefix_uses_template_name() {
    let template = ChipTemplate {
        name: "Glass Cannon".to_owned(),
        max_taken: 1,
        common: None,
        uncommon: None,
        rare: None,
        legendary: Some(RaritySlot {
            prefix: String::new(),
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
            }],
        }),
    };
    let defs = expand_chip_template(&template);
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].name, "Glass Cannon");
    assert_eq!(defs[0].rarity, Rarity::Legendary);
}

#[test]
fn expanded_chip_whitespace_prefix_uses_template_name() {
    let template = ChipTemplate {
        name: "Glass Cannon".to_owned(),
        max_taken: 1,
        common: None,
        uncommon: None,
        rare: None,
        legendary: Some(RaritySlot {
            prefix: "   ".to_owned(),
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
            }],
        }),
    };
    let defs = expand_chip_template(&template);
    assert_eq!(defs.len(), 1);
    assert_eq!(
        defs[0].name, "Glass Cannon",
        "whitespace-only prefix should be treated as empty — name should equal template name"
    );
    assert_eq!(defs[0].rarity, Rarity::Legendary);
}
