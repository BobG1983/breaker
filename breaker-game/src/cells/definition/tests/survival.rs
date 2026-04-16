use super::{super::data::*, helpers::valid_definition};

// ── Part A: AttackPattern Enum ─────────────────────────────────────

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
