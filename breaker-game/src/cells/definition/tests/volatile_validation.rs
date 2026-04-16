use super::{super::data::*, helpers::valid_definition};

// ── Volatile validation (Wave 1) ────────────────────────────────

#[test]
fn validate_accepts_valid_volatile_behavior() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Volatile {
        damage: 25.0,
        radius: 40.0,
    }]);
    assert!(def.validate().is_ok());
}

#[test]
fn validate_accepts_volatile_with_minimum_positive_fields() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Volatile {
        damage: f32::MIN_POSITIVE,
        radius: f32::MIN_POSITIVE,
    }]);
    assert!(def.validate().is_ok());
}

#[test]
fn validate_rejects_volatile_with_zero_damage() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Volatile {
        damage: 0.0,
        radius: 40.0,
    }]);
    let err = def.validate().expect_err("zero damage should be rejected");
    assert!(
        err.to_lowercase().contains("damage"),
        "error should mention damage, got: {err}"
    );
}

#[test]
fn validate_rejects_volatile_with_negative_zero_damage() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Volatile {
        damage: -0.0_f32,
        radius: 40.0,
    }]);
    assert!(def.validate().is_err());
}

#[test]
fn validate_rejects_volatile_with_negative_damage() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Volatile {
        damage: -1.0,
        radius: 40.0,
    }]);
    let err = def
        .validate()
        .expect_err("negative damage should be rejected");
    assert!(
        err.to_lowercase().contains("damage"),
        "error should mention damage, got: {err}"
    );
}

#[test]
fn validate_rejects_volatile_with_f32_min_damage() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Volatile {
        damage: f32::MIN,
        radius: 40.0,
    }]);
    assert!(def.validate().is_err());
}

#[test]
fn validate_rejects_volatile_with_nan_damage() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Volatile {
        damage: f32::NAN,
        radius: 40.0,
    }]);
    assert!(def.validate().is_err());
}

#[test]
fn validate_rejects_volatile_with_infinite_damage() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Volatile {
        damage: f32::INFINITY,
        radius: 40.0,
    }]);
    assert!(def.validate().is_err());
}

#[test]
fn validate_rejects_volatile_with_negative_infinite_damage() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Volatile {
        damage: f32::NEG_INFINITY,
        radius: 40.0,
    }]);
    assert!(def.validate().is_err());
}

#[test]
fn validate_rejects_volatile_with_zero_radius() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Volatile {
        damage: 25.0,
        radius: 0.0,
    }]);
    let err = def.validate().expect_err("zero radius should be rejected");
    assert!(
        err.to_lowercase().contains("radius"),
        "error should mention radius, got: {err}"
    );
}

#[test]
fn validate_rejects_volatile_with_negative_zero_radius() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Volatile {
        damage: 25.0,
        radius: -0.0_f32,
    }]);
    assert!(def.validate().is_err());
}

#[test]
fn validate_rejects_volatile_with_negative_radius() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Volatile {
        damage: 25.0,
        radius: -1.0,
    }]);
    let err = def
        .validate()
        .expect_err("negative radius should be rejected");
    assert!(
        err.to_lowercase().contains("radius"),
        "error should mention radius, got: {err}"
    );
}

#[test]
fn validate_rejects_volatile_with_f32_min_radius() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Volatile {
        damage: 25.0,
        radius: f32::MIN,
    }]);
    assert!(def.validate().is_err());
}

#[test]
fn validate_rejects_volatile_with_nan_radius() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Volatile {
        damage: 25.0,
        radius: f32::NAN,
    }]);
    assert!(def.validate().is_err());
}

#[test]
fn validate_rejects_volatile_with_infinite_radius() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Volatile {
        damage: 25.0,
        radius: f32::INFINITY,
    }]);
    assert!(def.validate().is_err());
}

#[test]
fn validate_rejects_volatile_with_negative_infinite_radius() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Volatile {
        damage: 25.0,
        radius: f32::NEG_INFINITY,
    }]);
    assert!(def.validate().is_err());
}

#[test]
fn validate_rejects_volatile_with_both_fields_invalid() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Volatile {
        damage: -5.0,
        radius: -10.0,
    }]);
    assert!(def.validate().is_err());
}

#[test]
fn validate_accepts_volatile_mixed_with_regen() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![
        CellBehavior::Volatile {
            damage: 25.0,
            radius: 40.0,
        },
        CellBehavior::Regen { rate: 2.0 },
    ]);
    assert!(def.validate().is_ok());
}

#[test]
fn validate_accepts_regen_then_volatile() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![
        CellBehavior::Regen { rate: 2.0 },
        CellBehavior::Volatile {
            damage: 25.0,
            radius: 40.0,
        },
    ]);
    assert!(def.validate().is_ok());
}

#[test]
fn validate_rejects_invalid_volatile_after_valid_regen() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![
        CellBehavior::Regen { rate: 2.0 },
        CellBehavior::Volatile {
            damage: 0.0,
            radius: 40.0,
        },
    ]);
    let err = def.validate().expect_err("should reject zero damage");
    assert!(
        err.to_lowercase().contains("damage"),
        "error should mention damage, got: {err}"
    );
}

#[test]
fn validate_rejects_invalid_volatile_before_valid_regen() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![
        CellBehavior::Volatile {
            damage: 0.0,
            radius: 40.0,
        },
        CellBehavior::Regen { rate: 2.0 },
    ]);
    let err = def.validate().expect_err("should reject zero damage");
    assert!(
        err.to_lowercase().contains("damage"),
        "error should mention damage, got: {err}"
    );
}
