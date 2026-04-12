use ordered_float::OrderedFloat;

use crate::{
    chips::definition::types::*,
    effect_v3::{
        effects::{PiercingConfig, SizeBoostConfig, SpeedBoostConfig},
        types::{EffectType, RootNode, StampTarget, Tree},
    },
};

// =========================================================================
// ChipDefinition with Vec<RootNode>
// =========================================================================

#[test]
fn chip_definition_effects_is_vec_root_node() {
    let def = ChipDefinition {
        name:          "Test".to_owned(),
        description:   String::new(),
        rarity:        Rarity::Common,
        max_stacks:    3,
        effects:       vec![RootNode::Stamp(
            StampTarget::Bolt,
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
        )],
        ingredients:   None,
        template_name: None,
    };
    assert!(matches!(
        def.effects[0],
        RootNode::Stamp(StampTarget::Bolt, _)
    ));
}

#[test]
fn chip_definition_empty_effects_is_valid() {
    let def = ChipDefinition {
        name:          "Empty".to_owned(),
        description:   String::new(),
        rarity:        Rarity::Common,
        max_stacks:    1,
        effects:       vec![],
        ingredients:   None,
        template_name: None,
    };
    assert!(def.effects.is_empty());
}

// =========================================================================
// RaritySlot with Vec<RootNode>
// =========================================================================

#[test]
fn rarity_slot_effects_is_vec_root_node() {
    let slot = RaritySlot {
        prefix:  "Basic".to_owned(),
        effects: vec![RootNode::Stamp(
            StampTarget::Bolt,
            Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.2),
            })),
        )],
    };
    assert_eq!(slot.effects.len(), 1);
    assert!(matches!(
        slot.effects[0],
        RootNode::Stamp(StampTarget::Bolt, _)
    ));
}

// =========================================================================
// Test constructors
// =========================================================================

#[test]
fn test_constructor_wraps_in_root_node() {
    let def = ChipDefinition::test(
        "P",
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
        3,
    );
    assert_eq!(def.name, "P");
    assert_eq!(def.max_stacks, 3);
    assert_eq!(def.effects.len(), 1);
    assert!(
        matches!(&def.effects[0], RootNode::Stamp(StampTarget::Bolt, _)),
        "test() should wrap effect in RootNode::Stamp(Bolt, _)"
    );
}

#[test]
fn test_simple_wraps_in_root_node() {
    let def = ChipDefinition::test_simple("T");
    assert_eq!(def.name, "T");
    assert_eq!(def.max_stacks, 1);
    assert_eq!(def.effects.len(), 1);
    assert!(
        matches!(&def.effects[0], RootNode::Stamp(StampTarget::Bolt, _)),
        "test_simple() should wrap effect in RootNode::Stamp(Bolt, _)"
    );
}

#[test]
fn test_on_uses_specified_target() {
    let def = ChipDefinition::test_on(
        "W",
        StampTarget::Breaker,
        Tree::Fire(EffectType::SizeBoost(SizeBoostConfig {
            multiplier: OrderedFloat(20.0),
        })),
        3,
    );
    assert_eq!(def.name, "W");
    assert_eq!(def.max_stacks, 3);
    assert_eq!(def.effects.len(), 1);
    assert!(
        matches!(&def.effects[0], RootNode::Stamp(StampTarget::Breaker, _)),
        "test_on() with StampTarget::Breaker should create RootNode::Stamp(Breaker, _)"
    );
}
