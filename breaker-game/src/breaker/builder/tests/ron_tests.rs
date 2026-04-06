use crate::breaker::definition::BreakerDefinition;

// ── Behavior 44: BreakerDefinition with all-default fields parses from minimal RON ──

#[test]
fn minimal_ron_parses_with_all_defaults() {
    let ron_str = r#"(name: "TestBreaker", effects: [])"#;
    let def: BreakerDefinition = ron::de::from_str(ron_str).expect("minimal RON should parse");
    let defaults = BreakerDefinition::default();

    assert_eq!(def.name, "TestBreaker");
    assert_eq!(def.bolt, "Bolt");
    assert_eq!(def.life_pool, None);
    assert!(def.effects.is_empty());

    // Dimensions
    assert!((def.width - defaults.width).abs() < f32::EPSILON);
    assert!((def.height - defaults.height).abs() < f32::EPSILON);
    assert!((def.y_position - defaults.y_position).abs() < f32::EPSILON);
    assert!(def.min_w.is_none());
    assert!(def.max_w.is_none());
    assert!(def.min_h.is_none());
    assert!(def.max_h.is_none());

    // Movement
    assert!((def.max_speed - defaults.max_speed).abs() < f32::EPSILON);
    assert!((def.acceleration - defaults.acceleration).abs() < f32::EPSILON);
    assert!((def.deceleration - defaults.deceleration).abs() < f32::EPSILON);
    assert_eq!(def.decel_ease, defaults.decel_ease);
    assert!((def.decel_ease_strength - defaults.decel_ease_strength).abs() < f32::EPSILON);

    // Dash
    assert!((def.dash_speed_multiplier - defaults.dash_speed_multiplier).abs() < f32::EPSILON);
    assert!((def.dash_duration - defaults.dash_duration).abs() < f32::EPSILON);
    assert!((def.dash_tilt_angle - defaults.dash_tilt_angle).abs() < f32::EPSILON);
    assert_eq!(def.dash_tilt_ease, defaults.dash_tilt_ease);
    assert!((def.brake_tilt_angle - defaults.brake_tilt_angle).abs() < f32::EPSILON);
    assert!((def.brake_tilt_duration - defaults.brake_tilt_duration).abs() < f32::EPSILON);
    assert_eq!(def.brake_tilt_ease, defaults.brake_tilt_ease);
    assert!((def.brake_decel_multiplier - defaults.brake_decel_multiplier).abs() < f32::EPSILON);
    assert!((def.settle_duration - defaults.settle_duration).abs() < f32::EPSILON);
    assert_eq!(def.settle_tilt_ease, defaults.settle_tilt_ease);

    // Bump
    assert!((def.perfect_window - defaults.perfect_window).abs() < f32::EPSILON);
    assert!((def.early_window - defaults.early_window).abs() < f32::EPSILON);
    assert!((def.late_window - defaults.late_window).abs() < f32::EPSILON);
    assert!((def.perfect_bump_cooldown - defaults.perfect_bump_cooldown).abs() < f32::EPSILON);
    assert!((def.weak_bump_cooldown - defaults.weak_bump_cooldown).abs() < f32::EPSILON);
    assert!((def.bump_visual_duration - defaults.bump_visual_duration).abs() < f32::EPSILON);
    assert!((def.bump_visual_peak - defaults.bump_visual_peak).abs() < f32::EPSILON);
    assert!(
        (def.bump_visual_peak_fraction - defaults.bump_visual_peak_fraction).abs() < f32::EPSILON
    );
    assert_eq!(def.bump_visual_rise_ease, defaults.bump_visual_rise_ease);
    assert_eq!(def.bump_visual_fall_ease, defaults.bump_visual_fall_ease);

    // Spread
    assert!((def.reflection_spread - defaults.reflection_spread).abs() < f32::EPSILON);

    // Visual
    assert!((def.color_rgb[0] - defaults.color_rgb[0]).abs() < f32::EPSILON);
    assert!((def.color_rgb[1] - defaults.color_rgb[1]).abs() < f32::EPSILON);
    assert!((def.color_rgb[2] - defaults.color_rgb[2]).abs() < f32::EPSILON);
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

    let defaults = BreakerDefinition::default();
    assert!((def.width - 150.0).abs() < f32::EPSILON);
    assert!((def.height - 25.0).abs() < f32::EPSILON);
    assert!((def.max_speed - 600.0).abs() < f32::EPSILON);
    assert!((def.reflection_spread - 60.0).abs() < f32::EPSILON);
    // Other fields should be defaults
    assert!((def.acceleration - defaults.acceleration).abs() < f32::EPSILON);
    assert!((def.deceleration - defaults.deceleration).abs() < f32::EPSILON);
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

    let defaults = BreakerDefinition::default();
    assert_eq!(def.name, "Aegis");
    assert_eq!(def.bolt, "Bolt");
    assert_eq!(def.life_pool, Some(3));
    assert_eq!(def.effects.len(), 4);
    // All gameplay fields should be defaults
    assert!((def.width - defaults.width).abs() < f32::EPSILON);
    assert!((def.max_speed - defaults.max_speed).abs() < f32::EPSILON);
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
