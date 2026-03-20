//! Cell type definition — RON-deserialized data for a single cell type.

use bevy::prelude::*;
use serde::Deserialize;

/// Optional behavior flags for a cell type.
#[derive(Deserialize, Clone, Debug, Default)]
pub(crate) struct CellBehavior {
    /// Whether this cell starts locked (immune to damage until adjacents are cleared).
    #[serde(default)]
    pub locked: bool,
    /// If set, HP regenerates at this rate per second.
    #[serde(default)]
    pub regen_rate: Option<f32>,
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
}
