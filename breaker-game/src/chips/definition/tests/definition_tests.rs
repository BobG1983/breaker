use super::super::types::*;
use crate::effect::{EffectKind, EffectNode, RootEffect, Target};

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
