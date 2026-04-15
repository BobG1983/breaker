//! Shared validation helpers for RON-loaded configuration types.
//!
//! Centralizes the positive-and-finite `f32` check that appears across
//! `CellTypeDefinition::validate()`, `ToughnessConfig::validate()`, and any
//! future definition validators. Each call site previously duplicated the
//! same `x <= 0.0 || !x.is_finite()` + `format!` error construction — this
//! module consolidates it behind a single helper so the check, the error
//! string format, and any future refinements (NaN-only vs non-finite
//! distinction, locale-aware formatting, etc.) have one owner.

/// Validates that `value` is positive and finite.
///
/// Returns `Ok(())` if `value > 0.0` and `value.is_finite()`. Otherwise
/// returns `Err(String)` with a message of the form
/// `"{name} must be positive and finite, got {value}"`.
///
/// `name` is the human-readable field name to surface in the error (e.g.
/// `"Regen rate"`, `"Volatile damage"`, `"tier_multiplier"`).
pub(crate) fn positive_finite_f32(name: &str, value: f32) -> Result<(), String> {
    if value <= 0.0 || !value.is_finite() {
        return Err(format!("{name} must be positive and finite, got {value}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_positive_finite() {
        assert!(positive_finite_f32("field", 1.0).is_ok());
        assert!(positive_finite_f32("field", f32::MIN_POSITIVE).is_ok());
        assert!(positive_finite_f32("field", 1e30).is_ok());
    }

    #[test]
    fn rejects_zero() {
        let err = positive_finite_f32("field", 0.0).unwrap_err();
        assert!(err.contains("field"));
        assert!(err.contains("must be positive and finite"));
    }

    #[test]
    fn rejects_negative_zero() {
        assert!(positive_finite_f32("field", -0.0).is_err());
    }

    #[test]
    fn rejects_negative() {
        assert!(positive_finite_f32("field", -1.0).is_err());
    }

    #[test]
    fn rejects_nan() {
        assert!(positive_finite_f32("field", f32::NAN).is_err());
    }

    #[test]
    fn rejects_positive_infinity() {
        assert!(positive_finite_f32("field", f32::INFINITY).is_err());
    }

    #[test]
    fn rejects_negative_infinity() {
        assert!(positive_finite_f32("field", f32::NEG_INFINITY).is_err());
    }

    #[test]
    fn error_message_includes_field_name_and_value() {
        let err = positive_finite_f32("Volatile damage", -2.5).unwrap_err();
        assert!(err.contains("Volatile damage"));
        assert!(err.contains("-2.5"));
    }
}
