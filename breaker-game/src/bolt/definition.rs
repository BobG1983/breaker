//! Bolt definition -- RON-deserialized bolt data.

use bevy::prelude::*;
use serde::Deserialize;

use crate::effect_v3::types::RootNode;

/// Default value for `min_angle_horizontal` when omitted from RON.
const fn default_min_angle_horizontal() -> f32 {
    5.0
}

/// Default value for `min_angle_vertical` when omitted from RON.
const fn default_min_angle_vertical() -> f32 {
    5.0
}

/// A bolt definition loaded from a RON file.
///
/// Defines base stats for a bolt archetype. Multiple bolts can share
/// the same definition. Loaded from `assets/bolts/*.bolt.ron`.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct BoltDefinition {
    /// Display name of the bolt archetype.
    pub name:                 String,
    /// Base movement speed in world units per second.
    pub base_speed:           f32,
    /// Minimum speed cap.
    pub min_speed:            f32,
    /// Maximum speed cap.
    pub max_speed:            f32,
    /// Bolt radius in world units.
    pub radius:               f32,
    /// Base damage per hit.
    pub base_damage:          f32,
    /// Effect chains bound to this bolt archetype.
    pub effects:              Vec<RootNode>,
    /// RGB values for the bolt HDR color.
    pub color_rgb:            [f32; 3],
    /// Minimum angle from horizontal in degrees.
    #[serde(default = "default_min_angle_horizontal")]
    pub min_angle_horizontal: f32,
    /// Minimum angle from vertical in degrees.
    #[serde(default = "default_min_angle_vertical")]
    pub min_angle_vertical:   f32,
    /// Minimum bolt radius after boosts and scaling.
    #[serde(default)]
    pub min_radius:           Option<f32>,
    /// Maximum bolt radius after boosts and scaling.
    #[serde(default)]
    pub max_radius:           Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Behavior 1: BoltDefinition parses from RON with all fields explicit ──

    #[test]
    fn bolt_definition_parses_all_fields_explicit() {
        let ron_str = r#"(
            name: "Bolt",
            base_speed: 720.0,
            min_speed: 360.0,
            max_speed: 1440.0,
            radius: 14.0,
            base_damage: 10.0,
            effects: [],
            color_rgb: (6.0, 5.0, 0.5),
            min_angle_horizontal: 5.0,
            min_angle_vertical: 5.0,
        )"#;
        let def: BoltDefinition =
            ron::de::from_str(ron_str).expect("RON with all fields should parse");
        assert_eq!(def.name, "Bolt");
        assert!((def.base_speed - 720.0).abs() < f32::EPSILON);
        assert!((def.min_speed - 360.0).abs() < f32::EPSILON);
        assert!((def.max_speed - 1440.0).abs() < f32::EPSILON);
        assert!((def.radius - 14.0).abs() < f32::EPSILON);
        assert!((def.base_damage - 10.0).abs() < f32::EPSILON);
        assert!(def.effects.is_empty());
        assert!((def.color_rgb[0] - 6.0).abs() < f32::EPSILON);
        assert!((def.color_rgb[1] - 5.0).abs() < f32::EPSILON);
        assert!((def.color_rgb[2] - 0.5).abs() < f32::EPSILON);
        assert!((def.min_angle_horizontal - 5.0).abs() < f32::EPSILON);
        assert!((def.min_angle_vertical - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bolt_definition_parses_fields_in_different_order() {
        let ron_str = r#"(
            color_rgb: (6.0, 5.0, 0.5),
            effects: [],
            name: "Bolt",
            radius: 14.0,
            base_damage: 10.0,
            max_speed: 1440.0,
            min_speed: 360.0,
            base_speed: 720.0,
            min_angle_vertical: 5.0,
            min_angle_horizontal: 5.0,
        )"#;
        let def: BoltDefinition =
            ron::de::from_str(ron_str).expect("RON with reordered fields should parse");
        assert_eq!(def.name, "Bolt");
        assert!((def.base_speed - 720.0).abs() < f32::EPSILON);
    }

    // ── Behavior 2: BoltDefinition serde defaults for angle fields ──

    #[test]
    fn bolt_definition_defaults_angle_fields_when_omitted() {
        let ron_str = r#"(
            name: "Bolt",
            base_speed: 720.0,
            min_speed: 360.0,
            max_speed: 1440.0,
            radius: 14.0,
            base_damage: 10.0,
            effects: [],
            color_rgb: (6.0, 5.0, 0.5),
        )"#;
        let def: BoltDefinition =
            ron::de::from_str(ron_str).expect("RON without angle fields should parse");
        assert!(
            (def.min_angle_horizontal - 5.0).abs() < f32::EPSILON,
            "min_angle_horizontal should default to 5.0, got {}",
            def.min_angle_horizontal
        );
        assert!(
            (def.min_angle_vertical - 5.0).abs() < f32::EPSILON,
            "min_angle_vertical should default to 5.0, got {}",
            def.min_angle_vertical
        );
    }

    #[test]
    fn bolt_definition_one_angle_explicit_other_defaults() {
        let ron_str = r#"(
            name: "Bolt",
            base_speed: 720.0,
            min_speed: 360.0,
            max_speed: 1440.0,
            radius: 14.0,
            base_damage: 10.0,
            effects: [],
            color_rgb: (6.0, 5.0, 0.5),
            min_angle_horizontal: 10.0,
        )"#;
        let def: BoltDefinition =
            ron::de::from_str(ron_str).expect("RON with one angle field should parse");
        assert!(
            (def.min_angle_horizontal - 10.0).abs() < f32::EPSILON,
            "explicit min_angle_horizontal should be 10.0, got {}",
            def.min_angle_horizontal
        );
        assert!(
            (def.min_angle_vertical - 5.0).abs() < f32::EPSILON,
            "omitted min_angle_vertical should default to 5.0, got {}",
            def.min_angle_vertical
        );
    }

    // ── Behavior 3: BoltDefinition parses the actual default.bolt.ron file ──

    #[test]
    fn bolt_definition_parses_default_bolt_ron_file() {
        let ron_str = include_str!("../../assets/bolts/default.bolt.ron");
        let def: BoltDefinition =
            ron::de::from_str(ron_str).expect("default.bolt.ron should parse");
        assert_eq!(def.name, "Bolt");
        assert!((def.base_speed - 720.0).abs() < f32::EPSILON);
        assert!((def.base_damage - 10.0).abs() < f32::EPSILON);
    }

    // ── Behavior 4: BoltDefinition implements Clone ──

    #[test]
    fn bolt_definition_clone_preserves_fields() {
        let def = BoltDefinition {
            name:                 "Heavy".to_string(),
            base_speed:           500.0,
            min_speed:            250.0,
            max_speed:            1000.0,
            radius:               20.0,
            base_damage:          20.0,
            effects:              vec![],
            color_rgb:            [6.0, 5.0, 0.5],
            min_angle_horizontal: 5.0,
            min_angle_vertical:   5.0,
            min_radius:           None,
            max_radius:           None,
        };
        let cloned = def.clone();
        assert_eq!(cloned.name, "Heavy");
        assert!((cloned.base_speed - 500.0).abs() < f32::EPSILON);
        assert!((cloned.base_damage - 20.0).abs() < f32::EPSILON);
        assert_eq!(def.name, "Heavy");
    }

    #[test]
    fn bolt_definition_clone_with_effects_preserves_entries() {
        use ordered_float::OrderedFloat;

        use crate::effect_v3::types::{EffectType, StampTarget, Tree};

        let def = BoltDefinition {
            name:                 "EffectBolt".to_string(),
            base_speed:           720.0,
            min_speed:            360.0,
            max_speed:            1440.0,
            radius:               14.0,
            base_damage:          10.0,
            effects:              vec![RootNode::Stamp(
                StampTarget::Bolt,
                Tree::Fire(EffectType::SpeedBoost(
                    crate::effect_v3::effects::SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                )),
            )],
            color_rgb:            [6.0, 5.0, 0.5],
            min_angle_horizontal: 5.0,
            min_angle_vertical:   5.0,
            min_radius:           None,
            max_radius:           None,
        };
        let cloned = def.clone();
        assert_eq!(
            cloned.effects.len(),
            1,
            "cloned effects should have 1 entry"
        );
        assert_eq!(def.effects.len(), 1);
    }

    // ── Behavior 5: BoltDefinition implements Debug ──

    #[test]
    fn bolt_definition_debug_contains_name() {
        let def = BoltDefinition {
            name:                 "Bolt".to_string(),
            base_speed:           720.0,
            min_speed:            360.0,
            max_speed:            1440.0,
            radius:               14.0,
            base_damage:          10.0,
            effects:              vec![],
            color_rgb:            [6.0, 5.0, 0.5],
            min_angle_horizontal: 5.0,
            min_angle_vertical:   5.0,
            min_radius:           None,
            max_radius:           None,
        };
        let debug_str = format!("{def:?}");
        assert!(
            debug_str.contains("Bolt"),
            "Debug output should contain 'Bolt', got: {debug_str}"
        );
    }

    // ── Behavior 6: BoltDefinition parses with effects ──

    #[test]
    fn bolt_definition_parses_with_effects() {
        use crate::effect_v3::types::StampTarget;

        let ron_str = r#"(
            name: "EffectBolt",
            base_speed: 720.0,
            min_speed: 360.0,
            max_speed: 1440.0,
            radius: 14.0,
            base_damage: 10.0,
            effects: [
                Stamp(Bolt, When(PerfectBumped, Fire(SpeedBoost((multiplier: 1.5))))),
            ],
            color_rgb: (6.0, 5.0, 0.5),
            min_angle_horizontal: 5.0,
            min_angle_vertical: 5.0,
        )"#;
        let def: BoltDefinition =
            ron::de::from_str(ron_str).expect("RON with effects should parse");
        assert_eq!(def.effects.len(), 1, "should have 1 root effect");
        match &def.effects[0] {
            RootNode::Stamp(target, _tree) => {
                assert_eq!(*target, StampTarget::Bolt);
            }
            RootNode::Spawn(..) => panic!("expected RootNode::Stamp"),
        }
    }

    #[test]
    fn bolt_definition_parses_with_empty_effects() {
        let ron_str = r#"(
            name: "Plain",
            base_speed: 720.0,
            min_speed: 360.0,
            max_speed: 1440.0,
            radius: 14.0,
            base_damage: 10.0,
            effects: [],
            color_rgb: (6.0, 5.0, 0.5),
        )"#;
        let def: BoltDefinition =
            ron::de::from_str(ron_str).expect("RON with empty effects should parse");
        assert!(
            def.effects.is_empty(),
            "empty effects vec should parse to Vec::new()"
        );
    }

    // ── Behavior 28: BoltDefinition min_radius/max_radius serde defaults ──

    #[test]
    fn bolt_definition_min_max_radius_default_to_none_when_omitted() {
        let ron_str = r#"(
            name: "Bolt",
            base_speed: 720.0,
            min_speed: 360.0,
            max_speed: 1440.0,
            radius: 14.0,
            base_damage: 10.0,
            effects: [],
            color_rgb: (6.0, 5.0, 0.5),
        )"#;
        let def: BoltDefinition =
            ron::de::from_str(ron_str).expect("RON without radius constraints should parse");
        assert_eq!(
            def.min_radius, None,
            "min_radius should default to None, got {:?}",
            def.min_radius
        );
        assert_eq!(
            def.max_radius, None,
            "max_radius should default to None, got {:?}",
            def.max_radius
        );
    }

    #[test]
    fn bolt_definition_min_max_radius_explicit_values_parse() {
        let ron_str = r#"(
            name: "Bolt",
            base_speed: 720.0,
            min_speed: 360.0,
            max_speed: 1440.0,
            radius: 14.0,
            base_damage: 10.0,
            effects: [],
            color_rgb: (6.0, 5.0, 0.5),
            min_radius: Some(4.0),
            max_radius: Some(20.0),
        )"#;
        let def: BoltDefinition =
            ron::de::from_str(ron_str).expect("RON with explicit radius constraints should parse");
        assert_eq!(
            def.min_radius,
            Some(4.0),
            "min_radius should be Some(4.0), got {:?}",
            def.min_radius
        );
        assert_eq!(
            def.max_radius,
            Some(20.0),
            "max_radius should be Some(20.0), got {:?}",
            def.max_radius
        );
    }
}
