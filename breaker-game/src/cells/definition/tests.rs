use super::data::*;

/// Builds a valid [`CellTypeDefinition`] with sensible defaults.
/// Individual tests override fields to test specific validation rules.
fn valid_definition() -> CellTypeDefinition {
    CellTypeDefinition {
        id:                "test".to_owned(),
        alias:             "T".to_owned(),
        toughness:         Toughness::Standard,
        color_rgb:         [1.0, 0.5, 0.2],
        required_to_clear: true,
        damage_hdr_base:   4.0,
        damage_green_min:  0.2,
        damage_blue_range: 0.4,
        damage_blue_base:  0.2,
        behaviors:         None,
        effects:           None,
    }
}

fn valid_guarded_behavior() -> GuardedBehavior {
    GuardedBehavior {
        guardian_hp_fraction: 0.5,
        guardian_color_rgb:   [0.5, 0.8, 1.0],
        slide_speed:          30.0,
    }
}

// ── Part A: Toughness Enum ──────────────────────────────────────

// Behavior 1: Toughness enum has three variants
#[test]
fn toughness_has_three_distinct_variants() {
    // Exhaustive match proves all three variants exist.
    let label = |t: Toughness| match t {
        Toughness::Weak => "weak",
        Toughness::Standard => "standard",
        Toughness::Tough => "tough",
    };
    assert_eq!(label(Toughness::Weak), "weak");
    assert_eq!(label(Toughness::Standard), "standard");
    assert_eq!(label(Toughness::Tough), "tough");
}

// Behavior 1 edge case: Default variant is Standard
#[test]
fn toughness_default_is_standard() {
    assert_eq!(Toughness::default(), Toughness::Standard);
}

// Behavior 2: Toughness::default_base_hp() returns hardcoded fallback per variant
#[test]
fn toughness_weak_default_base_hp_returns_10() {
    assert!(
        (Toughness::Weak.default_base_hp() - 10.0).abs() < f32::EPSILON,
        "Weak.default_base_hp() should return 10.0, got {}",
        Toughness::Weak.default_base_hp()
    );
}

#[test]
fn toughness_standard_default_base_hp_returns_20() {
    assert!(
        (Toughness::Standard.default_base_hp() - 20.0).abs() < f32::EPSILON,
        "Standard.default_base_hp() should return 20.0, got {}",
        Toughness::Standard.default_base_hp()
    );
}

#[test]
fn toughness_tough_default_base_hp_returns_30() {
    assert!(
        (Toughness::Tough.default_base_hp() - 30.0).abs() < f32::EPSILON,
        "Tough.default_base_hp() should return 30.0, got {}",
        Toughness::Tough.default_base_hp()
    );
}

// Behavior 3: Toughness deserializes from RON
#[test]
fn toughness_weak_deserializes_from_ron() {
    let result: Toughness = ron::de::from_str("Weak").expect("should deserialize Weak");
    assert_eq!(result, Toughness::Weak);
}

#[test]
fn toughness_standard_deserializes_from_ron() {
    let result: Toughness = ron::de::from_str("Standard").expect("should deserialize Standard");
    assert_eq!(result, Toughness::Standard);
}

#[test]
fn toughness_tough_deserializes_from_ron() {
    let result: Toughness = ron::de::from_str("Tough").expect("should deserialize Tough");
    assert_eq!(result, Toughness::Tough);
}

// Behavior 3 edge case: invalid variant
#[test]
fn toughness_invalid_variant_fails_deserialization() {
    let result: Result<Toughness, _> = ron::de::from_str("Legendary");
    assert!(
        result.is_err(),
        "\"Legendary\" should not deserialize as Toughness"
    );
}

// Behavior 4: Toughness traits
#[test]
fn toughness_is_clone_copy_debug_eq() {
    let t = Toughness::Weak;
    let cloned = t;
    assert_eq!(t, cloned, "copy should equal original");
    let debug_str = format!("{t:?}");
    assert!(
        debug_str.contains("Weak"),
        "debug should contain 'Weak', got: {debug_str}"
    );
    assert_eq!(Toughness::Weak, Toughness::Weak);
    assert_ne!(Toughness::Weak, Toughness::Standard);
}

// Behavior 4 edge case: serialize round-trip
#[test]
fn toughness_serialize_round_trip() {
    let original = Toughness::Tough;
    let serialized = ron::ser::to_string(&original).expect("should serialize");
    let deserialized: Toughness = ron::de::from_str(&serialized).expect("should deserialize");
    assert_eq!(
        original, deserialized,
        "round-trip should produce same value"
    );
}

