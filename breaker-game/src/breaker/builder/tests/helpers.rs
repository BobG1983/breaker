use bevy::math::curve::easing::EaseFunction;

use crate::breaker::definition::BreakerDefinition;

/// Creates a `BreakerDefinition` matching `BreakerDefinition::default()` values,
/// so existing component assertions remain valid.
pub(super) fn test_breaker_definition() -> BreakerDefinition {
    ron::de::from_str(
        r#"(
            name: "TestBreaker",
            effects: [],
        )"#,
    )
    .expect("test RON should parse")
}

/// Creates a `BreakerDefinition` with custom gameplay values for testing
/// specific field propagation.
pub(super) fn custom_breaker_definition() -> BreakerDefinition {
    BreakerDefinition {
        name: "CustomBreaker".to_string(),
        bolt: "Bolt".to_string(),
        width: 150.0,
        height: 25.0,
        y_position: -300.0,
        max_speed: 600.0,
        acceleration: 4000.0,
        deceleration: 3000.0,
        decel_ease: EaseFunction::CubicIn,
        decel_ease_strength: 2.0,
        dash_speed_multiplier: 3.0,
        dash_duration: 0.2,
        dash_tilt_angle: 20.0,
        dash_tilt_ease: EaseFunction::CubicInOut,
        brake_tilt_angle: 30.0,
        brake_tilt_duration: 0.3,
        brake_tilt_ease: EaseFunction::QuadraticIn,
        brake_decel_multiplier: 3.0,
        settle_duration: 0.3,
        settle_tilt_ease: EaseFunction::QuadraticOut,
        reflection_spread: 60.0,
        perfect_window: 0.2,
        early_window: 0.1,
        late_window: 0.1,
        perfect_bump_cooldown: 0.05,
        weak_bump_cooldown: 0.2,
        bump_visual_duration: 0.2,
        bump_visual_peak: 30.0,
        bump_visual_peak_fraction: 0.4,
        bump_visual_rise_ease: EaseFunction::CubicOut,
        bump_visual_fall_ease: EaseFunction::QuadraticIn,
        color_rgb: [0.2, 2.0, 3.0],
        life_pool: None,
        effects: vec![],
        min_w: None,
        max_w: None,
        min_h: None,
        max_h: None,
    }
}
