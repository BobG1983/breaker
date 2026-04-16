use super::{super::data::*, helpers::valid_definition};

// ── Magnetic validation (Wave 5) ───────────────────────────────

// Behavior 23: CellBehavior::Magnetic variant validates with positive values
#[test]
fn validate_accepts_valid_magnetic_behavior() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Magnetic {
        radius:   200.0,
        strength: 1000.0,
    }]);
    assert!(def.validate().is_ok());
}

#[test]
fn validate_accepts_magnetic_with_very_small_positive_values() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Magnetic {
        radius:   0.001,
        strength: 0.001,
    }]);
    assert!(def.validate().is_ok());
}

// Behavior 24: Validation rejects zero radius
#[test]
fn validate_rejects_magnetic_with_zero_radius() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Magnetic {
        radius:   0.0,
        strength: 1000.0,
    }]);
    let err = def.validate().expect_err("zero radius should be rejected");
    assert!(
        err.to_lowercase().contains("radius"),
        "error should mention radius, got: {err}"
    );
    assert!(
        err.to_lowercase().contains("positive"),
        "error should mention positive, got: {err}"
    );
}

// Behavior 25: Validation rejects negative radius
#[test]
fn validate_rejects_magnetic_with_negative_radius() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Magnetic {
        radius:   -5.0,
        strength: 1000.0,
    }]);
    let err = def
        .validate()
        .expect_err("negative radius should be rejected");
    assert!(
        err.to_lowercase().contains("radius"),
        "error should mention radius, got: {err}"
    );
}

// Behavior 26: Validation rejects infinite radius
#[test]
fn validate_rejects_magnetic_with_infinite_radius() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Magnetic {
        radius:   f32::INFINITY,
        strength: 1000.0,
    }]);
    assert!(
        def.validate().is_err(),
        "infinite radius should be rejected"
    );
}

// Behavior 27: Validation rejects NaN radius
#[test]
fn validate_rejects_magnetic_with_nan_radius() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Magnetic {
        radius:   f32::NAN,
        strength: 1000.0,
    }]);
    assert!(def.validate().is_err(), "NaN radius should be rejected");
}

// Behavior 28: Validation rejects zero strength
#[test]
fn validate_rejects_magnetic_with_zero_strength() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Magnetic {
        radius:   200.0,
        strength: 0.0,
    }]);
    let err = def
        .validate()
        .expect_err("zero strength should be rejected");
    assert!(
        err.to_lowercase().contains("strength"),
        "error should mention strength, got: {err}"
    );
    assert!(
        err.to_lowercase().contains("positive"),
        "error should mention positive, got: {err}"
    );
}

// Behavior 29: Validation rejects negative strength
#[test]
fn validate_rejects_magnetic_with_negative_strength() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Magnetic {
        radius:   200.0,
        strength: -100.0,
    }]);
    let err = def
        .validate()
        .expect_err("negative strength should be rejected");
    assert!(
        err.to_lowercase().contains("strength"),
        "error should mention strength, got: {err}"
    );
}

// Behavior 30: Validation rejects infinite strength
#[test]
fn validate_rejects_magnetic_with_infinite_strength() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Magnetic {
        radius:   200.0,
        strength: f32::INFINITY,
    }]);
    assert!(
        def.validate().is_err(),
        "infinite strength should be rejected"
    );
}

// Behavior 31: Validation rejects NaN strength
#[test]
fn validate_rejects_magnetic_with_nan_strength() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Magnetic {
        radius:   200.0,
        strength: f32::NAN,
    }]);
    assert!(def.validate().is_err(), "NaN strength should be rejected");
}
