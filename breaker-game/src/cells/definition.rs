//! Cell type definition — RON-deserialized data for a single cell type.

use bevy::prelude::*;
use serde::Deserialize;

use crate::effect::RootEffect;

/// Configuration for a shield cell's orbiting children.
#[derive(Deserialize, Clone, Debug)]
pub(crate) struct ShieldBehavior {
    /// Number of orbit cells to spawn around the shield.
    pub count: u32,
    /// Distance from shield center to orbit cell center (before grid scaling).
    pub radius: f32,
    /// Angular speed in radians per second.
    pub speed: f32,
    /// Hit points for each orbit cell.
    pub hp: f32,
    /// HDR RGB color for orbit cells.
    pub color_rgb: [f32; 3],
}

impl ShieldBehavior {
    /// Validates that all fields are well-formed.
    ///
    /// Checks:
    /// - `radius` must be finite and positive (> 0.0).
    /// - `speed` must be finite and non-negative (>= 0.0).
    /// - `hp` must be finite and positive (> 0.0).
    ///
    /// # Errors
    ///
    /// Returns an error string describing the first invalid field found.
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.radius <= 0.0 || !self.radius.is_finite() {
            return Err(format!(
                "radius must be positive and finite, got {}",
                self.radius
            ));
        }
        if self.speed < 0.0 || !self.speed.is_finite() {
            return Err(format!(
                "speed must be non-negative and finite, got {}",
                self.speed
            ));
        }
        if self.hp <= 0.0 || !self.hp.is_finite() {
            return Err(format!("hp must be positive and finite, got {}", self.hp));
        }
        Ok(())
    }
}

/// Behavioral variants that can be attached to a cell type.
///
/// Each variant represents a distinct runtime behavior. A cell type may have
/// zero or more behaviors via `Option<Vec<CellBehavior>>`.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) enum CellBehavior {
    /// Cell regenerates HP at the given rate per second.
    Regen { rate: f32 },
}

/// A cell type definition loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub(crate) struct CellTypeDefinition {
    /// Unique identifier.
    pub id: String,
    /// Alias used in node layout grids — may be multi-character.
    pub alias: String,
    /// Hit points for this cell type.
    pub hp: f32,
    /// HDR RGB color.
    pub color_rgb: [f32; 3],
    /// Whether this cell counts toward node completion.
    pub required_to_clear: bool,
    /// HDR intensity multiplier for damaged cells at full health.
    pub damage_hdr_base: f32,
    /// Minimum green channel value for damage color feedback.
    pub damage_green_min: f32,
    /// Blue channel range added based on health fraction.
    pub damage_blue_range: f32,
    /// Base blue channel value for damage color feedback.
    pub damage_blue_base: f32,
    /// Optional behavior list (regen, etc.). Defaults to `None`.
    #[serde(default)]
    pub behaviors: Option<Vec<CellBehavior>>,
    /// Optional shield configuration. Stays as a separate field until Wave 4.
    #[serde(default)]
    pub shield: Option<ShieldBehavior>,
    /// Optional effect chains for this cell type. Defaults to `None`.
    #[serde(default)]
    pub effects: Option<Vec<RootEffect>>,
}

impl CellTypeDefinition {
    /// Cell color as a Bevy [`Color`].
    #[must_use]
    pub(crate) const fn color(&self) -> Color {
        crate::shared::color_from_rgb(self.color_rgb)
    }

