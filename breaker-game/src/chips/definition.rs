//! Chip definition types — shared across Amps, Augments, and Overclocks.

use bevy::prelude::*;
use serde::Deserialize;

/// The category of a chip.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChipKind {
    /// Passive bolt chip.
    Amp,
    /// Passive breaker chip.
    Augment,
    /// Triggered ability.
    Overclock,
}

/// How rare a chip is — controls appearance weight in the selection pool.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

/// Effect variants for Amp chips (passive bolt upgrades).
#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
pub(crate) enum AmpEffect {
    /// Bolt passes through N cells before stopping.
    Piercing(u32),
    /// Adds fractional bonus damage per stack. Formula: damage = `BASE_BOLT_DAMAGE` * (1.0 + boost).
    DamageBoost(f32),
    /// Adds flat speed per stack.
    SpeedBoost(f32),
    /// Bolt chains to N additional cells on hit.
    ChainHit(u32),
    /// Increases bolt radius by a fraction per stack.
    SizeBoost(f32),
}

/// Effect variants for Augment chips (passive breaker upgrades).
#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
pub(crate) enum AugmentEffect {
    /// Adds flat width per stack.
    WidthBoost(f32),
    /// Adds flat speed per stack.
    SpeedBoost(f32),
    /// Adds flat bump force per stack.
    BumpForce(f32),
    /// Adds flat tilt control sensitivity per stack.
    TiltControl(f32),
}

/// Trigger chain for Overclock effects — defines when and what happens.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum TriggerChain {
    /// Area damage around impact point.
    Shockwave {
        /// Radius of the shockwave effect.
        range: f32,
    },
    /// Spawns additional bolts on trigger.
    MultiBolt {
        /// Number of extra bolts to spawn.
        count: u32,
    },
    /// Temporary shield protecting the breaker.
    Shield {
        /// How long the shield lasts in seconds.
        duration: f32,
    },
    /// Fires on a perfect bump.
    OnPerfectBump(Box<Self>),
    /// Fires on bolt impact with a cell.
    OnImpact(Box<Self>),
    /// Fires when a cell is destroyed.
    OnCellDestroyed(Box<Self>),
    /// Fires when a bolt is lost.
    OnBoltLost(Box<Self>),
}

impl TriggerChain {
    /// Returns the nesting depth of this chain.
    ///
    /// Leaf variants return 0, trigger variants return 1 + inner depth.
    #[must_use]
    pub(crate) fn depth(&self) -> u32 {
        match self {
            Self::Shockwave { .. } | Self::MultiBolt { .. } | Self::Shield { .. } => 0,
            Self::OnPerfectBump(inner)
            | Self::OnImpact(inner)
            | Self::OnCellDestroyed(inner)
            | Self::OnBoltLost(inner) => 1 + inner.depth(),
        }
    }

    /// Returns true if this is a leaf (action) variant, false if it is a trigger wrapper.
    #[must_use]
    pub(crate) const fn is_leaf(&self) -> bool {
        matches!(
            self,
            Self::Shockwave { .. } | Self::MultiBolt { .. } | Self::Shield { .. }
        )
    }
}

/// Top-level effect wrapper for any chip type.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) enum ChipEffect {
    /// Applies an Amp (bolt) effect.
    Amp(AmpEffect),
    /// Applies an Augment (breaker) effect.
    Augment(AugmentEffect),
    /// Triggered ability with a trigger chain.
    Overclock(TriggerChain),
}

/// Triggered when a chip effect should be applied.
///
/// Dispatched by `apply_chip_effect` for each selected chip.
/// Each per-effect observer self-selects via pattern matching on `effect`.
#[derive(Event, Clone, Debug)]
pub(crate) struct ChipEffectApplied {
    /// The effect to apply.
    pub effect: ChipEffect,
    /// Maximum stacks for this chip.
    pub max_stacks: u32,
}

/// A single chip definition loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub(crate) struct ChipDefinition {
    /// Display name shown on the chip card.
    pub name: String,
    /// Category discriminator.
    pub kind: ChipKind,
    /// Flavor text shown below the name.
    pub description: String,
    /// How rare this chip is.
    pub rarity: Rarity,
    /// Maximum number of times this chip can be stacked.
    pub max_stacks: u32,
    /// The effect applied when this chip is selected.
    pub effect: ChipEffect,
}

