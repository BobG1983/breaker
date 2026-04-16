use super::{super::data::*, helpers::valid_definition};

// ── Regen rate validation ───────────────────────────────────────

#[test]
fn validate_accepts_valid_definition_with_regen_behavior() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Regen { rate: 2.0 }]);
    assert!(def.validate().is_ok());
}

#[test]
fn validate_rejects_regen_with_zero_rate() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Regen { rate: 0.0 }]);
    assert!(def.validate().is_err());
}

#[test]
fn validate_rejects_regen_with_negative_rate() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Regen { rate: -1.0 }]);
    assert!(def.validate().is_err());
}

#[test]
fn validate_rejects_regen_with_nan_rate() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Regen { rate: f32::NAN }]);
    assert!(def.validate().is_err());
}

#[test]
fn validate_rejects_regen_with_infinite_rate() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Regen {
        rate: f32::INFINITY,
    }]);
    assert!(def.validate().is_err());
}

#[test]
fn validate_accepts_behaviors_none() {
    let mut def = valid_definition();
    def.behaviors = None;
    assert!(def.validate().is_ok());
}

#[test]
fn validate_accepts_empty_behaviors_vec() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![]);
    assert!(def.validate().is_ok());
}
