//! Tier-based difficulty curve — RON-deserialized content data types.

use bevy::prelude::*;
use serde::Deserialize;

/// The type of a node in the run sequence.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeType {
    /// A passive node — no active timer pressure.
    Passive,
    /// An active node — timer ticks down during play.
    Active,
    /// A boss node — harder encounter at the end of a tier.
    Boss,
}

/// How many nodes a tier contains — fixed count or a range.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TierNodeCount {
    /// Exactly this many nodes.
    Fixed(u32),
    /// Between min and max nodes (inclusive).
    Range(u32, u32),
}

impl TierNodeCount {
    /// Validates that the count specification is well-formed.
    ///
    /// # Errors
    ///
    /// Returns an error string if the range has min greater than max.
    pub fn validate(&self) -> Result<(), String> {
        if let Self::Range(min, max) = self
            && min > max
        {
            return Err(format!(
                "TierNodeCount::Range min ({min}) must not be greater than max ({max})"
            ));
        }
        Ok(())
    }
}

/// Definition of a single difficulty tier loaded from RON.
#[derive(Deserialize, Clone, Debug)]
pub struct TierDefinition {
    /// How many nodes this tier contains.
    pub nodes: TierNodeCount,
    /// Fraction of nodes in this tier that are active (0.0 to 1.0).
    pub active_ratio: f32,
    /// Hit-point multiplier applied to cells in this tier.
    pub hp_mult: f32,
    /// Timer multiplier applied to node timers in this tier.
    pub timer_mult: f32,
    /// Cell-type aliases introduced in this tier.
    pub introduced_cells: Vec<char>,
}

