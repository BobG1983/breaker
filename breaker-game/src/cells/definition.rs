//! Cell type definition — RON-deserialized data for a single cell type.

use bevy::prelude::*;
use serde::Deserialize;

/// Configuration for a shield cell's orbiting children.
#[derive(Deserialize, Clone, Debug)]
pub(crate) struct ShieldBehavior {
    /// Number of orbit cells to spawn around the shield.
    pub orbit_count: u32,
    /// Distance from shield center to orbit cell center (before grid scaling).
    pub orbit_radius: f32,
    /// Angular speed in radians per second.
    pub orbit_speed: f32,
    /// Hit points for each orbit cell.
    pub orbit_hp: f32,
    /// HDR RGB color for orbit cells.
    pub orbit_color_rgb: [f32; 3],
}

impl ShieldBehavior {
    /// Validates that all fields are well-formed.
    ///
    /// Checks:
    /// - `orbit_radius` must be finite and positive (> 0.0).
    /// - `orbit_speed` must be finite and non-negative (>= 0.0).
    /// - `orbit_hp` must be finite and positive (> 0.0).
    ///
    /// # Errors
    ///
    /// Returns an error string describing the first invalid field found.
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.orbit_radius <= 0.0 || !self.orbit_radius.is_finite() {
            return Err(format!(
                "orbit_radius must be positive and finite, got {}",
                self.orbit_radius
            ));
        }
        if self.orbit_speed < 0.0 || !self.orbit_speed.is_finite() {
            return Err(format!(
                "orbit_speed must be non-negative and finite, got {}",
                self.orbit_speed
            ));
        }
        if self.orbit_hp <= 0.0 || !self.orbit_hp.is_finite() {
            return Err(format!(
                "orbit_hp must be positive and finite, got {}",
                self.orbit_hp
            ));
        }
        Ok(())
    }
}

/// Optional behavior flags for a cell type.
#[derive(Deserialize, Clone, Debug, Default)]
pub(crate) struct CellBehavior {
    /// Whether this cell starts locked (immune to damage until adjacents are cleared).
    #[serde(default)]
    pub locked: bool,
    /// If set, HP regenerates at this rate per second.
    #[serde(default)]
    pub regen_rate: Option<f32>,
    /// If set, this cell is a shield that spawns orbiting children.
    #[serde(default)]
    pub shield: Option<ShieldBehavior>,
}

