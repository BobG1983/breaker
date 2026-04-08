use super::data::*;

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
        effects: None,
    }
}

fn valid_guarded_behavior() -> GuardedBehavior {
    GuardedBehavior {
        guardian_hp: 10.0,
        guardian_color_rgb: [0.5, 0.8, 1.0],
        slide_speed: 30.0,
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

// ── CellBehavior enum tests ────────────────────────────────────

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

// ── Section A: CellBehavior::Guarded deserializes ──────────────

// Behavior 1: Guarded variant deserializes from RON
#[test]
fn cell_behavior_guarded_deserializes_from_ron() {
    let ron_str =
        "Guarded((guardian_hp: 10.0, guardian_color_rgb: (0.5, 0.8, 1.0), slide_speed: 30.0))";
    let result: CellBehavior = ron::de::from_str(ron_str).expect("should deserialize");
    assert_eq!(
        result,
        CellBehavior::Guarded(GuardedBehavior {
            guardian_hp: 10.0,
            guardian_color_rgb: [0.5, 0.8, 1.0],
            slide_speed: 30.0,
        })
    );
}

// Behavior 1 edge case: slide_speed: 0.0 (stationary guardians)
#[test]
fn cell_behavior_guarded_zero_slide_speed_deserializes() {
    let ron_str =
        "Guarded((guardian_hp: 10.0, guardian_color_rgb: (0.5, 0.8, 1.0), slide_speed: 0.0))";
    let result: CellBehavior = ron::de::from_str(ron_str).expect("should deserialize");
    assert_eq!(
        result,
        CellBehavior::Guarded(GuardedBehavior {
            guardian_hp: 10.0,
            guardian_color_rgb: [0.5, 0.8, 1.0],
            slide_speed: 0.0,
        })
    );
}

// Behavior 2: CellBehavior::Guarded is Clone + Debug + PartialEq
#[test]
fn cell_behavior_guarded_is_clone_debug_partial_eq() {
    let behavior = CellBehavior::Guarded(GuardedBehavior {
        guardian_hp: 10.0,
        guardian_color_rgb: [0.5, 0.8, 1.0],
        slide_speed: 30.0,
    });
    let cloned = behavior.clone();
    assert_eq!(behavior, cloned, "clone should equal original via ==");
    let debug_str = format!("{behavior:?}");
    assert!(
        debug_str.contains("Guarded"),
        "debug should contain 'Guarded', got: {debug_str}"
    );
    assert!(
        debug_str.contains("10.0") || debug_str.contains("10"),
        "debug should contain '10.0', got: {debug_str}"
    );
}

// Behavior 3: GuardedBehavior struct has required fields
#[test]
fn guarded_behavior_struct_has_required_fields() {
    let gb = GuardedBehavior {
        guardian_hp: 5.0,
        guardian_color_rgb: [1.0, 0.0, 0.0],
        slide_speed: 50.0,
    };
    assert!(
        (gb.guardian_hp - 5.0).abs() < f32::EPSILON,
        "guardian_hp should be 5.0"
    );
    assert!(
        (gb.guardian_color_rgb[0] - 1.0).abs() < f32::EPSILON
            && (gb.guardian_color_rgb[1] - 0.0).abs() < f32::EPSILON
            && (gb.guardian_color_rgb[2] - 0.0).abs() < f32::EPSILON,
        "guardian_color_rgb should be [1.0, 0.0, 0.0]"
    );
    assert!(
        (gb.slide_speed - 50.0).abs() < f32::EPSILON,
        "slide_speed should be 50.0"
    );
}

// ── Section B: GuardedBehavior Validation ──────────────────────

// Behavior 4: validate() accepts valid values
#[test]
fn guarded_behavior_validate_accepts_valid() {
    let gb = valid_guarded_behavior();
    assert!(
        gb.validate().is_ok(),
        "valid GuardedBehavior should pass validation: {:?}",
        gb.validate(),
    );
}

// Behavior 4 edge case: smallest positive hp
#[test]
fn guarded_behavior_validate_accepts_smallest_positive_hp() {
    let mut gb = valid_guarded_behavior();
    gb.guardian_hp = 0.001;
    assert!(
        gb.validate().is_ok(),
        "guardian_hp = 0.001 should pass: {:?}",
        gb.validate(),
    );
}

// Behavior 5: validate() rejects zero guardian_hp
#[test]
fn guarded_behavior_validate_rejects_zero_hp() {
    let mut gb = valid_guarded_behavior();
    gb.guardian_hp = 0.0;
    assert!(
        gb.validate().is_err(),
        "guardian_hp = 0.0 should be rejected"
    );
}

// Behavior 6: validate() rejects negative guardian_hp
#[test]
fn guarded_behavior_validate_rejects_negative_hp() {
    let mut gb = valid_guarded_behavior();
    gb.guardian_hp = -5.0;
    assert!(
        gb.validate().is_err(),
        "guardian_hp = -5.0 should be rejected"
    );
}

// Behavior 6 edge case: -0.001
#[test]
fn guarded_behavior_validate_rejects_tiny_negative_hp() {
    let mut gb = valid_guarded_behavior();
    gb.guardian_hp = -0.001;
    assert!(
        gb.validate().is_err(),
        "guardian_hp = -0.001 should be rejected"
    );
}

// Behavior 7: validate() rejects NaN guardian_hp
#[test]
fn guarded_behavior_validate_rejects_nan_hp() {
    let mut gb = valid_guarded_behavior();
    gb.guardian_hp = f32::NAN;
    assert!(
        gb.validate().is_err(),
        "guardian_hp = NaN should be rejected"
    );
}

// Behavior 8: validate() rejects infinite guardian_hp
#[test]
fn guarded_behavior_validate_rejects_infinite_hp() {
    let mut gb = valid_guarded_behavior();
    gb.guardian_hp = f32::INFINITY;
    assert!(
        gb.validate().is_err(),
        "guardian_hp = INFINITY should be rejected"
    );
}

// Behavior 8 edge case: NEG_INFINITY
#[test]
fn guarded_behavior_validate_rejects_neg_infinite_hp() {
    let mut gb = valid_guarded_behavior();
    gb.guardian_hp = f32::NEG_INFINITY;
    assert!(
        gb.validate().is_err(),
        "guardian_hp = NEG_INFINITY should be rejected"
    );
}

// Behavior 9: validate() accepts zero slide_speed
#[test]
fn guarded_behavior_validate_accepts_zero_slide_speed() {
    let mut gb = valid_guarded_behavior();
    gb.slide_speed = 0.0;
    assert!(
        gb.validate().is_ok(),
        "slide_speed = 0.0 should be accepted: {:?}",
        gb.validate(),
    );
}

// Behavior 10: validate() rejects negative slide_speed
#[test]
fn guarded_behavior_validate_rejects_negative_slide_speed() {
    let mut gb = valid_guarded_behavior();
    gb.slide_speed = -1.0;
    assert!(
        gb.validate().is_err(),
        "slide_speed = -1.0 should be rejected"
    );
}

// Behavior 10 edge case: -0.001
#[test]
fn guarded_behavior_validate_rejects_tiny_negative_slide_speed() {
    let mut gb = valid_guarded_behavior();
    gb.slide_speed = -0.001;
    assert!(
        gb.validate().is_err(),
        "slide_speed = -0.001 should be rejected"
    );
}

// Behavior 11: validate() rejects NaN slide_speed
#[test]
fn guarded_behavior_validate_rejects_nan_slide_speed() {
    let mut gb = valid_guarded_behavior();
    gb.slide_speed = f32::NAN;
    assert!(
        gb.validate().is_err(),
        "slide_speed = NaN should be rejected"
    );
}

// Behavior 12: validate() rejects infinite slide_speed
#[test]
fn guarded_behavior_validate_rejects_infinite_slide_speed() {
    let mut gb = valid_guarded_behavior();
    gb.slide_speed = f32::INFINITY;
    assert!(
        gb.validate().is_err(),
        "slide_speed = INFINITY should be rejected"
    );
}

// Behavior 12 edge case: NEG_INFINITY
#[test]
fn guarded_behavior_validate_rejects_neg_infinite_slide_speed() {
    let mut gb = valid_guarded_behavior();
    gb.slide_speed = f32::NEG_INFINITY;
    assert!(
        gb.validate().is_err(),
        "slide_speed = NEG_INFINITY should be rejected"
    );
}

// ── Section C: CellTypeDefinition Guarded Validation Delegation ──

// Behavior 13: validate() delegates to GuardedBehavior::validate()
#[test]
fn cell_definition_validate_delegates_to_guarded_validate() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Guarded(GuardedBehavior {
        guardian_hp: -1.0, // invalid
        guardian_color_rgb: [0.5, 0.8, 1.0],
        slide_speed: 30.0,
    })]);
    assert!(
        def.validate().is_err(),
        "CellTypeDefinition.validate should reject invalid GuardedBehavior"
    );
}

