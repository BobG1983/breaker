use ordered_float::OrderedFloat;

use crate::{
    chips::definition::types::*,
    effect_v3::{
        effects::{DamageBoostConfig, PiercingConfig, SizeBoostConfig, SpeedBoostConfig},
        types::{EffectType, RootNode, StampTarget, Tree, Trigger},
    },
};

// =========================================================================
// ChipTemplate with Vec<RootNode>
// =========================================================================

#[test]
fn chip_template_ron_with_root_node() {
    let ron_str = r#"(name: "Surge", max_taken: 3, common: Some((prefix: "Basic", effects: [Stamp(Bolt, When(PerfectBumped, Fire(SpeedBoost((multiplier: 1.2)))))])), uncommon: None, rare: None, legendary: None)"#;
    let template: ChipTemplate =
        ron::de::from_str(ron_str).expect("ChipTemplate with RootNode RON should parse");
    assert_eq!(template.name, "Surge");
    let common = template.common.unwrap();
    assert!(matches!(
        common.effects[0],
        RootNode::Stamp(StampTarget::Bolt, _)
    ));
}

#[test]
fn expand_chip_template_produces_root_node() {
    let template = ChipTemplate {
        name: "Surge".to_owned(),
        max_taken: 3,
        common: Some(RaritySlot {
            prefix: "Basic".to_owned(),
            effects: vec![RootNode::Stamp(
                StampTarget::Bolt,
                Tree::When(
                    Trigger::PerfectBumped,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.2),
                    }))),
                ),
            )],
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
        RootNode::Stamp(StampTarget::Bolt, _)
    ));
}

#[test]
fn expand_chip_template_preserves_target() {
    let template = ChipTemplate {
        name: "Wide".to_owned(),
        max_taken: 3,
        common: Some(RaritySlot {
            prefix: "Basic".to_owned(),
            effects: vec![RootNode::Stamp(
                StampTarget::Breaker,
                Tree::Fire(EffectType::SizeBoost(SizeBoostConfig {
                    multiplier: OrderedFloat(20.0),
                })),
            )],
        }),
        uncommon: None,
        rare: None,
        legendary: None,
    };
    let defs = expand_chip_template(&template);
    assert_eq!(defs.len(), 1);
    assert!(
        matches!(defs[0].effects[0], RootNode::Stamp(StampTarget::Breaker, _)),
        "expand_chip_template should preserve StampTarget::Breaker from slot"
    );
}

#[test]
fn expanded_defs_have_correct_rarities_with_root_node() {
    let make_slot = |prefix: &str, val: u32| RaritySlot {
        prefix: prefix.to_owned(),
        effects: vec![RootNode::Stamp(
            StampTarget::Bolt,
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: val })),
        )],
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
            effects: vec![RootNode::Stamp(
                StampTarget::Bolt,
                Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                    multiplier: OrderedFloat(1.0),
                })),
            )],
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
            effects: vec![RootNode::Stamp(
                StampTarget::Bolt,
                Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                    multiplier: OrderedFloat(1.0),
                })),
            )],
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

// =========================================================================
// C6: expand_chip_template sets template_name on all variants
// =========================================================================

#[test]
fn expand_chip_template_sets_template_name_on_all_variants() {
    let make_slot = |prefix: &str| RaritySlot {
        prefix: prefix.to_owned(),
        effects: vec![RootNode::Stamp(
            StampTarget::Bolt,
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
        )],
    };
    let template = ChipTemplate {
        name: "Surge".to_owned(),
        max_taken: 3,
        common: Some(make_slot("Basic")),
        uncommon: Some(make_slot("Keen")),
        rare: Some(make_slot("Honed")),
        legendary: Some(make_slot("Mythic")),
    };
    let defs = expand_chip_template(&template);
    assert_eq!(defs.len(), 4);

    for (i, def) in defs.iter().enumerate() {
        assert_eq!(
            def.template_name,
            Some("Surge".to_owned()),
            "defs[{i}] ({}) should have template_name == Some(\"Surge\"), got {:?}",
            def.name,
            def.template_name
        );
    }

    // Edge case: single slot template still gets template_name
    let single_template = ChipTemplate {
        name: "Surge".to_owned(),
        max_taken: 3,
        common: Some(make_slot("Basic")),
        uncommon: None,
        rare: None,
        legendary: None,
    };
    let single_defs = expand_chip_template(&single_template);
    assert_eq!(single_defs.len(), 1);
    assert_eq!(
        single_defs[0].template_name,
        Some("Surge".to_owned()),
        "single-slot template should still set template_name"
    );
}
