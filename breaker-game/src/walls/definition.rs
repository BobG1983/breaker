//! Wall definition — RON-deserialized wall data.

use bevy::prelude::*;
use serde::Deserialize;

use crate::effect_v3::types::RootNode;

// ── Default value functions ─────────────────────────────────────────────────

const fn default_half_thickness() -> f32 {
    90.0
}

/// A wall definition loaded from a RON file.
///
/// All gameplay fields except `name` have serde defaults; RON files only need
/// to specify `name`.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct WallDefinition {
    /// Display name of the wall.
    pub name: String,
    /// Half-thickness of the wall in world units.
    #[serde(default = "default_half_thickness")]
    pub half_thickness: f32,
    /// Optional HDR color override (values may exceed 1.0 for bloom).
    #[serde(default)]
    pub color_rgb: Option<[f32; 3]>,
    /// Effect chains bound to this wall type.
    #[serde(default)]
    pub effects: Vec<RootNode>,
}

impl Default for WallDefinition {
    fn default() -> Self {
        Self {
            name: String::new(),
            half_thickness: default_half_thickness(),
            color_rgb: None,
            effects: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Behavior 1: WallDefinition parses RON with only name, others get serde defaults ──

    #[test]
    fn wall_definition_parses_ron_with_only_name_field() {
        let ron_str = r#"(name: "Wall")"#;
        let def: WallDefinition =
            ron::de::from_str(ron_str).expect("RON with only name should parse");
        assert_eq!(def.name, "Wall");
        assert!(
            (def.half_thickness - 90.0).abs() < f32::EPSILON,
            "half_thickness should default to 90.0, got {}",
            def.half_thickness
        );
        assert_eq!(def.color_rgb, None, "color_rgb should default to None");
        assert!(
            def.effects.is_empty(),
            "effects should default to empty vec"
        );
    }

    #[test]
    fn wall_definition_parses_ron_with_empty_name() {
        let ron_str = r#"(name: "")"#;
        let def: WallDefinition =
            ron::de::from_str(ron_str).expect("RON with empty name should parse");
        assert_eq!(def.name, "");
    }

    // ── Behavior 2: WallDefinition parses RON with all fields explicitly set ──

    #[test]
    fn wall_definition_parses_ron_with_all_fields_explicit() {
        let ron_str = r#"(name: "Bouncy", half_thickness: 45.0, color_rgb: Some((0.2, 2.0, 3.0)), effects: [])"#;
        let def: WallDefinition =
            ron::de::from_str(ron_str).expect("RON with all fields should parse");
        assert_eq!(def.name, "Bouncy");
        assert!(
            (def.half_thickness - 45.0).abs() < f32::EPSILON,
            "half_thickness should be 45.0, got {}",
            def.half_thickness
        );
        assert_eq!(
            def.color_rgb,
            Some([0.2, 2.0, 3.0]),
            "color_rgb should be Some([0.2, 2.0, 3.0])"
        );
        assert!(def.effects.is_empty(), "effects should be empty");
    }

    #[test]
    fn wall_definition_parses_ron_with_fields_in_different_order() {
        let ron_str = r#"(effects: [], color_rgb: None, name: "Reversed", half_thickness: 10.0)"#;
        let def: WallDefinition =
            ron::de::from_str(ron_str).expect("RON with reordered fields should parse");
        assert_eq!(def.name, "Reversed");
        assert!(
            (def.half_thickness - 10.0).abs() < f32::EPSILON,
            "half_thickness should be 10.0, got {}",
            def.half_thickness
        );
    }

    // ── Behavior 3: WallDefinition serde default for half_thickness is 90.0 ──

    #[test]
    fn wall_definition_defaults_half_thickness_to_90() {
        let ron_str = r#"(name: "Wall", color_rgb: Some((1.0, 1.0, 1.0)), effects: [])"#;
        let def: WallDefinition =
            ron::de::from_str(ron_str).expect("RON without half_thickness should parse");
        assert!(
            (def.half_thickness - 90.0).abs() < f32::EPSILON,
            "half_thickness should default to 90.0, got {}",
            def.half_thickness
        );
    }

    #[test]
    fn wall_definition_explicit_half_thickness_90_matches_default() {
        let ron_str = r#"(name: "Wall", half_thickness: 90.0, color_rgb: Some((1.0, 1.0, 1.0)), effects: [])"#;
        let def: WallDefinition =
            ron::de::from_str(ron_str).expect("RON with explicit 90.0 should parse");
        assert!(
            (def.half_thickness - 90.0).abs() < f32::EPSILON,
            "explicit half_thickness 90.0 should match default"
        );
    }

    // ── Behavior 4: WallDefinition serde default for color_rgb is None ──

    #[test]
    fn wall_definition_defaults_color_rgb_to_none() {
        let ron_str = r#"(name: "Wall", half_thickness: 50.0, effects: [])"#;
        let def: WallDefinition =
            ron::de::from_str(ron_str).expect("RON without color_rgb should parse");
        assert_eq!(def.color_rgb, None, "color_rgb should default to None");
    }

    #[test]
    fn wall_definition_explicit_color_rgb_none_matches_omission() {
        let ron_str = r#"(name: "Wall", half_thickness: 50.0, color_rgb: None, effects: [])"#;
        let def: WallDefinition =
            ron::de::from_str(ron_str).expect("RON with explicit None should parse");
        assert_eq!(
            def.color_rgb, None,
            "explicit color_rgb: None should parse identically to omission"
        );
    }

    // ── Behavior 5: WallDefinition serde default for effects is empty Vec ──

    #[test]
    fn wall_definition_defaults_effects_to_empty_vec() {
        let ron_str = r#"(name: "Wall")"#;
        let def: WallDefinition =
            ron::de::from_str(ron_str).expect("RON without effects should parse");
        assert!(
            def.effects.is_empty(),
            "effects should default to empty vec"
        );
    }

    #[test]
    fn wall_definition_explicit_empty_effects_matches_omission() {
        let ron_str = r#"(name: "Wall", effects: [])"#;
        let def: WallDefinition =
            ron::de::from_str(ron_str).expect("RON with explicit empty effects should parse");
        assert!(
            def.effects.is_empty(),
            "explicit effects: [] should parse identically to omission"
        );
    }

    // ── Behavior 6: WallDefinition rejects unknown fields ──

    #[test]
    fn wall_definition_rejects_unknown_fields() {
        let ron_str = r#"(name: "Wall", unknown_field: 42)"#;
        let result = ron::de::from_str::<WallDefinition>(ron_str);
        assert!(
            result.is_err(),
            "unknown field should cause deserialization error"
        );
    }

    #[test]
    fn wall_definition_rejects_misspelled_field() {
        let ron_str = r#"(name: "Wall", half_thicknes: 10.0)"#;
        let result = ron::de::from_str::<WallDefinition>(ron_str);
        assert!(
            result.is_err(),
            "misspelled field should cause deserialization error"
        );
    }

    // ── Behavior 7: WallDefinition parses RON with effects containing a RootEffect ──

    #[test]
    fn wall_definition_parses_ron_with_effects_containing_root_effect() {
        let ron_str = r#"(
            name: "EffectWall",
            effects: [
                Stamp(Bolt, When(Bumped, Fire(SpeedBoost((multiplier: 1.5))))),
            ],
        )"#;
        let def: WallDefinition =
            ron::de::from_str(ron_str).expect("RON with effects should parse");
        assert_eq!(def.effects.len(), 1, "should have 1 root effect");
        match &def.effects[0] {
            RootNode::Stamp(target, tree) => {
                assert_eq!(*target, crate::effect_v3::types::StampTarget::Bolt);
                match tree {
                    crate::effect_v3::types::Tree::When(trigger, _) => {
                        assert_eq!(*trigger, crate::effect_v3::types::Trigger::Bumped);
                    }
                    other => panic!("expected When node, got {other:?}"),
                }
            }
            other @ RootNode::Spawn(..) => panic!("expected Stamp node, got {other:?}"),
        }
    }

    #[test]
    fn wall_definition_parses_ron_with_multiple_root_effects() {
        let ron_str = r#"(
            name: "MultiEffectWall",
            effects: [
                Stamp(Bolt, Fire(SpeedBoost((multiplier: 1.5)))),
                Stamp(Bolt, Fire(SpeedBoost((multiplier: 2.0)))),
            ],
        )"#;
        let def: WallDefinition =
            ron::de::from_str(ron_str).expect("RON with multiple effects should parse");
        assert_eq!(def.effects.len(), 2, "should have 2 root effects");
    }