/// Difficulty curve defaults loaded from `difficulty.ron`.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct DifficultyCurveDefaults {
    /// Ordered list of tier definitions.
    pub tiers: Vec<TierDefinition>,
    /// HP multiplier applied to boss nodes.
    pub boss_hp_mult: f32,
    /// Timer reduction applied after each boss encounter.
    pub timer_reduction_per_boss: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- NodeType deserialization --

    #[test]
    fn node_type_passive_deserializes_from_ron() {
        let result: NodeType = ron::de::from_str("Passive").expect("Passive should deserialize");
        assert_eq!(result, NodeType::Passive);
    }

    #[test]
    fn node_type_active_deserializes_from_ron() {
        let result: NodeType = ron::de::from_str("Active").expect("Active should deserialize");
        assert_eq!(result, NodeType::Active);
    }

    #[test]
    fn node_type_boss_deserializes_from_ron() {
        let result: NodeType = ron::de::from_str("Boss").expect("Boss should deserialize");
        assert_eq!(result, NodeType::Boss);
    }

    // -- TierNodeCount deserialization --

    #[test]
    fn tier_node_count_fixed_deserializes_from_ron() {
        let result: TierNodeCount =
            ron::de::from_str("Fixed(5)").expect("Fixed(5) should deserialize");
        assert_eq!(result, TierNodeCount::Fixed(5));
    }

    #[test]
    fn tier_node_count_fixed_zero_deserializes_from_ron() {
        let result: TierNodeCount =
            ron::de::from_str("Fixed(0)").expect("Fixed(0) should deserialize");
        assert_eq!(result, TierNodeCount::Fixed(0));
    }

    #[test]
    fn tier_node_count_range_deserializes_from_ron() {
        let result: TierNodeCount =
            ron::de::from_str("Range(4, 6)").expect("Range(4, 6) should deserialize");
        assert_eq!(result, TierNodeCount::Range(4, 6));
    }

    #[test]
    fn tier_node_count_range_min_equals_max_deserializes_from_ron() {
        let result: TierNodeCount =
            ron::de::from_str("Range(5, 5)").expect("Range(5, 5) should deserialize");
        assert_eq!(result, TierNodeCount::Range(5, 5));
    }

    // -- TierNodeCount::validate --

    #[test]
    fn validate_accepts_fixed_count() {
        assert!(TierNodeCount::Fixed(5).validate().is_ok());
    }

    #[test]
    fn validate_accepts_fixed_zero() {
        assert!(TierNodeCount::Fixed(0).validate().is_ok());
    }

    #[test]
    fn validate_accepts_valid_range() {
        assert!(TierNodeCount::Range(4, 6).validate().is_ok());
    }

    #[test]
    fn validate_accepts_range_min_equals_max() {
        assert!(TierNodeCount::Range(5, 5).validate().is_ok());
    }

    #[test]
    fn validate_rejects_range_min_greater_than_max() {
        let result = TierNodeCount::Range(6, 4).validate();
        assert!(result.is_err(), "Range(6, 4) should be rejected");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("min") || msg.contains("max"),
            "error message should mention min/max, got: {msg}"
        );
    }

    // -- TierDefinition deserialization --

    #[test]
    fn tier_definition_deserializes_from_ron() {
        let ron_str = "(nodes: Range(4, 6), active_ratio: 0.2, hp_mult: 1.3, timer_mult: 0.9, introduced_cells: ['T'])";
        let tier: TierDefinition =
            ron::de::from_str(ron_str).expect("TierDefinition should deserialize");
        assert_eq!(tier.nodes, TierNodeCount::Range(4, 6));
        assert!((tier.active_ratio - 0.2).abs() < f32::EPSILON);
        assert!((tier.hp_mult - 1.3).abs() < f32::EPSILON);
        assert!((tier.timer_mult - 0.9).abs() < f32::EPSILON);
        assert_eq!(tier.introduced_cells, vec!['T']);
    }

    #[test]
    fn tier_definition_empty_introduced_cells_deserializes() {
        let ron_str = "(nodes: Fixed(3), active_ratio: 0.0, hp_mult: 1.0, timer_mult: 1.0, introduced_cells: [])";
        let tier: TierDefinition =
            ron::de::from_str(ron_str).expect("empty introduced_cells should deserialize");
        assert!(tier.introduced_cells.is_empty());
    }

    // -- DifficultyCurveDefaults deserialization --

    #[test]
    fn difficulty_curve_defaults_deserializes_from_ron() {
        let ron_str = "
(
    tiers: [
        (nodes: Fixed(3), active_ratio: 0.0, hp_mult: 1.0, timer_mult: 1.0, introduced_cells: []),
        (nodes: Range(4, 6), active_ratio: 0.5, hp_mult: 1.5, timer_mult: 0.8, introduced_cells: ['T']),
    ],
    boss_hp_mult: 3.0,
    timer_reduction_per_boss: 0.1,
)";
        let defaults: DifficultyCurveDefaults =
            ron::de::from_str(ron_str).expect("DifficultyCurveDefaults should deserialize");
        assert_eq!(defaults.tiers.len(), 2);
        assert!((defaults.boss_hp_mult - 3.0).abs() < f32::EPSILON);
        assert!((defaults.timer_reduction_per_boss - 0.1).abs() < f32::EPSILON);
        assert_eq!(defaults.tiers[0].nodes, TierNodeCount::Fixed(3));
        assert!((defaults.tiers[0].active_ratio - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn difficulty_curve_defaults_empty_tiers_deserializes() {
        let ron_str = "
(
    tiers: [],
    boss_hp_mult: 2.0,
    timer_reduction_per_boss: 0.05,
)";
        let defaults: DifficultyCurveDefaults =
            ron::de::from_str(ron_str).expect("empty tiers should deserialize");
        assert!(defaults.tiers.is_empty());
    }

    // -- difficulty.ron file parse --

    #[test]
    fn difficulty_ron_file_parses() {
        let ron_str = include_str!("../../assets/config/defaults.difficulty.ron");
        let defaults: DifficultyCurveDefaults = ron::de::from_str(ron_str)
            .expect("difficulty.ron should parse as DifficultyCurveDefaults");
        assert_eq!(defaults.tiers.len(), 5);
        assert!((defaults.boss_hp_mult - 3.0).abs() < f32::EPSILON);
        assert!((defaults.timer_reduction_per_boss - 0.1).abs() < f32::EPSILON);
    }
}