// ── Part B: CellTypeDefinition with toughness ───────────────────

// Behavior 5: CellTypeDefinition has toughness field
#[test]
fn definition_has_toughness_field() {
    let def = CellTypeDefinition {
        id:                "test".to_owned(),
        alias:             "T".to_owned(),
        toughness:         Toughness::Standard,
        color_rgb:         [1.0, 0.5, 0.2],
        required_to_clear: true,
        damage_hdr_base:   4.0,
        damage_green_min:  0.2,
        damage_blue_range: 0.4,
        damage_blue_base:  0.2,
        behaviors:         None,
        effects:           None,
    };
    assert_eq!(def.toughness, Toughness::Standard);
}

// Behavior 5 edge case: toughness defaults via serde when omitted
#[test]
fn definition_toughness_defaults_when_omitted_from_ron() {
    let ron_str = r#"(
        id: "test",
        alias: "T",
        color_rgb: (1.0, 0.5, 0.2),
        required_to_clear: true,
        damage_hdr_base: 4.0,
        damage_green_min: 0.2,
        damage_blue_range: 0.4,
        damage_blue_base: 0.2,
    )"#;
    let def: CellTypeDefinition =
        ron::de::from_str(ron_str).expect("should deserialize without toughness field");
    assert_eq!(
        def.toughness,
        Toughness::Standard,
        "toughness should default to Standard"
    );
}

// Behavior 6: validate() no longer checks hp
#[test]
fn validate_accepts_all_toughness_variants() {
    for toughness in [Toughness::Weak, Toughness::Standard, Toughness::Tough] {
        let mut def = valid_definition();
        def.toughness = toughness;
        assert!(
            def.validate().is_ok(),
            "toughness {toughness:?} should pass validation"
        );
    }
}

// Behavior 7: CellTypeDefinition deserializes from RON with toughness field
#[test]
fn definition_deserializes_with_toughness_weak() {
    let ron_str = r#"(
        id: "test",
        alias: "T",
        toughness: Weak,
        color_rgb: (1.0, 0.5, 0.2),
        required_to_clear: true,
        damage_hdr_base: 4.0,
        damage_green_min: 0.2,
        damage_blue_range: 0.4,
        damage_blue_base: 0.2,
    )"#;
    let def: CellTypeDefinition =
        ron::de::from_str(ron_str).expect("should deserialize with toughness: Weak");
    assert_eq!(def.toughness, Toughness::Weak);
}

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
        | CellBehavior::SurvivalPermanent { .. } => {
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

// ── CellTypeDefinition deserialization ──────────────────────────

#[test]
fn definition_with_no_behaviors_field_deserializes_to_none() {
    let ron_str = r#"(
        id: "test",
        alias: "S",
        toughness: Weak,
        color_rgb: (1.0, 0.5, 0.2),
        required_to_clear: true,
        damage_hdr_base: 4.0,
        damage_green_min: 0.2,
        damage_blue_range: 0.4,
        damage_blue_base: 0.2,
    )"#;
    let def: CellTypeDefinition =
        ron::de::from_str(ron_str).expect("should deserialize without behaviors field");
    assert!(def.behaviors.is_none());
}

#[test]
fn definition_with_single_regen_behavior_deserializes() {
    let ron_str = r#"(
        id: "regen",
        alias: "R",
        toughness: Standard,
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
    assert_eq!(def.behaviors, Some(vec![CellBehavior::Regen { rate: 2.0 }]),);
}

// ── alias validation ────────────────────────────────────────────

#[test]
fn validate_rejects_empty_alias() {
    let mut def = valid_definition();
    def.alias = String::new();
    assert!(def.validate().is_err());
}

#[test]
fn validate_rejects_dot_alias() {
    let mut def = valid_definition();
    def.alias = ".".to_owned();
    assert!(def.validate().is_err());
}

#[test]
fn validate_accepts_valid_definition_without_behaviors() {
    let def = valid_definition();
    assert!(def.validate().is_ok());
}

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

// ── Part A: AttackPattern Enum ─────────────────────────────────────

use super::data::AttackPattern;

// Behavior 1: AttackPattern::StraightDown variant exists and is distinct
#[test]
fn attack_pattern_straight_down_is_distinct_from_spread() {
    let straight = AttackPattern::StraightDown;
    let spread = AttackPattern::Spread(1);
    assert_ne!(
        straight, spread,
        "StraightDown should not equal any Spread variant"
    );
}

