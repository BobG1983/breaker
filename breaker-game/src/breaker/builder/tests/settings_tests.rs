use bevy::math::curve::easing::EaseFunction;

use super::super::core::*;

// ── Behavior 54: MovementSettings can be constructed ──

#[test]
fn movement_settings_construction() {
    let settings = MovementSettings {
        max_speed: 500.0,
        acceleration: 3000.0,
        deceleration: 2500.0,
        decel_ease: EaseFunction::QuadraticIn,
        decel_ease_strength: 1.0,
    };
    assert!((settings.max_speed - 500.0).abs() < f32::EPSILON);
    assert!((settings.acceleration - 3000.0).abs() < f32::EPSILON);
    assert!((settings.deceleration - 2500.0).abs() < f32::EPSILON);
    assert!(matches!(settings.decel_ease, EaseFunction::QuadraticIn));
    assert!((settings.decel_ease_strength - 1.0).abs() < f32::EPSILON);
}

// ── Behavior 55: DashSettings can be constructed with all sub-structs ──

#[test]
fn dash_settings_construction() {
    let settings = DashSettings {
        dash: DashParams {
            speed_multiplier: 4.0,
            duration: 0.15,
            tilt_angle: 15.0,
            tilt_ease: EaseFunction::QuadraticInOut,
        },
        brake: BrakeParams {
            tilt_angle: 25.0,
            tilt_duration: 0.2,
            tilt_ease: EaseFunction::CubicInOut,
            decel_multiplier: 2.0,
        },
        settle: SettleParams {
            duration: 0.25,
            tilt_ease: EaseFunction::CubicOut,
        },
    };
    assert!((settings.dash.speed_multiplier - 4.0).abs() < f32::EPSILON);
    assert!((settings.dash.duration - 0.15).abs() < f32::EPSILON);
    assert!((settings.dash.tilt_angle - 15.0).abs() < f32::EPSILON);
    assert!(matches!(
        settings.dash.tilt_ease,
        EaseFunction::QuadraticInOut
    ));
    assert!((settings.brake.tilt_angle - 25.0).abs() < f32::EPSILON);
    assert!((settings.brake.tilt_duration - 0.2).abs() < f32::EPSILON);
    assert!(matches!(settings.brake.tilt_ease, EaseFunction::CubicInOut));
    assert!((settings.brake.decel_multiplier - 2.0).abs() < f32::EPSILON);
    assert!((settings.settle.duration - 0.25).abs() < f32::EPSILON);
    assert!(matches!(settings.settle.tilt_ease, EaseFunction::CubicOut));
}

// ── Behavior 56: BumpSettings can be constructed ──

#[test]
fn bump_settings_construction() {
    let settings = BumpSettings {
        perfect_window: 0.15,
        early_window: 0.15,
        late_window: 0.15,
        perfect_cooldown: 0.0,
        weak_cooldown: 0.15,
        feedback: BumpFeedbackSettings {
            duration: 0.15,
            peak: 24.0,
            peak_fraction: 0.3,
            rise_ease: EaseFunction::CubicOut,
            fall_ease: EaseFunction::QuadraticIn,
        },
    };
    assert!((settings.perfect_window - 0.15).abs() < f32::EPSILON);
    assert!((settings.early_window - 0.15).abs() < f32::EPSILON);
    assert!((settings.late_window - 0.15).abs() < f32::EPSILON);
    assert!((settings.perfect_cooldown - 0.0).abs() < f32::EPSILON);
    assert!((settings.weak_cooldown - 0.15).abs() < f32::EPSILON);
    assert!((settings.feedback.duration - 0.15).abs() < f32::EPSILON);
    assert!((settings.feedback.peak - 24.0).abs() < f32::EPSILON);
    assert!((settings.feedback.peak_fraction - 0.3).abs() < f32::EPSILON);
    assert!(matches!(
        settings.feedback.rise_ease,
        EaseFunction::CubicOut
    ));
    assert!(matches!(
        settings.feedback.fall_ease,
        EaseFunction::QuadraticIn
    ));
}