    /// Validates that all fields of this definition are well-formed at runtime.
    ///
    /// Checks:
    /// - `hp` must be finite and positive (> 0.0).
    /// - `alias` must not be empty or the reserved `"."`.
    /// - Each `CellBehavior::Regen { rate }` must have a finite positive rate.
    /// - `shield`, if present, must pass its own validation.
    ///
    /// # Errors
    ///
    /// Returns an error string describing the first invalid field found.
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.hp <= 0.0 || !self.hp.is_finite() {
            return Err(format!("hp must be positive and finite, got {}", self.hp));
        }
        if self.alias.is_empty() {
            return Err("alias must not be empty".to_owned());
        }
        if self.alias == "." {
            return Err("alias '.' is reserved for empty grid positions".to_owned());
        }
        if let Some(ref behaviors) = self.behaviors {
            for behavior in behaviors {
                match behavior {
                    CellBehavior::Regen { rate } => {
                        if *rate <= 0.0 || !rate.is_finite() {
                            return Err(format!(
                                "regen rate must be positive and finite, got {rate}"
                            ));
                        }
                    }
                }
            }
        }
        if let Some(ref shield) = self.shield {
            shield.validate()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a valid [`CellTypeDefinition`] with sensible defaults.
    /// Individual tests override fields to test specific validation rules.
    fn valid_definition() -> CellTypeDefinition {
        CellTypeDefinition {
            id: "test".to_owned(),
            alias: "T".to_owned(),
            hp: 20.0,
            color_rgb: [1.0, 0.5, 0.2],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behaviors: None,
            shield: None,
            effects: None,
        }
    }

    // ── hp validation ────────────────────────────────────────────────

    #[test]
    fn validate_rejects_zero_hp() {
        let mut def = valid_definition();
        def.hp = 0.0;
        assert!(def.validate().is_err(), "hp = 0.0 should be rejected");
    }

    #[test]
    fn validate_rejects_negative_hp() {
        let mut def = valid_definition();
        def.hp = -1.0;
        assert!(def.validate().is_err(), "hp = -1.0 should be rejected");
    }

    #[test]
    fn validate_rejects_nan_hp() {
        let mut def = valid_definition();
        def.hp = f32::NAN;
        assert!(def.validate().is_err(), "hp = NaN should be rejected");
    }

    #[test]
    fn validate_rejects_infinite_hp() {
        let mut def = valid_definition();
        def.hp = f32::INFINITY;
        assert!(def.validate().is_err(), "hp = INFINITY should be rejected");
    }

    // ── CellBehavior enum tests (Part A behaviors 1-2) ──────────────

    #[test]
    fn cell_behavior_regen_deserializes_from_ron() {
        let ron_str = "Regen(rate: 2.0)";
        let result: CellBehavior = ron::de::from_str(ron_str).expect("should deserialize");
        assert_eq!(result, CellBehavior::Regen { rate: 2.0 });
    }

    #[test]
    fn cell_behavior_regen_smallest_positive_rate_deserializes() {
        let ron_str = "Regen(rate: 0.001)";
        let result: CellBehavior = ron::de::from_str(ron_str).expect("should deserialize");
        assert_eq!(result, CellBehavior::Regen { rate: 0.001 });
    }

    #[test]
    fn cell_behavior_is_clone_debug() {
        let behavior = CellBehavior::Regen { rate: 3.5 };
        let cloned = behavior.clone();
        assert_eq!(behavior, cloned, "clone should equal original");
        let debug_str = format!("{behavior:?}");
        assert!(
            debug_str.contains("Regen"),
            "debug should contain 'Regen', got: {debug_str}"
        );
        assert!(
            debug_str.contains("3.5"),
            "debug should contain '3.5', got: {debug_str}"
        );
    }

    // ── CellTypeDefinition deserialization (Part A behaviors 3-5, 14) ─

    #[test]
    fn definition_with_no_behaviors_field_deserializes_to_none() {
        let ron_str = r#"(
            id: "test",
            alias: "S",
            hp: 10.0,
            color_rgb: (1.0, 0.5, 0.2),
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
        )"#;
        let def: CellTypeDefinition =
            ron::de::from_str(ron_str).expect("should deserialize without behaviors field");
        assert!(
            def.behaviors.is_none(),
            "missing behaviors field should default to None"
        );
    }