/// A cell type definition loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub(crate) struct CellTypeDefinition {
    /// Unique identifier.
    pub id: String,
    /// Single-char alias used in node layout grids.
    pub alias: char,
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
    /// Optional behavior flags (locked, regen). Defaults to no behavior.
    #[serde(default)]
    pub behavior: CellBehavior,
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
    /// - `behavior.regen_rate`, if `Some`, must be finite and positive (> 0.0).
    ///
    /// # Errors
    ///
    /// Returns an error string describing the first invalid field found.
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.hp <= 0.0 || !self.hp.is_finite() {
            return Err(format!("hp must be positive and finite, got {}", self.hp));
        }
        if let Some(rate) = self.behavior.regen_rate
            && (rate <= 0.0 || !rate.is_finite())
        {
            return Err(format!(
                "regen_rate must be positive and finite, got {rate}"
            ));
        }
        if let Some(ref shield) = self.behavior.shield {
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
            alias: 'T',
            hp: 20.0,
            color_rgb: [1.0, 0.5, 0.2],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behavior: CellBehavior::default(),
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

    // ── regen_rate validation ────────────────────────────────────────

    #[test]
    fn validate_rejects_zero_regen_rate() {
        let mut def = valid_definition();
        def.behavior.regen_rate = Some(0.0);
        assert!(
            def.validate().is_err(),
            "regen_rate = Some(0.0) should be rejected"
        );
    }

    #[test]
    fn validate_rejects_negative_regen_rate() {
        let mut def = valid_definition();
        def.behavior.regen_rate = Some(-1.0);
        assert!(
            def.validate().is_err(),
            "regen_rate = Some(-1.0) should be rejected"
        );
    }

    #[test]
    fn validate_rejects_infinite_regen_rate() {
        let mut def = valid_definition();
        def.behavior.regen_rate = Some(f32::INFINITY);
        assert!(
            def.validate().is_err(),
            "regen_rate = Some(INFINITY) should be rejected"
        );
    }

    // ── positive cases ──────────────────────────────────────────────

    #[test]
    fn validate_accepts_valid_definition_without_regen() {
        let def = valid_definition();
        assert!(
            def.validate().is_ok(),
            "valid definition with regen_rate = None should pass: {:?}",
            def.validate(),
        );
    }

    #[test]
    fn validate_accepts_valid_definition_with_regen() {
        let mut def = valid_definition();
        def.behavior.regen_rate = Some(2.0);
        assert!(
            def.validate().is_ok(),
            "valid definition with regen_rate = Some(2.0) should pass: {:?}",
            def.validate(),
        );
    }

    // ── ShieldBehavior validation ─────────────────────────────────

    fn valid_shield() -> ShieldBehavior {
        ShieldBehavior {
            orbit_count: 3,
            orbit_radius: 60.0,
            orbit_speed: std::f32::consts::FRAC_PI_2,
            orbit_hp: 10.0,
            orbit_color_rgb: [0.5, 0.8, 1.0],
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
        shield.orbit_radius = 0.0;
        assert!(
            shield.validate().is_err(),
            "orbit_radius = 0.0 should be rejected"
        );
    }

    #[test]
    fn shield_validate_rejects_negative_orbit_radius() {
        let mut shield = valid_shield();
        shield.orbit_radius = -10.0;
        assert!(
            shield.validate().is_err(),
            "orbit_radius = -10.0 should be rejected"
        );
    }

    #[test]
    fn shield_validate_rejects_infinite_orbit_radius() {
        let mut shield = valid_shield();
        shield.orbit_radius = f32::INFINITY;
        assert!(
            shield.validate().is_err(),
            "orbit_radius = INFINITY should be rejected"
        );
    }

    #[test]
    fn shield_validate_rejects_nan_orbit_radius() {
        let mut shield = valid_shield();
        shield.orbit_radius = f32::NAN;
        assert!(
            shield.validate().is_err(),
            "orbit_radius = NaN should be rejected"
        );
    }

    #[test]
    fn shield_validate_rejects_negative_orbit_speed() {
        let mut shield = valid_shield();
        shield.orbit_speed = -1.0;
        assert!(
            shield.validate().is_err(),
            "orbit_speed = -1.0 should be rejected"
        );
    }

    #[test]
    fn shield_validate_accepts_zero_orbit_speed() {
        // Zero speed means orbit cells don't rotate, which is valid.
        let mut shield = valid_shield();
        shield.orbit_speed = 0.0;
        assert!(
            shield.validate().is_ok(),
            "orbit_speed = 0.0 should be accepted (stationary orbits)"
        );
    }

    #[test]
    fn shield_validate_rejects_infinite_orbit_speed() {
        let mut shield = valid_shield();
        shield.orbit_speed = f32::INFINITY;
        assert!(
            shield.validate().is_err(),
            "orbit_speed = INFINITY should be rejected"
        );
    }

    #[test]
    fn shield_validate_rejects_zero_orbit_hp() {
        let mut shield = valid_shield();
        shield.orbit_hp = 0.0;
        assert!(
            shield.validate().is_err(),
            "orbit_hp = 0.0 should be rejected"
        );
    }

    #[test]
    fn shield_validate_rejects_negative_orbit_hp() {
        let mut shield = valid_shield();
        shield.orbit_hp = -5.0;
        assert!(
            shield.validate().is_err(),
            "orbit_hp = -5.0 should be rejected"
        );
    }

    #[test]
    fn shield_validate_rejects_nan_orbit_hp() {
        let mut shield = valid_shield();
        shield.orbit_hp = f32::NAN;
        assert!(
            shield.validate().is_err(),
            "orbit_hp = NaN should be rejected"
        );
    }

    #[test]
    fn cell_definition_validate_delegates_to_shield_validate() {
        let mut def = valid_definition();
        def.behavior.shield = Some(ShieldBehavior {
            orbit_count: 3,
            orbit_radius: -1.0, // invalid
            orbit_speed: std::f32::consts::FRAC_PI_2,
            orbit_hp: 10.0,
            orbit_color_rgb: [0.5, 0.8, 1.0],
        });
        assert!(
            def.validate().is_err(),
            "CellTypeDefinition.validate should reject invalid ShieldBehavior"
        );
    }

    #[test]
    fn cell_definition_validate_accepts_valid_shield() {
        let mut def = valid_definition();
        def.behavior.shield = Some(valid_shield());
        assert!(
            def.validate().is_ok(),
            "CellTypeDefinition with valid ShieldBehavior should pass: {:?}",
            def.validate(),
        );
    }
}
