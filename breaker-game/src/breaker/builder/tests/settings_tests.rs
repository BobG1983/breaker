use super::super::core::*;
use crate::breaker::definition::BreakerDefinition;

// ── Behavior 54: MovementSettings can be constructed ──

#[test]
fn movement_settings_construction() {
    let defaults = BreakerDefinition::default();
    let settings = MovementSettings {
        max_speed: defaults.max_speed,
        acceleration: defaults.acceleration,
        deceleration: defaults.deceleration,
        decel_ease: defaults.decel_ease,
        decel_ease_strength: defaults.decel_ease_strength,
    };
    assert!((settings.max_speed - defaults.max_speed).abs() < f32::EPSILON);
    assert!((settings.acceleration - defaults.acceleration).abs() < f32::EPSILON);
    assert!((settings.deceleration - defaults.deceleration).abs() < f32::EPSILON);
    assert_eq!(settings.decel_ease, defaults.decel_ease);
    assert!((settings.decel_ease_strength - defaults.decel_ease_strength).abs() < f32::EPSILON);
}

// ── Behavior 55: DashSettings can be constructed with all sub-structs ──

#[test]
fn dash_settings_construction() {
    let defaults = BreakerDefinition::default();
    let settings = DashSettings {
        dash: DashParams {
            speed_multiplier: defaults.dash_speed_multiplier,
            duration: defaults.dash_duration,
            tilt_angle: defaults.dash_tilt_angle,
            tilt_ease: defaults.dash_tilt_ease,
        },
        brake: BrakeParams {
            tilt_angle: defaults.brake_tilt_angle,
            tilt_duration: defaults.brake_tilt_duration,
            tilt_ease: defaults.brake_tilt_ease,
            decel_multiplier: defaults.brake_decel_multiplier,
        },
        settle: SettleParams {
            duration: defaults.settle_duration,
            tilt_ease: defaults.settle_tilt_ease,
        },
    };
    assert!((settings.dash.speed_multiplier - defaults.dash_speed_multiplier).abs() < f32::EPSILON);
    assert!((settings.dash.duration - defaults.dash_duration).abs() < f32::EPSILON);
    assert!((settings.dash.tilt_angle - defaults.dash_tilt_angle).abs() < f32::EPSILON);
    assert_eq!(settings.dash.tilt_ease, defaults.dash_tilt_ease);
    assert!((settings.brake.tilt_angle - defaults.brake_tilt_angle).abs() < f32::EPSILON);
    assert!((settings.brake.tilt_duration - defaults.brake_tilt_duration).abs() < f32::EPSILON);
    assert_eq!(settings.brake.tilt_ease, defaults.brake_tilt_ease);
    assert!((settings.brake.decel_multiplier - defaults.brake_decel_multiplier).abs() < f32::EPSILON);
    assert!((settings.settle.duration - defaults.settle_duration).abs() < f32::EPSILON);
    assert_eq!(settings.settle.tilt_ease, defaults.settle_tilt_ease);
}

// ── Behavior 56: BumpSettings can be constructed ──

#[test]
fn bump_settings_construction() {
    let defaults = BreakerDefinition::default();
    let settings = BumpSettings {
        perfect_window: defaults.perfect_window,
        early_window: defaults.early_window,
        late_window: defaults.late_window,
        perfect_cooldown: defaults.perfect_bump_cooldown,
        weak_cooldown: defaults.weak_bump_cooldown,
        feedback: BumpFeedbackSettings {
            duration: defaults.bump_visual_duration,
            peak: defaults.bump_visual_peak,
            peak_fraction: defaults.bump_visual_peak_fraction,
            rise_ease: defaults.bump_visual_rise_ease,
            fall_ease: defaults.bump_visual_fall_ease,
        },
    };
    assert!((settings.perfect_window - defaults.perfect_window).abs() < f32::EPSILON);
    assert!((settings.early_window - defaults.early_window).abs() < f32::EPSILON);
    assert!((settings.late_window - defaults.late_window).abs() < f32::EPSILON);
    assert!((settings.perfect_cooldown - defaults.perfect_bump_cooldown).abs() < f32::EPSILON);
    assert!((settings.weak_cooldown - defaults.weak_bump_cooldown).abs() < f32::EPSILON);
    assert!((settings.feedback.duration - defaults.bump_visual_duration).abs() < f32::EPSILON);
    assert!((settings.feedback.peak - defaults.bump_visual_peak).abs() < f32::EPSILON);
    assert!((settings.feedback.peak_fraction - defaults.bump_visual_peak_fraction).abs() < f32::EPSILON);
    assert_eq!(settings.feedback.rise_ease, defaults.bump_visual_rise_ease);
    assert_eq!(settings.feedback.fall_ease, defaults.bump_visual_fall_ease);
}