// Behavior 1 edge: debug output
#[test]
fn attack_pattern_straight_down_debug_contains_name() {
    let debug_str = format!("{:?}", AttackPattern::StraightDown);
    assert!(
        debug_str.contains("StraightDown"),
        "debug should contain 'StraightDown', got: {debug_str}"
    );
}

// Behavior 2: AttackPattern::Spread(u32) carries a count
#[test]
fn attack_pattern_spread_carries_count() {
    let pattern = AttackPattern::Spread(3);
    match pattern {
        AttackPattern::Spread(n) => assert_eq!(n, 3, "inner value should be 3"),
        AttackPattern::StraightDown => panic!("expected Spread variant"),
    }
}

// Behavior 2 edge: minimum valid Spread(2)
#[test]
fn attack_pattern_spread_minimum_valid_count() {
    let pattern = AttackPattern::Spread(2);
    match pattern {
        AttackPattern::Spread(n) => assert_eq!(n, 2, "inner value should be 2"),
        AttackPattern::StraightDown => panic!("expected Spread variant"),
    }
}

// Behavior 3: AttackPattern is Clone, Debug, PartialEq
#[test]
fn attack_pattern_is_clone_debug_partial_eq() {
    let original = AttackPattern::Spread(4);
    let cloned = original;
    assert_eq!(original, cloned, "copy should equal original");
    let debug_str = format!("{original:?}");
    assert!(
        debug_str.contains("Spread"),
        "debug should contain 'Spread', got: {debug_str}"
    );
    assert!(
        debug_str.contains('4'),
        "debug should contain '4', got: {debug_str}"
    );
}

// Behavior 3 edge: StraightDown clone equals original
#[test]
fn attack_pattern_straight_down_clone_equals_original() {
    let original = AttackPattern::StraightDown;
    let cloned = original;
    assert_eq!(original, cloned, "StraightDown clone should equal original");
}

// Behavior 4: AttackPattern deserializes from RON
#[test]
fn attack_pattern_deserializes_straight_down_from_ron() {
    let result: AttackPattern =
        ron::de::from_str("StraightDown").expect("should deserialize StraightDown");
    assert_eq!(result, AttackPattern::StraightDown);
}

// Behavior 4 edge: Spread(3) deserializes
#[test]
fn attack_pattern_deserializes_spread_from_ron() {
    let result: AttackPattern =
        ron::de::from_str("Spread(3)").expect("should deserialize Spread(3)");
    assert_eq!(result, AttackPattern::Spread(3));
}

// Behavior 5: Invalid RON fails deserialization
#[test]
fn attack_pattern_invalid_ron_fails_deserialization() {
    let result: Result<AttackPattern, _> = ron::de::from_str("Shotgun");
    assert!(
        result.is_err(),
        "\"Shotgun\" should not deserialize as AttackPattern"
    );
}

// Behavior 5 edge: Spread without count fails
#[test]
fn attack_pattern_spread_without_count_fails_deserialization() {
    let result: Result<AttackPattern, _> = ron::de::from_str("Spread");
    assert!(
        result.is_err(),
        "\"Spread\" without count should not deserialize as AttackPattern"
    );
}

// ── Part B: CellBehavior::Survival and SurvivalPermanent Variants ──

// Behavior 6: CellBehavior::Survival variant carries pattern and timer_secs
#[test]
fn cell_behavior_survival_carries_pattern_and_timer_secs() {
    let behavior = CellBehavior::Survival {
        pattern:    AttackPattern::StraightDown,
        timer_secs: 10.0,
    };
    match behavior {
        CellBehavior::Survival {
            pattern,
            timer_secs,
        } => {
            assert_eq!(pattern, AttackPattern::StraightDown);
            assert!((timer_secs - 10.0).abs() < f32::EPSILON);
        }
        _ => panic!("expected Survival variant"),
    }
}

// Behavior 6 edge: Spread(3) with timer 0.5
#[test]
fn cell_behavior_survival_with_spread_and_small_timer() {
    let behavior = CellBehavior::Survival {
        pattern:    AttackPattern::Spread(3),
        timer_secs: 0.5,
    };
    match behavior {
        CellBehavior::Survival {
            pattern,
            timer_secs,
        } => {
            assert_eq!(pattern, AttackPattern::Spread(3));
            assert!((timer_secs - 0.5).abs() < f32::EPSILON);
        }
        _ => panic!("expected Survival variant"),
    }
}

