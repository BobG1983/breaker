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
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
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
    /// Multiplies damage dealt per stack.
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

/// Top-level effect wrapper for any chip type.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
pub(crate) enum ChipEffect {
    /// Applies an Amp (bolt) effect.
    Amp(AmpEffect),
    /// Applies an Augment (breaker) effect.
    Augment(AugmentEffect),
    /// Triggered ability — deferred to phase 4d.
    Overclock,
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
    pub fn test(name: &str, kind: ChipKind, effect: ChipEffect, max_stacks: u32) -> Self {
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
    pub fn test_simple(name: &str, kind: ChipKind) -> Self {
        Self::test(name, kind, ChipEffect::Overclock, 1)
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
        let e: ChipEffect = ron::de::from_str("Overclock").expect("should parse Overclock");
        assert_eq!(e, ChipEffect::Overclock);
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
    }
}
