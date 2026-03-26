//! Chip definition types — content types for chip definitions and templates.

use bevy::prelude::*;
use serde::Deserialize;

/// How rare a chip is — controls appearance weight in the selection pool.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rarity {
    /// Frequently appearing chips.
    Common,
    /// Moderately rare chips.
    Uncommon,
    /// Hard to find chips.
    Rare,
    /// Extremely rare, run-defining chips.
    Legendary,
    /// Evolution-tier chips — produced by combining maxed ingredient chips.
    Evolution,
}

/// A single ingredient required for a chip evolution recipe.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct EvolutionIngredient {
    /// Name of the chip required.
    pub chip_name: String,
    /// Minimum stacks the player must hold.
    pub stacks_required: u32,
}

/// A rarity slot within a [`ChipTemplate`], defining the prefix and effects
/// for one rarity tier of a template chip.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct RaritySlot {
    /// Display prefix prepended to the template name (e.g., "Basic", "Keen").
    pub prefix: String,
    /// The effects applied when this rarity variant is selected.
    pub effects: Vec<crate::effect::definition::RootEffect>,
}

/// A chip template loaded from RON (`.chip.ron`).
///
/// Each template defines up to four rarity variants. At load time,
/// [`expand_template`] converts each non-`None` slot into a [`ChipDefinition`].
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct ChipTemplate {
    /// Base name shared by all rarity variants.
    pub name: String,
    /// Maximum total chips from this template the player may hold.
    pub max_taken: u32,
    /// Common-rarity variant, if any.
    pub common: Option<RaritySlot>,
    /// Uncommon-rarity variant, if any.
    pub uncommon: Option<RaritySlot>,
    /// Rare-rarity variant, if any.
    pub rare: Option<RaritySlot>,
    /// Legendary-rarity variant, if any.
    pub legendary: Option<RaritySlot>,
}

/// Expand a [`ChipTemplate`] into one [`ChipDefinition`] per non-`None` rarity slot.
///
/// Each slot's prefix is prepended to the template name. An empty or
/// whitespace-only prefix causes the expanded name to equal the template name
/// (no prefix prepended).
#[must_use]
pub(crate) fn expand_template(template: &ChipTemplate) -> Vec<ChipDefinition> {
    let slots: [(Rarity, &Option<RaritySlot>); 4] = [
        (Rarity::Common, &template.common),
        (Rarity::Uncommon, &template.uncommon),
        (Rarity::Rare, &template.rare),
        (Rarity::Legendary, &template.legendary),
    ];

    slots
        .iter()
        .filter_map(|(rarity, slot_opt)| {
            let slot = slot_opt.as_ref()?;
            let name = if slot.prefix.trim().is_empty() {
                template.name.clone()
            } else {
                format!("{} {}", slot.prefix, template.name)
            };
            Some(ChipDefinition {
                name,
                description: String::new(),
                rarity: *rarity,
                max_stacks: template.max_taken,
                effects: slot.effects.clone(),
                ingredients: None,
                template_name: Some(template.name.clone()),
            })
        })
        .collect()
}

/// A single chip definition loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct ChipDefinition {
    /// Display name shown on the chip card.
    pub name: String,
    /// Flavor text shown below the name.
    pub description: String,
    /// How rare this chip is.
    pub rarity: Rarity,
    /// Maximum number of times this chip can be stacked.
    pub max_stacks: u32,
    /// The effects applied when this chip is selected.
    pub effects: Vec<crate::effect::definition::RootEffect>,
    /// Evolution ingredients. `None` for non-evolution chips.
    #[serde(default)]
    pub ingredients: Option<Vec<EvolutionIngredient>>,
    /// Template this chip was expanded from, if any.
    #[serde(default)]
    pub template_name: Option<String>,
}

#[cfg(test)]
impl ChipDefinition {
    /// Build a test chip wrapping the effect in `RootEffect::On` targeting `Bolt`.
    pub(crate) fn test(
        name: &str,
        effect: crate::effect::definition::EffectNode,
        max_stacks: u32,
    ) -> Self {
        Self::test_on(
            name,
            crate::effect::definition::Target::Bolt,
            effect,
            max_stacks,
        )
    }

