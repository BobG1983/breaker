use super::{
    super::data::*,
    helpers::{valid_definition, valid_guarded_behavior},
};

// ── Part C: GuardedBehavior with guardian_hp_fraction ────────────

// Behavior 8: GuardedBehavior has guardian_hp_fraction field
#[test]
fn guarded_behavior_has_guardian_hp_fraction() {
    let gb = GuardedBehavior {
        guardian_hp_fraction: 0.5,
        guardian_color_rgb:   [0.5, 0.8, 1.0],
        slide_speed:          30.0,
    };
    assert!((gb.guardian_hp_fraction - 0.5).abs() < f32::EPSILON);
}

// Behavior 8 edge case: fraction of 1.0 (guardian HP = parent HP)
#[test]
fn guarded_behavior_fraction_one_valid() {
    let mut gb = valid_guarded_behavior();
    gb.guardian_hp_fraction = 1.0;
    assert!(gb.validate().is_ok(), "fraction 1.0 should be valid");
}

// Behavior 9: validate() checks guardian_hp_fraction in (0.0, 1.0]
#[test]
fn guarded_behavior_validate_accepts_valid_fraction() {
    let gb = valid_guarded_behavior();
    assert!(gb.validate().is_ok());
}

#[test]
fn guarded_behavior_validate_rejects_zero_fraction() {
    let mut gb = valid_guarded_behavior();
    gb.guardian_hp_fraction = 0.0;
    assert!(gb.validate().is_err(), "fraction 0.0 should be rejected");
}

#[test]
fn guarded_behavior_validate_rejects_negative_fraction() {
    let mut gb = valid_guarded_behavior();
    gb.guardian_hp_fraction = -0.5;
    assert!(gb.validate().is_err(), "fraction -0.5 should be rejected");
}

#[test]
fn guarded_behavior_validate_rejects_fraction_above_one() {
    let mut gb = valid_guarded_behavior();
    gb.guardian_hp_fraction = 1.5;
    assert!(gb.validate().is_err(), "fraction 1.5 should be rejected");
}

#[test]
fn guarded_behavior_validate_rejects_nan_fraction() {
    let mut gb = valid_guarded_behavior();
    gb.guardian_hp_fraction = f32::NAN;
    assert!(gb.validate().is_err(), "fraction NaN should be rejected");
}

#[test]
fn guarded_behavior_validate_rejects_infinite_fraction() {
    let mut gb = valid_guarded_behavior();
    gb.guardian_hp_fraction = f32::INFINITY;
    assert!(
        gb.validate().is_err(),
        "fraction INFINITY should be rejected"
    );
}

#[test]
fn guarded_behavior_validate_accepts_smallest_practical_fraction() {
    let mut gb = valid_guarded_behavior();
    gb.guardian_hp_fraction = 0.001;
    assert!(gb.validate().is_ok(), "fraction 0.001 should be valid");
}

// Behavior 10: GuardedBehavior deserializes from RON with guardian_hp_fraction
#[test]
fn cell_behavior_guarded_deserializes_with_fraction() {
    let ron_str = "Guarded((guardian_hp_fraction: 0.5, guardian_color_rgb: (0.5, 0.8, 1.0), slide_speed: 30.0))";
    let result: CellBehavior = ron::de::from_str(ron_str).expect("should deserialize");
    assert_eq!(
        result,
        CellBehavior::Guarded(GuardedBehavior {
            guardian_hp_fraction: 0.5,
            guardian_color_rgb:   [0.5, 0.8, 1.0],
            slide_speed:          30.0,
        })
    );
}

// Behavior 10 edge case: fraction 1.0 deserializes
#[test]
fn cell_behavior_guarded_fraction_one_deserializes() {
    let ron_str = "Guarded((guardian_hp_fraction: 1.0, guardian_color_rgb: (0.5, 0.8, 1.0), slide_speed: 30.0))";
    let result: CellBehavior = ron::de::from_str(ron_str).expect("should deserialize");
    match result {
        CellBehavior::Guarded(g) => {
            assert!((g.guardian_hp_fraction - 1.0).abs() < f32::EPSILON);
        }
        CellBehavior::Regen { .. }
        | CellBehavior::Volatile { .. }
        | CellBehavior::Sequence { .. }
        | CellBehavior::Armored { .. }
        | CellBehavior::Phantom { .. }
        | CellBehavior::Magnetic { .. }
        | CellBehavior::Survival { .. }
        | CellBehavior::SurvivalPermanent { .. }
        | CellBehavior::Portal { .. } => {
            panic!("expected Guarded variant")
        }
    }
}

// ── Existing tests updated for toughness/fraction ───────────────

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
    assert!(debug_str.contains("Regen"));
    assert!(debug_str.contains("3.5"));
}

// ── slide_speed validation ──────────────────────────────────────

#[test]
fn guarded_behavior_validate_accepts_zero_slide_speed() {
    let mut gb = valid_guarded_behavior();
    gb.slide_speed = 0.0;
    assert!(gb.validate().is_ok());
}

#[test]
fn guarded_behavior_validate_rejects_negative_slide_speed() {
    let mut gb = valid_guarded_behavior();
    gb.slide_speed = -1.0;
    assert!(gb.validate().is_err());
}

#[test]
fn guarded_behavior_validate_rejects_nan_slide_speed() {
    let mut gb = valid_guarded_behavior();
    gb.slide_speed = f32::NAN;
    assert!(gb.validate().is_err());
}

#[test]
fn guarded_behavior_validate_rejects_infinite_slide_speed() {
    let mut gb = valid_guarded_behavior();
    gb.slide_speed = f32::INFINITY;
    assert!(gb.validate().is_err());
}

// ── CellTypeDefinition validation delegation ────────────────────

#[test]
fn cell_definition_validate_delegates_to_guarded_validate() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Guarded(GuardedBehavior {
        guardian_hp_fraction: 0.0, // invalid
        guardian_color_rgb:   [0.5, 0.8, 1.0],
        slide_speed:          30.0,
    })]);
    assert!(def.validate().is_err());
}

#[test]
fn cell_definition_validate_accepts_valid_guarded() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Guarded(valid_guarded_behavior())]);
    assert!(def.validate().is_ok());
}

#[test]
fn cell_definition_validate_accepts_regen_and_guarded() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![
        CellBehavior::Regen { rate: 2.0 },
        CellBehavior::Guarded(valid_guarded_behavior()),
    ]);
    assert!(def.validate().is_ok());
}