// Behavior 7: CellBehavior::SurvivalPermanent variant carries pattern only
#[test]
fn cell_behavior_survival_permanent_carries_pattern() {
    let behavior = CellBehavior::SurvivalPermanent {
        pattern: AttackPattern::Spread(4),
    };
    match behavior {
        CellBehavior::SurvivalPermanent { pattern } => {
            assert_eq!(pattern, AttackPattern::Spread(4));
        }
        _ => panic!("expected SurvivalPermanent variant"),
    }
}

// Behavior 7 edge: StraightDown
#[test]
fn cell_behavior_survival_permanent_straight_down() {
    let behavior = CellBehavior::SurvivalPermanent {
        pattern: AttackPattern::StraightDown,
    };
    match behavior {
        CellBehavior::SurvivalPermanent { pattern } => {
            assert_eq!(pattern, AttackPattern::StraightDown);
        }
        _ => panic!("expected SurvivalPermanent variant"),
    }
}

// Behavior 8: CellBehavior Survival variants deserialize from RON
#[test]
fn cell_behavior_survival_deserializes_from_ron() {
    let ron_str = "Survival(pattern: StraightDown, timer_secs: 10.0)";
    let result: CellBehavior = ron::de::from_str(ron_str).expect("should deserialize");
    assert_eq!(
        result,
        CellBehavior::Survival {
            pattern:    AttackPattern::StraightDown,
            timer_secs: 10.0,
        }
    );
}

// Behavior 8 edge: SurvivalPermanent deserializes
#[test]
fn cell_behavior_survival_permanent_deserializes_from_ron() {
    let ron_str = "SurvivalPermanent(pattern: Spread(3))";
    let result: CellBehavior = ron::de::from_str(ron_str).expect("should deserialize");
    assert_eq!(
        result,
        CellBehavior::SurvivalPermanent {
            pattern: AttackPattern::Spread(3),
        }
    );
}

// Behavior 9: CellBehavior Survival is Clone + PartialEq
#[test]
fn cell_behavior_survival_is_clone_eq() {
    let behavior = CellBehavior::Survival {
        pattern:    AttackPattern::Spread(2),
        timer_secs: 5.0,
    };
    let cloned = behavior.clone();
    assert_eq!(behavior, cloned, "clone should equal original");
}

// Behavior 9 edge: different timer_secs not equal
#[test]
fn cell_behavior_survival_different_timer_not_equal() {
    let a = CellBehavior::Survival {
        pattern:    AttackPattern::StraightDown,
        timer_secs: 5.0,
    };
    let b = CellBehavior::Survival {
        pattern:    AttackPattern::StraightDown,
        timer_secs: 10.0,
    };
    assert_ne!(a, b, "different timer_secs should not be equal");
}

// ── Part C: Survival Validation ────────────────────────────────────

// Behavior 10: Survival valid StraightDown with positive finite timer passes
#[test]
fn validate_accepts_survival_straight_down_positive_timer() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Survival {
        pattern:    AttackPattern::StraightDown,
        timer_secs: 10.0,
    }]);
    assert!(def.validate().is_ok());
}

// Behavior 10 edge: MIN_POSITIVE timer passes
#[test]
fn validate_accepts_survival_min_positive_timer() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Survival {
        pattern:    AttackPattern::StraightDown,
        timer_secs: f32::MIN_POSITIVE,
    }]);
    assert!(def.validate().is_ok());
}

// Behavior 11: Survival valid Spread(3) with positive finite timer passes
#[test]
fn validate_accepts_survival_spread_3_positive_timer() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Survival {
        pattern:    AttackPattern::Spread(3),
        timer_secs: 5.0,
    }]);
    assert!(def.validate().is_ok());
}

// Behavior 11 edge: Spread(2) minimum valid
#[test]
fn validate_accepts_survival_spread_2_minimum_valid() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Survival {
        pattern:    AttackPattern::Spread(2),
        timer_secs: 5.0,
    }]);
    assert!(def.validate().is_ok());
}

// Behavior 12: Survival zero timer_secs rejected
#[test]
fn validate_rejects_survival_zero_timer_secs() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Survival {
        pattern:    AttackPattern::StraightDown,
        timer_secs: 0.0,
    }]);
    assert!(
        def.validate().is_err(),
        "zero timer_secs should be rejected"
    );
}