#[cfg(test)]
impl ChipDefinition {
    /// Build a test chip with full control over effect and stacking.
    pub(crate) fn test(name: &str, kind: ChipKind, effect: ChipEffect, max_stacks: u32) -> Self {
        Self {
            name: name.to_owned(),
            kind,
            description: format!("{name} description"),
            rarity: Rarity::Common,
            max_stacks,
            effect,
        }
    }

    /// Build a simple test chip with `Overclock` effect and `max_stacks` = 1.
    pub(crate) fn test_simple(name: &str, kind: ChipKind) -> Self {
        Self::test(
            name,
            kind,
            ChipEffect::Overclock(TriggerChain::Shockwave { range: 64.0 }),
            1,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_kind_deserializes_amp() {
        let ron_str = "Amp";
        let kind: ChipKind = ron::de::from_str(ron_str).expect("should parse Amp");
        assert_eq!(kind, ChipKind::Amp);
    }

    #[test]
    fn chip_kind_deserializes_augment() {
        let ron_str = "Augment";
        let kind: ChipKind = ron::de::from_str(ron_str).expect("should parse Augment");
        assert_eq!(kind, ChipKind::Augment);
    }

    #[test]
    fn chip_kind_deserializes_overclock() {
        let ron_str = "Overclock";
        let kind: ChipKind = ron::de::from_str(ron_str).expect("should parse Overclock");
        assert_eq!(kind, ChipKind::Overclock);
    }

    #[test]
    fn chip_definition_deserializes_from_ron() {
        let ron_str = r#"(name: "Piercing Shot", kind: Amp, description: "Bolt passes through", rarity: Common, max_stacks: 3, effect: Amp(Piercing(1)))"#;
        let def: ChipDefinition = ron::de::from_str(ron_str).expect("should parse ChipDefinition");
        assert_eq!(def.name, "Piercing Shot");
        assert_eq!(def.kind, ChipKind::Amp);
        assert_eq!(def.description, "Bolt passes through");
        assert_eq!(def.rarity, Rarity::Common);
        assert_eq!(def.max_stacks, 3);
        assert_eq!(def.effect, ChipEffect::Amp(AmpEffect::Piercing(1)));
    }

    // --- Part A: New type deserialization tests ---

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

    #[test]
    fn amp_effect_deserializes_piercing() {
        let e: AmpEffect = ron::de::from_str("Piercing(1)").expect("should parse Piercing(1)");
        assert_eq!(e, AmpEffect::Piercing(1));
    }

    #[test]
    fn amp_effect_deserializes_damage_boost() {
        let e: AmpEffect =
            ron::de::from_str("DamageBoost(1.5)").expect("should parse DamageBoost(1.5)");
        assert_eq!(e, AmpEffect::DamageBoost(1.5));
    }

    #[test]
    fn amp_effect_deserializes_speed_boost() {
        let e: AmpEffect =
            ron::de::from_str("SpeedBoost(50.0)").expect("should parse SpeedBoost(50.0)");
        assert_eq!(e, AmpEffect::SpeedBoost(50.0));
    }

    #[test]
    fn amp_effect_deserializes_chain_hit() {
        let e: AmpEffect = ron::de::from_str("ChainHit(2)").expect("should parse ChainHit(2)");
        assert_eq!(e, AmpEffect::ChainHit(2));
    }

    #[test]
    fn amp_effect_deserializes_size_boost() {
        let e: AmpEffect =
            ron::de::from_str("SizeBoost(0.5)").expect("should parse SizeBoost(0.5)");
        assert_eq!(e, AmpEffect::SizeBoost(0.5));
    }

    #[test]
    fn augment_effect_deserializes_width_boost() {
        let e: AugmentEffect =
            ron::de::from_str("WidthBoost(20.0)").expect("should parse WidthBoost(20.0)");
        assert_eq!(e, AugmentEffect::WidthBoost(20.0));
    }

    #[test]
    fn augment_effect_deserializes_speed_boost() {
        let e: AugmentEffect =
            ron::de::from_str("SpeedBoost(30.0)").expect("should parse SpeedBoost(30.0)");
        assert_eq!(e, AugmentEffect::SpeedBoost(30.0));
    }

    #[test]
    fn augment_effect_deserializes_bump_force() {
        let e: AugmentEffect =
            ron::de::from_str("BumpForce(10.0)").expect("should parse BumpForce(10.0)");
        assert_eq!(e, AugmentEffect::BumpForce(10.0));
    }

    #[test]
    fn augment_effect_deserializes_tilt_control() {
        let e: AugmentEffect =
            ron::de::from_str("TiltControl(5.0)").expect("should parse TiltControl(5.0)");
        assert_eq!(e, AugmentEffect::TiltControl(5.0));
    }

    #[test]
    fn chip_effect_deserializes_amp_piercing() {
        let e: ChipEffect =
            ron::de::from_str("Amp(Piercing(1))").expect("should parse Amp(Piercing(1))");
        assert_eq!(e, ChipEffect::Amp(AmpEffect::Piercing(1)));
    }

    #[test]
    fn chip_effect_deserializes_augment_width_boost() {
        let e: ChipEffect = ron::de::from_str("Augment(WidthBoost(20.0))")
            .expect("should parse Augment(WidthBoost(20.0))");
        assert_eq!(e, ChipEffect::Augment(AugmentEffect::WidthBoost(20.0)));
    }

    #[test]
    fn chip_effect_deserializes_overclock() {
        let e: ChipEffect = ron::de::from_str("Overclock(Shockwave(range: 64.0))")
            .expect("should parse Overclock(Shockwave)");
        assert_eq!(
            e,
            ChipEffect::Overclock(TriggerChain::Shockwave { range: 64.0 })
        );
    }

    #[test]
    fn chip_definition_deserializes_with_all_new_fields() {
        let ron_str = r#"(
            name: "Piercing Shot",
            kind: Amp,
            description: "Bolt passes through",
            rarity: Common,
            max_stacks: 3,
            effect: Amp(Piercing(1))
        )"#;
        let def: ChipDefinition =
            ron::de::from_str(ron_str).expect("should parse ChipDefinition with new fields");
        assert_eq!(def.name, "Piercing Shot");
        assert_eq!(def.kind, ChipKind::Amp);
        assert_eq!(def.description, "Bolt passes through");
        assert_eq!(def.rarity, Rarity::Common);
        assert_eq!(def.max_stacks, 3);
        assert_eq!(def.effect, ChipEffect::Amp(AmpEffect::Piercing(1)));
    }

    // --- Existing RON file tests (will fail until RON files are updated) ---

    #[test]
    fn amp_ron_parses() {
        let ron_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/amps/piercing.amp.ron"
        ));
        let def: ChipDefinition = ron::de::from_str(ron_str).expect("amp RON should parse");
        assert_eq!(def.kind, ChipKind::Amp);
    }

    #[test]
    fn augment_ron_parses() {
        let ron_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/augments/wide_breaker.augment.ron"
        ));
        let def: ChipDefinition = ron::de::from_str(ron_str).expect("augment RON should parse");
        assert_eq!(def.kind, ChipKind::Augment);
    }

    #[test]
    fn overclock_ron_parses() {
        let ron_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/overclocks/surge.overclock.ron"
        ));
        let def: ChipDefinition = ron::de::from_str(ron_str).expect("overclock RON should parse");
        assert_eq!(def.kind, ChipKind::Overclock);
        assert_eq!(
            def.effect,
            ChipEffect::Overclock(TriggerChain::OnPerfectBump(Box::new(
                TriggerChain::OnImpact(Box::new(TriggerChain::Shockwave { range: 64.0 }))
            )))
        );
    }

    // --- TriggerChain deserialization tests ---

    #[test]
    fn trigger_chain_deserializes_shockwave() {
        let tc: TriggerChain =
            ron::de::from_str("Shockwave(range: 64.0)").expect("should parse Shockwave");
        assert_eq!(tc, TriggerChain::Shockwave { range: 64.0 });
    }

    #[test]
    fn trigger_chain_deserializes_multi_bolt() {
        let tc: TriggerChain =
            ron::de::from_str("MultiBolt(count: 3)").expect("should parse MultiBolt");
        assert_eq!(tc, TriggerChain::MultiBolt { count: 3 });
    }

    #[test]
    fn trigger_chain_deserializes_shield() {
        let tc: TriggerChain =
            ron::de::from_str("Shield(duration: 5.0)").expect("should parse Shield");
        assert_eq!(tc, TriggerChain::Shield { duration: 5.0 });
    }

    #[test]
    fn trigger_chain_deserializes_on_perfect_bump_leaf() {
        let tc: TriggerChain = ron::de::from_str("OnPerfectBump(Shockwave(range: 64.0))")
            .expect("should parse OnPerfectBump wrapping Shockwave");
        assert_eq!(
            tc,
            TriggerChain::OnPerfectBump(Box::new(TriggerChain::Shockwave { range: 64.0 }))
        );
    }

    #[test]
    fn trigger_chain_deserializes_nested_two_deep() {
        let tc: TriggerChain = ron::de::from_str("OnPerfectBump(OnImpact(Shockwave(range: 64.0)))")
            .expect("should parse double-nested TriggerChain");
        assert_eq!(
            tc,
            TriggerChain::OnPerfectBump(Box::new(TriggerChain::OnImpact(Box::new(
                TriggerChain::Shockwave { range: 64.0 }
            ))))
        );
    }

    // --- TriggerChain depth tests ---

    #[test]
    fn trigger_chain_depth_leaf_is_zero() {
        assert_eq!(TriggerChain::Shockwave { range: 64.0 }.depth(), 0);
        assert_eq!(TriggerChain::MultiBolt { count: 3 }.depth(), 0);
        assert_eq!(TriggerChain::Shield { duration: 5.0 }.depth(), 0);
    }

    #[test]
    fn trigger_chain_depth_single_trigger_is_one() {
        let tc = TriggerChain::OnPerfectBump(Box::new(TriggerChain::Shockwave { range: 64.0 }));
        assert_eq!(tc.depth(), 1);
    }

    #[test]
    fn trigger_chain_depth_nested_is_two() {
        let tc = TriggerChain::OnPerfectBump(Box::new(TriggerChain::OnImpact(Box::new(
            TriggerChain::Shockwave { range: 64.0 },
        ))));
        assert_eq!(tc.depth(), 2);
    }

    // --- TriggerChain is_leaf tests ---

    #[test]
    fn trigger_chain_is_leaf_true_for_leaves() {
        assert!(TriggerChain::Shockwave { range: 64.0 }.is_leaf());
        assert!(TriggerChain::MultiBolt { count: 3 }.is_leaf());
        assert!(TriggerChain::Shield { duration: 5.0 }.is_leaf());
    }

    #[test]
    fn trigger_chain_is_leaf_false_for_triggers() {
        let leaf = TriggerChain::Shockwave { range: 64.0 };
        assert!(!TriggerChain::OnPerfectBump(Box::new(leaf.clone())).is_leaf());
        assert!(!TriggerChain::OnImpact(Box::new(leaf.clone())).is_leaf());
        assert!(!TriggerChain::OnCellDestroyed(Box::new(leaf.clone())).is_leaf());
        assert!(!TriggerChain::OnBoltLost(Box::new(leaf)).is_leaf());
    }

    // --- ChipEffect with TriggerChain tests ---

    #[test]
    fn chip_effect_overclock_with_trigger_chain_deserializes() {
        let e: ChipEffect = ron::de::from_str("Overclock(Shockwave(range: 64.0))")
            .expect("should parse Overclock with TriggerChain");
        assert_eq!(
            e,
            ChipEffect::Overclock(TriggerChain::Shockwave { range: 64.0 })
        );
    }

    #[test]
    fn full_surge_chain_ron_parses() {
        let e: ChipEffect =
            ron::de::from_str("Overclock(OnPerfectBump(OnImpact(Shockwave(range: 64.0))))")
                .expect("should parse full surge chain as ChipEffect");
        assert_eq!(
            e,
            ChipEffect::Overclock(TriggerChain::OnPerfectBump(Box::new(
                TriggerChain::OnImpact(Box::new(TriggerChain::Shockwave { range: 64.0 }))
            )))
        );
    }
}
