use bevy::math::curve::easing::EaseFunction;

use crate::breaker::definition::BreakerDefinition;

// ── Behavior 44: BreakerDefinition with all-default fields parses from minimal RON ──

#[test]
fn minimal_ron_parses_with_all_defaults() {
    let ron_str = r#"(name: "TestBreaker", effects: [])"#;
    let def: BreakerDefinition = ron::de::from_str(ron_str).expect("minimal RON should parse");

    assert_eq!(def.name, "TestBreaker");
    assert_eq!(def.bolt, "Bolt");
    assert_eq!(def.life_pool, None);
    assert!(def.effects.is_empty());

    // Dimensions
    assert!((def.width - 120.0).abs() < f32::EPSILON);
    assert!((def.height - 20.0).abs() < f32::EPSILON);
    assert!((def.y_position - (-250.0)).abs() < f32::EPSILON);
    assert!(def.min_w.is_none());
    assert!(def.max_w.is_none());
    assert!(def.min_h.is_none());
    assert!(def.max_h.is_none());

    // Movement
    assert!((def.max_speed - 500.0).abs() < f32::EPSILON);
    assert!((def.acceleration - 3000.0).abs() < f32::EPSILON);
    assert!((def.deceleration - 2500.0).abs() < f32::EPSILON);
    assert!(matches!(def.decel_ease, EaseFunction::QuadraticIn));
    assert!((def.decel_ease_strength - 1.0).abs() < f32::EPSILON);

    // Dash
    assert!((def.dash_speed_multiplier - 4.0).abs() < f32::EPSILON);
    assert!((def.dash_duration - 0.15).abs() < f32::EPSILON);
    assert!((def.dash_tilt_angle - 15.0).abs() < f32::EPSILON);
    assert!(matches!(def.dash_tilt_ease, EaseFunction::QuadraticInOut));
    assert!((def.brake_tilt_angle - 25.0).abs() < f32::EPSILON);
    assert!((def.brake_tilt_duration - 0.2).abs() < f32::EPSILON);
    assert!(matches!(def.brake_tilt_ease, EaseFunction::CubicInOut));
    assert!((def.brake_decel_multiplier - 2.0).abs() < f32::EPSILON);
    assert!((def.settle_duration - 0.25).abs() < f32::EPSILON);
    assert!(matches!(def.settle_tilt_ease, EaseFunction::CubicOut));

    // Bump
    assert!((def.perfect_window - 0.15).abs() < f32::EPSILON);
    assert!((def.early_window - 0.15).abs() < f32::EPSILON);
    assert!((def.late_window - 0.15).abs() < f32::EPSILON);
    assert!((def.perfect_bump_cooldown - 0.0).abs() < f32::EPSILON);
    assert!((def.weak_bump_cooldown - 0.15).abs() < f32::EPSILON);
    assert!((def.bump_visual_duration - 0.15).abs() < f32::EPSILON);
    assert!((def.bump_visual_peak - 24.0).abs() < f32::EPSILON);
    assert!((def.bump_visual_peak_fraction - 0.3).abs() < f32::EPSILON);
    assert!(matches!(def.bump_visual_rise_ease, EaseFunction::CubicOut));
    assert!(matches!(
        def.bump_visual_fall_ease,
        EaseFunction::QuadraticIn
    ));

    // Spread
    assert!((def.reflection_spread - 75.0).abs() < f32::EPSILON);

    // Visual
    assert!((def.color_rgb[0] - 0.2).abs() < f32::EPSILON);
    assert!((def.color_rgb[1] - 2.0).abs() < f32::EPSILON);
    assert!((def.color_rgb[2] - 3.0).abs() < f32::EPSILON);
}

// ── Behavior 45: BreakerDefinition with explicit gameplay fields parses ──

#[test]
fn ron_with_explicit_gameplay_fields_parses() {
    let ron_str = r#"(
        name: "Custom",
        width: 150.0,
        height: 25.0,
        max_speed: 600.0,
        reflection_spread: 60.0,
        effects: [],
    )"#;
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("RON with explicit fields should parse");

    assert!((def.width - 150.0).abs() < f32::EPSILON);
    assert!((def.height - 25.0).abs() < f32::EPSILON);
    assert!((def.max_speed - 600.0).abs() < f32::EPSILON);
    assert!((def.reflection_spread - 60.0).abs() < f32::EPSILON);
    // Other fields should be defaults
    assert!((def.acceleration - 3000.0).abs() < f32::EPSILON);
    assert!((def.deceleration - 2500.0).abs() < f32::EPSILON);
}

// ── Behavior 46: BreakerDefinition with explicit min/max size fields parses ──

#[test]
fn ron_with_explicit_min_max_size_parses() {
    let ron_str = r#"(
        name: "Sized",
        min_w: Some(80.0),
        max_w: Some(200.0),
        min_h: None,
        max_h: Some(50.0),
        effects: [],
    )"#;
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("RON with min/max size should parse");

    assert_eq!(def.min_w, Some(80.0));
    assert_eq!(def.max_w, Some(200.0));
    assert_eq!(def.min_h, None);
    assert_eq!(def.max_h, Some(50.0));
}

// ── Behavior 47: aegis.breaker.ron parses with expanded BreakerDefinition ──

#[test]
fn aegis_breaker_ron_parses_with_expanded_definition() {
    let ron_str = include_str!("../../../../assets/breakers/aegis.breaker.ron");
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("aegis.breaker.ron should parse");

    assert_eq!(def.name, "Aegis");
    assert_eq!(def.bolt, "Bolt");
    assert_eq!(def.life_pool, Some(3));
    assert_eq!(def.effects.len(), 4);
    // All gameplay fields should be defaults
    assert!((def.width - 120.0).abs() < f32::EPSILON);
    assert!((def.max_speed - 500.0).abs() < f32::EPSILON);
}

// ── Behavior 48: chrono.breaker.ron parses with expanded BreakerDefinition ──

#[test]
fn chrono_breaker_ron_parses_with_expanded_definition() {
    let ron_str = include_str!("../../../../assets/breakers/chrono.breaker.ron");
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("chrono.breaker.ron should parse");

    assert_eq!(def.name, "Chrono");
    assert_eq!(def.bolt, "Bolt");
    assert_eq!(def.life_pool, None);
    assert!(!def.effects.is_empty());
}

// ── Behavior 49: prism.breaker.ron parses with expanded BreakerDefinition ──

#[test]
fn prism_breaker_ron_parses_with_expanded_definition() {
    let ron_str = include_str!("../../../../assets/breakers/prism.breaker.ron");
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("prism.breaker.ron should parse");

    assert_eq!(def.name, "Prism");
    assert_eq!(def.bolt, "Bolt");
    assert_eq!(def.life_pool, None);
    assert!(!def.effects.is_empty());
}

// ── Behavior 50: BreakerDefinition no longer has stat_overrides field ──

#[test]
fn old_format_with_stat_overrides_fails_to_parse() {
    let ron_str = r#"(name: "Test", stat_overrides: (), effects: [])"#;
    let result = ron::de::from_str::<BreakerDefinition>(ron_str);
    assert!(
        result.is_err(),
        "RON with stat_overrides should fail to parse (field no longer exists)"
    );
}