// Behavior 12 edge: -0.0 rejected
#[test]
fn validate_rejects_survival_negative_zero_timer_secs() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Survival {
        pattern:    AttackPattern::StraightDown,
        timer_secs: -0.0_f32,
    }]);
    assert!(
        def.validate().is_err(),
        "-0.0 timer_secs should be rejected"
    );
}

// Behavior 13: Survival negative timer_secs rejected
#[test]
fn validate_rejects_survival_negative_timer_secs() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Survival {
        pattern:    AttackPattern::StraightDown,
        timer_secs: -5.0,
    }]);
    assert!(def.validate().is_err());
}

// Behavior 14: Survival NaN timer_secs rejected
#[test]
fn validate_rejects_survival_nan_timer_secs() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Survival {
        pattern:    AttackPattern::StraightDown,
        timer_secs: f32::NAN,
    }]);
    assert!(def.validate().is_err());
}

// Behavior 15: Survival infinite timer_secs rejected
#[test]
fn validate_rejects_survival_infinite_timer_secs() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Survival {
        pattern:    AttackPattern::StraightDown,
        timer_secs: f32::INFINITY,
    }]);
    assert!(def.validate().is_err());
}

// Behavior 15 edge: NEG_INFINITY rejected
#[test]
fn validate_rejects_survival_neg_infinite_timer_secs() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Survival {
        pattern:    AttackPattern::StraightDown,
        timer_secs: f32::NEG_INFINITY,
    }]);
    assert!(def.validate().is_err());
}

// Behavior 16: Survival Spread(1) rejected
#[test]
fn validate_rejects_survival_spread_1() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Survival {
        pattern:    AttackPattern::Spread(1),
        timer_secs: 10.0,
    }]);
    let err = def.validate().expect_err("Spread(1) should be rejected");
    assert!(
        err.contains("Spread") || err.contains('2'),
        "error should mention Spread or minimum 2, got: {err}"
    );
}

// Behavior 16 edge: Spread(0) also rejected
#[test]
fn validate_rejects_survival_spread_0() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Survival {
        pattern:    AttackPattern::Spread(0),
        timer_secs: 10.0,
    }]);
    assert!(def.validate().is_err(), "Spread(0) should be rejected");
}

// Behavior 17: SurvivalPermanent valid StraightDown passes
#[test]
fn validate_accepts_survival_permanent_straight_down() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::SurvivalPermanent {
        pattern: AttackPattern::StraightDown,
    }]);
    assert!(def.validate().is_ok());
}

// Behavior 18: SurvivalPermanent valid Spread(4) passes
#[test]
fn validate_accepts_survival_permanent_spread_4() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::SurvivalPermanent {
        pattern: AttackPattern::Spread(4),
    }]);
    assert!(def.validate().is_ok());
}

// Behavior 18 edge: Spread(2) passes
#[test]
fn validate_accepts_survival_permanent_spread_2() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::SurvivalPermanent {
        pattern: AttackPattern::Spread(2),
    }]);
    assert!(def.validate().is_ok());
}

// Behavior 19: SurvivalPermanent Spread(1) rejected
#[test]
fn validate_rejects_survival_permanent_spread_1() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::SurvivalPermanent {
        pattern: AttackPattern::Spread(1),
    }]);
    let err = def.validate().expect_err("Spread(1) should be rejected");
    assert!(
        err.contains("Spread") || err.contains('2'),
        "error should mention Spread or minimum 2, got: {err}"
    );
}

// Behavior 19 edge: Spread(0) also rejected
#[test]
fn validate_rejects_survival_permanent_spread_0() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::SurvivalPermanent {
        pattern: AttackPattern::Spread(0),
    }]);
    assert!(def.validate().is_err(), "Spread(0) should be rejected");
}

// Behavior 20: Survival mixed with other valid behaviors passes
#[test]
fn validate_accepts_survival_mixed_with_regen() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![
        CellBehavior::Regen { rate: 2.0 },
        CellBehavior::Survival {
            pattern:    AttackPattern::StraightDown,
            timer_secs: 10.0,
        },
    ]);
    assert!(def.validate().is_ok());
}

// Behavior 21: Invalid Survival after valid behaviors still rejected
#[test]
fn validate_rejects_invalid_survival_after_valid_regen() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![
        CellBehavior::Regen { rate: 2.0 },
        CellBehavior::Survival {
            pattern:    AttackPattern::Spread(1),
            timer_secs: 10.0,
        },
    ]);
    assert!(def.validate().is_err());
}
