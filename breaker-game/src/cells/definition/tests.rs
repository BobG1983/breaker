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
        | CellBehavior::Sequence { .. } => {
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
