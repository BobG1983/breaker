//! System to initialize breaker entity components from config.

use bevy::prelude::*;

use crate::breaker::{
    components::{
        BrakeDecel, BrakeTilt, Breaker, BreakerAcceleration, BreakerBaseY, BreakerDeceleration,
        BreakerHeight, BreakerMaxSpeed, BreakerWidth, BumpEarlyWindow, BumpLateWindow,
        BumpPerfectCooldown, BumpPerfectMultiplier, BumpPerfectWindow, BumpVisualParams,
        BumpWeakCooldown, BumpWeakMultiplier, DashDuration, DashSpeedMultiplier, DashTilt,
        DashTiltEase, DecelEasing, MaxReflectionAngle, MinAngleFromHorizontal, SettleDuration,
        SettleTiltEase,
    },
    resources::BreakerConfig,
};

/// Materializes config values as components on the breaker entity.
///
/// Runs `OnEnter(GameState::Playing)` after `spawn_breaker`. Uses
/// `Without<BreakerMaxSpeed>` to skip already-initialized breakers
/// (persisted across nodes).
pub fn init_breaker_params(
    mut commands: Commands,
    config: Res<BreakerConfig>,
    query: Query<Entity, (With<Breaker>, Without<BreakerMaxSpeed>)>,
) {
    for entity in &query {
        commands
            .entity(entity)
            .insert((
                BreakerWidth(config.width),
                BreakerHeight(config.height),
                BreakerBaseY(config.y_position),
                BreakerMaxSpeed(config.max_speed),
                BreakerAcceleration(config.acceleration),
                BreakerDeceleration(config.deceleration),
                DecelEasing {
                    ease: config.decel_ease,
                    strength: config.decel_ease_strength,
                },
                DashSpeedMultiplier(config.dash_speed_multiplier),
                DashDuration(config.dash_duration),
                DashTilt(config.dash_tilt_angle.to_radians()),
                DashTiltEase(config.dash_tilt_ease),
                BrakeTilt {
                    angle: config.brake_tilt_angle.to_radians(),
                    duration: config.brake_tilt_duration,
                    ease: config.brake_tilt_ease,
                },
                BrakeDecel(config.brake_decel_multiplier),
                MaxReflectionAngle(config.max_reflection_angle.to_radians()),
                MinAngleFromHorizontal(config.min_angle_from_horizontal.to_radians()),
            ))
            .insert((
                SettleDuration(config.settle_duration),
                SettleTiltEase(config.settle_tilt_ease),
                BumpPerfectWindow(config.perfect_window),
                BumpEarlyWindow(config.early_window),
                BumpLateWindow(config.late_window),
                BumpPerfectCooldown(config.perfect_bump_cooldown),
                BumpWeakCooldown(config.weak_bump_cooldown),
                BumpVisualParams {
                    duration: config.bump_visual_duration,
                    peak: config.bump_visual_peak,
                    peak_fraction: config.bump_visual_peak_fraction,
                    rise_ease: config.bump_visual_rise_ease,
                    fall_ease: config.bump_visual_fall_ease,
                },
            ))
            // Default bump multipliers — identity (1.0) unless overridden by
            // init_archetype's apply_bolt_speed_boosts. Inserted separately so
            // archetype-stamped values take precedence via last-write-wins.
            .insert_if_new((BumpPerfectMultiplier(1.0), BumpWeakMultiplier(1.0)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::breaker::components::{
        BreakerState, BreakerVelocity, BumpPerfectMultiplier, BumpState, BumpWeakMultiplier,
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BreakerConfig>();
        app.add_systems(Update, init_breaker_params);
        app
    }

    #[test]
    fn init_inserts_all_components() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerState::default(),
                BreakerVelocity::default(),
                BumpState::default(),
            ))
            .id();

        app.update();

        let world = app.world();
        assert!(world.get::<BreakerWidth>(entity).is_some());
        assert!(world.get::<BreakerHeight>(entity).is_some());
        assert!(world.get::<BreakerBaseY>(entity).is_some());
        assert!(world.get::<BreakerMaxSpeed>(entity).is_some());
        assert!(world.get::<BreakerAcceleration>(entity).is_some());
        assert!(world.get::<BreakerDeceleration>(entity).is_some());
        assert!(world.get::<DecelEasing>(entity).is_some());
        assert!(world.get::<DashSpeedMultiplier>(entity).is_some());
        assert!(world.get::<DashDuration>(entity).is_some());
        assert!(world.get::<DashTilt>(entity).is_some());
        assert!(world.get::<DashTiltEase>(entity).is_some());
        assert!(world.get::<BrakeTilt>(entity).is_some());
        assert!(world.get::<BrakeDecel>(entity).is_some());
        assert!(world.get::<MaxReflectionAngle>(entity).is_some());
        assert!(world.get::<MinAngleFromHorizontal>(entity).is_some());
        assert!(world.get::<SettleDuration>(entity).is_some());
        assert!(world.get::<SettleTiltEase>(entity).is_some());
        assert!(world.get::<BumpPerfectWindow>(entity).is_some());
        assert!(world.get::<BumpEarlyWindow>(entity).is_some());
        assert!(world.get::<BumpLateWindow>(entity).is_some());
        assert!(world.get::<BumpPerfectCooldown>(entity).is_some());
        assert!(world.get::<BumpWeakCooldown>(entity).is_some());
        assert!(world.get::<BumpVisualParams>(entity).is_some());
        assert!(world.get::<BumpPerfectMultiplier>(entity).is_some());
        assert!(world.get::<BumpWeakMultiplier>(entity).is_some());
    }

    #[test]
    fn init_values_match_config() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerState::default(),
                BreakerVelocity::default(),
                BumpState::default(),
            ))
            .id();

        app.update();

        let config = app.world().resource::<BreakerConfig>();
        let world = app.world();
        assert!(
            (world.get::<BreakerMaxSpeed>(entity).unwrap().0 - config.max_speed).abs()
                < f32::EPSILON
        );
        assert!(
            (world.get::<BreakerBaseY>(entity).unwrap().0 - config.y_position).abs() < f32::EPSILON
        );
        assert!(
            (world.get::<BumpPerfectWindow>(entity).unwrap().0 - config.perfect_window).abs()
                < f32::EPSILON
        );
        assert!(
            (world.get::<BreakerWidth>(entity).unwrap().0 - config.width).abs() < f32::EPSILON,
            "BreakerWidth should match config.width"
        );
        assert!(
            (world.get::<BreakerHeight>(entity).unwrap().0 - config.height).abs() < f32::EPSILON,
            "BreakerHeight should match config.height"
        );
        assert!(
            (world.get::<MaxReflectionAngle>(entity).unwrap().0
                - config.max_reflection_angle.to_radians())
            .abs()
                < 1e-5,
            "MaxReflectionAngle should match config (converted to radians)"
        );
        assert!(
            (world.get::<MinAngleFromHorizontal>(entity).unwrap().0
                - config.min_angle_from_horizontal.to_radians())
            .abs()
                < 1e-5,
            "MinAngleFromHorizontal should match config (converted to radians)"
        );
        let params = world.get::<BumpVisualParams>(entity).unwrap();
        assert!(
            (params.duration - config.bump_visual_duration).abs() < f32::EPSILON,
            "BumpVisualParams.duration should match config"
        );
        assert!(
            (params.peak - config.bump_visual_peak).abs() < f32::EPSILON,
            "BumpVisualParams.peak should match config"
        );
    }

    #[test]
    fn default_bump_multipliers_are_identity() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerState::default(),
                BreakerVelocity::default(),
                BumpState::default(),
            ))
            .id();

        app.update();

        let perfect = app.world().get::<BumpPerfectMultiplier>(entity).unwrap();
        assert!(
            (perfect.0 - 1.0).abs() < f32::EPSILON,
            "default perfect multiplier should be 1.0"
        );
        let weak = app.world().get::<BumpWeakMultiplier>(entity).unwrap();
        assert!(
            (weak.0 - 1.0).abs() < f32::EPSILON,
            "default weak multiplier should be 1.0"
        );
    }

    #[test]
    fn archetype_multipliers_not_overwritten() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerState::default(),
                BreakerVelocity::default(),
                BumpState::default(),
                BumpPerfectMultiplier(1.5),
                BumpWeakMultiplier(0.8),
            ))
            .id();

        app.update();

        let perfect = app.world().get::<BumpPerfectMultiplier>(entity).unwrap();
        assert!(
            (perfect.0 - 1.5).abs() < f32::EPSILON,
            "archetype-stamped perfect multiplier should be preserved"
        );
        let weak = app.world().get::<BumpWeakMultiplier>(entity).unwrap();
        assert!(
            (weak.0 - 0.8).abs() < f32::EPSILON,
            "archetype-stamped weak multiplier should be preserved"
        );
    }

    #[test]
    fn skips_already_initialized() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerState::default(),
                BreakerVelocity::default(),
                BumpState::default(),
                BreakerMaxSpeed(999.0),
            ))
            .id();

        app.update();

        // Should not overwrite existing value
        let max_speed = app.world().get::<BreakerMaxSpeed>(entity).unwrap();
        assert!((max_speed.0 - 999.0).abs() < f32::EPSILON);
    }
}