    #[test]
    fn definition_with_explicit_behaviors_none_deserializes() {
        let ron_str = r#"(
            id: "test",
            alias: "S",
            hp: 10.0,
            color_rgb: (1.0, 0.5, 0.2),
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behaviors: None,
        )"#;
        let def: CellTypeDefinition =
            ron::de::from_str(ron_str).expect("should deserialize with behaviors: None");
        assert!(
            def.behaviors.is_none(),
            "explicit behaviors: None should produce None"
        );
    }

    #[test]
    fn definition_with_empty_behaviors_vec_deserializes() {
        let ron_str = r#"(
            id: "test",
            alias: "S",
            hp: 10.0,
            color_rgb: (1.0, 0.5, 0.2),
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behaviors: Some([]),
        )"#;
        let def: CellTypeDefinition =
            ron::de::from_str(ron_str).expect("should deserialize with behaviors: Some([])");
        assert_eq!(
            def.behaviors,
            Some(vec![]),
            "behaviors: Some([]) should produce Some(empty vec)"
        );
    }

    #[test]
    fn definition_with_single_regen_behavior_deserializes() {
        let ron_str = r#"(
            id: "regen",
            alias: "R",
            hp: 20.0,
            color_rgb: (0.3, 4.0, 0.3),
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.4,
            damage_blue_range: 0.3,
            damage_blue_base: 0.1,
            behaviors: Some([Regen(rate: 2.0)]),
        )"#;
        let def: CellTypeDefinition =
            ron::de::from_str(ron_str).expect("should deserialize with Regen behavior");
        assert_eq!(
            def.behaviors,
            Some(vec![CellBehavior::Regen { rate: 2.0 }]),
            "should parse single Regen behavior"
        );
    }

    #[test]
    fn definition_alias_is_string() {
        let ron_str = r#"(
            id: "test",
            alias: "S",
            hp: 10.0,
            color_rgb: (1.0, 0.5, 0.2),
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
        )"#;
        let def: CellTypeDefinition = ron::de::from_str(ron_str).expect("should deserialize");
        assert_eq!(def.alias, "S".to_owned(), "alias should be a String");
    }

    #[test]
    fn definition_multi_char_alias_deserializes() {
        let ron_str = r#"(
            id: "guard",
            alias: "Gu",
            hp: 10.0,
            color_rgb: (1.0, 0.5, 0.2),
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
        )"#;
        let def: CellTypeDefinition = ron::de::from_str(ron_str).expect("should deserialize");
        assert_eq!(
            def.alias,
            "Gu".to_owned(),
            "multi-char alias should deserialize"
        );
    }

    // ── validate() for behaviors (Part A behaviors 6-13) ─────────────

    #[test]
    fn validate_accepts_valid_definition_with_regen_behavior() {
        let mut def = valid_definition();
        def.behaviors = Some(vec![CellBehavior::Regen { rate: 2.0 }]);
        assert!(
            def.validate().is_ok(),
            "valid Regen {{ rate: 2.0 }} should pass: {:?}",
            def.validate(),
        );
    }

    #[test]
    fn validate_accepts_regen_with_very_small_positive_rate() {
        let mut def = valid_definition();
        def.behaviors = Some(vec![CellBehavior::Regen { rate: 0.001 }]);
        assert!(
            def.validate().is_ok(),
            "Regen {{ rate: 0.001 }} should pass: {:?}",
            def.validate(),
        );
    }

    #[test]
    fn validate_rejects_regen_with_zero_rate() {
        let mut def = valid_definition();
        def.behaviors = Some(vec![CellBehavior::Regen { rate: 0.0 }]);
        let err = def.validate().expect_err("rate = 0.0 should be rejected");
        let err_lower = err.to_lowercase();
        assert!(
            err_lower.contains("regen") || err_lower.contains('0'),
            "error should mention regen or zero, got: {err}"
        );
    }

    #[test]
    fn validate_rejects_regen_with_negative_rate() {
        let mut def = valid_definition();
        def.behaviors = Some(vec![CellBehavior::Regen { rate: -1.0 }]);
        assert!(def.validate().is_err(), "rate = -1.0 should be rejected");
    }

    #[test]
    fn validate_rejects_regen_with_tiny_negative_rate() {
        let mut def = valid_definition();
        def.behaviors = Some(vec![CellBehavior::Regen { rate: -0.001 }]);
        assert!(def.validate().is_err(), "rate = -0.001 should be rejected");
    }

    #[test]
    fn validate_rejects_regen_with_nan_rate() {
        let mut def = valid_definition();
        def.behaviors = Some(vec![CellBehavior::Regen { rate: f32::NAN }]);
        assert!(def.validate().is_err(), "rate = NaN should be rejected");
    }

    #[test]
    fn validate_rejects_regen_with_infinite_rate() {
        let mut def = valid_definition();
        def.behaviors = Some(vec![CellBehavior::Regen {
            rate: f32::INFINITY,
        }]);
        assert!(
            def.validate().is_err(),
            "rate = INFINITY should be rejected"
        );
    }

    #[test]
    fn validate_rejects_regen_with_neg_infinite_rate() {
        let mut def = valid_definition();
        def.behaviors = Some(vec![CellBehavior::Regen {
            rate: f32::NEG_INFINITY,
        }]);
        assert!(
            def.validate().is_err(),
            "rate = NEG_INFINITY should be rejected"
        );
    }

    #[test]
    fn validate_accepts_behaviors_none() {
        let mut def = valid_definition();
        def.behaviors = None;
        assert!(
            def.validate().is_ok(),
            "behaviors: None should pass: {:?}",
            def.validate(),
        );
    }

    #[test]
    fn validate_accepts_empty_behaviors_vec() {
        let mut def = valid_definition();
        def.behaviors = Some(vec![]);
        assert!(
            def.validate().is_ok(),
            "behaviors: Some(vec![]) should pass: {:?}",
            def.validate(),
        );
    }

    #[test]
    fn validate_rejects_when_any_behavior_invalid() {
        // First valid, second invalid
        let mut def = valid_definition();
        def.behaviors = Some(vec![
            CellBehavior::Regen { rate: 2.0 },
            CellBehavior::Regen { rate: -1.0 },
        ]);
        assert!(
            def.validate().is_err(),
            "behaviors with one invalid entry should be rejected"
        );
    }

    #[test]
    fn validate_rejects_when_first_behavior_invalid() {
        // First invalid, second valid
        let mut def = valid_definition();
        def.behaviors = Some(vec![
            CellBehavior::Regen { rate: -1.0 },
            CellBehavior::Regen { rate: 2.0 },
        ]);
        assert!(
            def.validate().is_err(),
            "behaviors with first entry invalid should be rejected"
        );
    }

    // ── alias validation (Part A behaviors 15-16) ────────────────────

    #[test]
    fn validate_rejects_empty_alias() {
        let mut def = valid_definition();
        def.alias = String::new();
        assert!(def.validate().is_err(), "empty alias should be rejected");
    }

    #[test]
    fn validate_rejects_dot_alias() {
        let mut def = valid_definition();
        def.alias = ".".to_owned();
        let err = def.validate().expect_err("dot alias should be rejected");
        assert!(
            err.contains("reserved") || err.contains('.'),
            "error should mention reserved or dot, got: {err}"
        );
    }

    // ── ShieldBehavior delegation (Part A behavior 17) ───────────────

    fn valid_shield() -> ShieldBehavior {
        ShieldBehavior {
            count: 3,
            radius: 60.0,
            speed: std::f32::consts::FRAC_PI_2,
            hp: 10.0,
            color_rgb: [0.5, 0.8, 1.0],
        }
    }

    #[test]
    fn shield_validate_accepts_valid_shield() {
        let shield = valid_shield();
        assert!(
            shield.validate().is_ok(),
            "valid ShieldBehavior should pass validation: {:?}",
            shield.validate(),
        );
    }

    #[test]
    fn shield_validate_rejects_zero_orbit_radius() {
        let mut shield = valid_shield();
        shield.radius = 0.0;
        assert!(
            shield.validate().is_err(),
            "orbit_radius = 0.0 should be rejected"
        );
    }

    #[test]
    fn shield_validate_rejects_negative_orbit_radius() {
        let mut shield = valid_shield();
        shield.radius = -10.0;
        assert!(
            shield.validate().is_err(),
            "orbit_radius = -10.0 should be rejected"
        );
    }

    #[test]
    fn shield_validate_rejects_infinite_orbit_radius() {
        let mut shield = valid_shield();
        shield.radius = f32::INFINITY;
        assert!(
            shield.validate().is_err(),
            "orbit_radius = INFINITY should be rejected"
        );
    }

    #[test]
    fn shield_validate_rejects_nan_orbit_radius() {
        let mut shield = valid_shield();
        shield.radius = f32::NAN;
        assert!(
            shield.validate().is_err(),
            "orbit_radius = NaN should be rejected"
        );
    }

    #[test]
    fn shield_validate_rejects_negative_orbit_speed() {
        let mut shield = valid_shield();
        shield.speed = -1.0;
        assert!(
            shield.validate().is_err(),
            "orbit_speed = -1.0 should be rejected"
        );
    }

    #[test]
    fn shield_validate_accepts_zero_orbit_speed() {
        // Zero speed means orbit cells don't rotate, which is valid.
        let mut shield = valid_shield();
        shield.speed = 0.0;
        assert!(
            shield.validate().is_ok(),
            "orbit_speed = 0.0 should be accepted (stationary orbits)"
        );
    }

    #[test]
    fn shield_validate_rejects_infinite_orbit_speed() {
        let mut shield = valid_shield();
        shield.speed = f32::INFINITY;
        assert!(
            shield.validate().is_err(),
            "orbit_speed = INFINITY should be rejected"
        );
    }

    #[test]
    fn shield_validate_rejects_zero_orbit_hp() {
        let mut shield = valid_shield();
        shield.hp = 0.0;
        assert!(
            shield.validate().is_err(),
            "orbit_hp = 0.0 should be rejected"
        );
    }

    #[test]
    fn shield_validate_rejects_negative_orbit_hp() {
        let mut shield = valid_shield();
        shield.hp = -5.0;
        assert!(
            shield.validate().is_err(),
            "orbit_hp = -5.0 should be rejected"
        );
    }

    #[test]
    fn shield_validate_rejects_nan_orbit_hp() {
        let mut shield = valid_shield();
        shield.hp = f32::NAN;
        assert!(
            shield.validate().is_err(),
            "orbit_hp = NaN should be rejected"
        );
    }

    #[test]
    fn cell_definition_validate_delegates_to_shield_validate() {
        let mut def = valid_definition();
        def.shield = Some(ShieldBehavior {
            count: 3,
            radius: -1.0, // invalid
            speed: std::f32::consts::FRAC_PI_2,
            hp: 10.0,
            color_rgb: [0.5, 0.8, 1.0],
        });
        assert!(
            def.validate().is_err(),
            "CellTypeDefinition.validate should reject invalid ShieldBehavior"
        );
    }

    #[test]
    fn cell_definition_validate_accepts_valid_shield() {
        let mut def = valid_definition();
        def.shield = Some(valid_shield());
        assert!(
            def.validate().is_ok(),
            "CellTypeDefinition with valid ShieldBehavior should pass: {:?}",
            def.validate(),
        );
    }

    #[test]
    fn validate_accepts_valid_definition_without_behaviors() {
        let def = valid_definition();
        assert!(
            def.validate().is_ok(),
            "valid definition with behaviors = None should pass: {:?}",
            def.validate(),
        );
    }
}