    /// Build a simple test chip with a triggered chain and `max_stacks` = 1.
    pub(crate) fn test_simple(name: &str) -> Self {
        use crate::effect::definition::{Effect, EffectNode, Trigger};
        Self::test(
            name,
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            },
            1,
        )
    }

    /// Build a test chip with explicit target control.
    pub(crate) fn test_on(
        name: &str,
        target: crate::effect::definition::Target,
        effect: crate::effect::definition::EffectNode,
        max_stacks: u32,
    ) -> Self {
        use crate::effect::definition::RootEffect;
        Self {
            name: name.to_owned(),
            description: format!("{name} description"),
            rarity: Rarity::Common,
            max_stacks,
            effects: vec![RootEffect::On {
                target,
                then: vec![effect],
            }],
            ingredients: None,
            template_name: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::definition::{
        Effect, EffectNode, ImpactTarget, RootEffect, Target, Trigger,
    };

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
                then: vec![EffectNode::Do(Effect::Piercing(1))],
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

    #[test]
    fn chip_definition_ron_with_root_effect_syntax() {
        let ron_str = r#"(name: "Test", description: "test", rarity: Common, max_stacks: 3, effects: [On(target: Bolt, then: [Do(Piercing(1))])])"#;
        let def: ChipDefinition =
            ron::de::from_str(ron_str).expect("ChipDefinition with RootEffect RON should parse");
        assert_eq!(def.name, "Test");
        assert!(matches!(
            def.effects[0],
            RootEffect::On {
                target: Target::Bolt,
                ..
            }
        ));
    }

    #[test]
    fn chip_definition_ron_triggered_chain() {
        let ron_str = r#"(name: "Test", description: "test", rarity: Rare, max_stacks: 1, effects: [On(target: Bolt, then: [When(trigger: OnPerfectBump, then: [Do(Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0))])])])"#;
        let def: ChipDefinition = ron::de::from_str(ron_str)
            .expect("ChipDefinition with triggered RootEffect RON should parse");
        assert!(matches!(
            def.effects[0],
            RootEffect::On {
                target: Target::Bolt,
                ..
            }
        ));
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
                then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.2 })],
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
    fn expand_template_produces_root_effect() {
        let template = ChipTemplate {
            name: "Surge".to_owned(),
            max_taken: 3,
            common: Some(RaritySlot {
                prefix: "Basic".to_owned(),
                effects: vec![RootEffect::On {
                    target: Target::Bolt,
                    then: vec![EffectNode::When {
                        trigger: Trigger::PerfectBump,
                        then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.2 })],
                    }],
                }],
            }),
            uncommon: None,
            rare: None,
            legendary: None,
        };
        let defs = expand_template(&template);
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
    fn expand_template_preserves_target() {
        let template = ChipTemplate {
            name: "Wide".to_owned(),
            max_taken: 3,
            common: Some(RaritySlot {
                prefix: "Basic".to_owned(),
                effects: vec![RootEffect::On {
                    target: Target::Breaker,
                    then: vec![EffectNode::Do(Effect::SizeBoost(20.0))],
                }],
            }),
            uncommon: None,
            rare: None,
            legendary: None,
        };
        let defs = expand_template(&template);
        assert_eq!(defs.len(), 1);
        assert!(
            matches!(
                defs[0].effects[0],
                RootEffect::On {
                    target: Target::Breaker,
                    ..
                }
            ),
            "expand_template should preserve Target::Breaker from slot"
        );
    }

    #[test]
    fn expanded_defs_have_correct_rarities_with_root_effect() {
        let make_slot = |prefix: &str, val: u32| RaritySlot {
            prefix: prefix.to_owned(),
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::Do(Effect::Piercing(val))],
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
        let defs = expand_template(&template);
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
        let def = ChipDefinition::test("P", EffectNode::Do(Effect::Piercing(1)), 3);
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
            EffectNode::Do(Effect::SizeBoost(20.0)),
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
    // Preserved tests: expand_template mechanics
    // =========================================================================

    #[test]
    fn expand_template_all_none_returns_empty() {
        let template = ChipTemplate {
            name: "Empty".to_owned(),
            max_taken: 1,
            common: None,
            uncommon: None,
            rare: None,
            legendary: None,
        };
        let defs = expand_template(&template);
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
                    then: vec![EffectNode::Do(Effect::DamageBoost(1.0))],
                }],
            }),
        };
        let defs = expand_template(&template);
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
                    then: vec![EffectNode::Do(Effect::DamageBoost(1.0))],
                }],
            }),
        };
        let defs = expand_template(&template);
        assert_eq!(defs.len(), 1);
        assert_eq!(
            defs[0].name, "Glass Cannon",
            "whitespace-only prefix should be treated as empty — name should equal template name"
        );
        assert_eq!(defs[0].rarity, Rarity::Legendary);
    }
}