// Behavior 14: validate() accepts valid Guarded behavior
#[test]
fn cell_definition_validate_accepts_valid_guarded() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![CellBehavior::Guarded(valid_guarded_behavior())]);
    assert!(
        def.validate().is_ok(),
        "CellTypeDefinition with valid GuardedBehavior should pass: {:?}",
        def.validate(),
    );
}

// Behavior 14 edge case: both Regen and Guarded valid
#[test]
fn cell_definition_validate_accepts_regen_and_guarded() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![
        CellBehavior::Regen { rate: 2.0 },
        CellBehavior::Guarded(valid_guarded_behavior()),
    ]);
    assert!(
        def.validate().is_ok(),
        "definition with both Regen and valid Guarded should pass: {:?}",
        def.validate(),
    );
}

// Behavior 15: validate() rejects when any behavior is invalid (mixed vec)
#[test]
fn cell_definition_validate_rejects_mixed_vec_with_invalid_guarded() {
    let mut def = valid_definition();
    def.behaviors = Some(vec![
        CellBehavior::Regen { rate: 2.0 },
        CellBehavior::Guarded(GuardedBehavior {
            guardian_hp: 0.0, // invalid
            guardian_color_rgb: [0.5, 0.8, 1.0],
            slide_speed: 30.0,
        }),
    ]);
    assert!(
        def.validate().is_err(),
        "behaviors with one invalid Guarded entry should be rejected"
    );
}

// Behavior 16: CellTypeDefinition without shield field deserializes
#[test]
fn definition_without_shield_field_deserializes() {
    let ron_str = r#"(
        id: "test",
        alias: "T",
        hp: 10.0,
        color_rgb: (1.0, 0.5, 0.2),
        required_to_clear: true,
        damage_hdr_base: 4.0,
        damage_green_min: 0.2,
        damage_blue_range: 0.4,
        damage_blue_base: 0.2,
    )"#;
    let def: CellTypeDefinition =
        ron::de::from_str(ron_str).expect("should deserialize without shield field");
    assert_eq!(def.alias, "T");
}

// ── CellTypeDefinition deserialization ──────────────────────────

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

// ── validate() for behaviors ────────────────────────────────────

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

// ── alias validation ────────────────────────────────────────────

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

#[test]
fn validate_accepts_valid_definition_without_behaviors() {
    let def = valid_definition();
    assert!(
        def.validate().is_ok(),
        "valid definition with behaviors = None should pass: {:?}",
        def.validate(),
    );
}