    // ── Behavior 8: WallDefinition implements Clone and preserves all fields ──

    #[test]
    fn wall_definition_clone_preserves_all_fields() {
        let ron_str = r#"(name: "TestWall", half_thickness: 50.0, color_rgb: Some((0.5, 1.0, 1.5)), effects: [])"#;
        let def: WallDefinition = ron::de::from_str(ron_str).expect("test RON should parse");
        let cloned = def.clone();
        assert_eq!(cloned.name, "TestWall");
        assert!(
            (cloned.half_thickness - 50.0).abs() < f32::EPSILON,
            "cloned half_thickness should be 50.0"
        );
        assert_eq!(
            cloned.color_rgb,
            Some([0.5, 1.0, 1.5]),
            "cloned color_rgb should match"
        );
        assert!(cloned.effects.is_empty());
        // original is still intact
        assert_eq!(def.name, "TestWall");
        assert!(
            (def.half_thickness - 50.0).abs() < f32::EPSILON,
            "original half_thickness should still be 50.0"
        );
    }

    #[test]
    fn wall_definition_clone_with_effects_preserves_length() {
        let ron_str = r#"(
            name: "EffectWall",
            effects: [
                Stamp(Bolt, Fire(SpeedBoost((multiplier: 1.5)))),
            ],
        )"#;
        let def: WallDefinition = ron::de::from_str(ron_str).expect("test RON should parse");
        let cloned = def.clone();
        assert_eq!(
            cloned.effects.len(),
            1,
            "cloned effects should have same length"
        );
        assert_eq!(def.effects.len(), 1, "original effects should be unchanged");
    }

    // ── Behavior 9: WallDefinition implements Debug and output contains name ──

    #[test]
    fn wall_definition_debug_contains_name() {
        let def = WallDefinition {
            name: "Wall".to_owned(),
            ..WallDefinition::default()
        };
        let debug_str = format!("{def:?}");
        assert!(
            debug_str.contains("Wall"),
            "Debug output should contain 'Wall', got: {debug_str}"
        );
    }

    #[test]
    fn wall_definition_debug_contains_field_names() {
        let def = WallDefinition::default();
        let debug_str = format!("{def:?}");
        assert!(
            debug_str.contains("half_thickness"),
            "Debug output should contain 'half_thickness', got: {debug_str}"
        );
    }

    // ── Behavior 10: WallDefinition Default impl produces correct values ──

    #[test]
    fn wall_definition_default_produces_correct_values() {
        let def = WallDefinition::default();
        assert_eq!(def.name, "", "default name should be empty string");
        assert!(
            (def.half_thickness - 90.0).abs() < f32::EPSILON,
            "default half_thickness should be 90.0, got {}",
            def.half_thickness
        );
        assert_eq!(def.color_rgb, None, "default color_rgb should be None");
        assert!(
            def.effects.is_empty(),
            "default effects should be empty vec"
        );
    }

    #[test]
    fn wall_definition_default_consistent_with_serde_defaults() {
        let ron_str = r#"(name: "")"#;
        let from_ron: WallDefinition =
            ron::de::from_str(ron_str).expect("RON with empty name should parse");
        let from_default = WallDefinition::default();
        assert_eq!(from_ron.name, from_default.name);
        assert!(
            (from_ron.half_thickness - from_default.half_thickness).abs() < f32::EPSILON,
            "serde default half_thickness should match Default impl"
        );
        assert_eq!(from_ron.color_rgb, from_default.color_rgb);
        assert_eq!(from_ron.effects.len(), from_default.effects.len());
    }

    // ── Behavior 11: WallDefinition parses the actual wall.wall.ron asset file ──

    #[test]
    fn wall_definition_parses_actual_asset_file() {
        let ron_str = include_str!("../../assets/walls/wall.wall.ron");
        let def: WallDefinition = ron::de::from_str(ron_str).expect("wall.wall.ron should parse");
        assert_eq!(def.name, "Wall");
        assert!(
            (def.half_thickness - 90.0).abs() < f32::EPSILON,
            "half_thickness should be default 90.0, got {}",
            def.half_thickness
        );
        assert_eq!(def.color_rgb, None, "color_rgb should be None");
        assert!(def.effects.is_empty(), "effects should be empty");
    }

    // ── Behavior 12: WallDefinition parses with HDR color values exceeding 1.0 ──

    #[test]
    fn wall_definition_parses_hdr_color_exceeding_one() {
        let ron_str = r#"(name: "GlowWall", color_rgb: Some((0.2, 2.0, 3.0)))"#;
        let def: WallDefinition =
            ron::de::from_str(ron_str).expect("RON with HDR color should parse");
        assert_eq!(
            def.color_rgb,
            Some([0.2, 2.0, 3.0]),
            "HDR values above 1.0 should be valid"
        );
    }

    #[test]
    fn wall_definition_parses_all_zero_color() {
        let ron_str = r#"(name: "DarkWall", color_rgb: Some((0.0, 0.0, 0.0)))"#;
        let def: WallDefinition =
            ron::de::from_str(ron_str).expect("RON with all-zero color should parse");
        assert_eq!(
            def.color_rgb,
            Some([0.0, 0.0, 0.0]),
            "all-zero color should parse to Some([0.0, 0.0, 0.0])"
        );
    }
}
