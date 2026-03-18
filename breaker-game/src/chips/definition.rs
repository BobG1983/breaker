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

/// A single chip definition loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub(crate) struct ChipDefinition {
    /// Display name shown on the chip card.
    pub name: String,
    /// Category discriminator.
    pub kind: ChipKind,
    /// Flavor text shown below the name.
    pub description: String,
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
        let ron_str = r#"(name: "Piercing Shot", kind: Amp, description: "Bolt passes through")"#;
        let def: ChipDefinition = ron::de::from_str(ron_str).expect("should parse ChipDefinition");
        assert_eq!(def.name, "Piercing Shot");
        assert_eq!(def.kind, ChipKind::Amp);
        assert_eq!(def.description, "Bolt passes through");
    }

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
