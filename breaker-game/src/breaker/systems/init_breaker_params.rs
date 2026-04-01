//! System to initialize breaker entity components from config.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::MaxSpeed;

use crate::breaker::{
    components::{
        BaseHeight, BaseWidth, BrakeDecel, BrakeTilt, Breaker, BreakerAcceleration, BreakerBaseY,
        BreakerDeceleration, BreakerReflectionSpread, BumpEarlyWindow, BumpFeedback,
        BumpLateWindow, BumpPerfectCooldown, BumpPerfectWindow, BumpWeakCooldown, DashDuration,
        DashSpeedMultiplier, DashTilt, DashTiltEase, DecelEasing, SettleDuration, SettleTiltEase,
    },
    resources::BreakerConfig,
};

/// Materializes config values as components on the breaker entity.
///
/// Runs `OnEnter(GameState::Playing)` after `spawn_breaker`. Uses
/// `Without<MaxSpeed>` to skip already-initialized breakers
/// (persisted across nodes).
pub fn init_breaker_params(
    mut commands: Commands,
    config: Res<BreakerConfig>,
    query: Query<Entity, (With<Breaker>, Without<MaxSpeed>)>,
) {
    for entity in &query {
        commands
            .entity(entity)
            .insert((
                BaseWidth(config.width),
                BaseHeight(config.height),
                BreakerBaseY(config.y_position),
                MaxSpeed(config.max_speed),
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
                BreakerReflectionSpread(config.reflection_spread.to_radians()),
            ))
            .insert((
                SettleDuration(config.settle_duration),
                SettleTiltEase(config.settle_tilt_ease),
                BumpPerfectWindow(config.perfect_window),
                BumpEarlyWindow(config.early_window),
                BumpLateWindow(config.late_window),
                BumpPerfectCooldown(config.perfect_bump_cooldown),
                BumpWeakCooldown(config.weak_bump_cooldown),
                BumpFeedback {
                    duration: config.bump_visual_duration,
                    peak: config.bump_visual_peak,
                    peak_fraction: config.bump_visual_peak_fraction,
                    rise_ease: config.bump_visual_rise_ease,
                    fall_ease: config.bump_visual_fall_ease,
                },
            ));
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_spatial2d::components::Velocity2D;

    use super::*;
    use crate::breaker::components::{BumpState, DashState};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<BreakerConfig>()
            .add_systems(Update, init_breaker_params);
        app
    }

    #[test]
    fn init_inserts_all_components() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                DashState::default(),
                Velocity2D::default(),
                BumpState::default(),
            ))
            .id();

        app.update();

        let world = app.world();
        assert!(world.get::<BaseWidth>(entity).is_some());
        assert!(world.get::<BaseHeight>(entity).is_some());
        assert!(world.get::<BreakerBaseY>(entity).is_some());
        assert!(world.get::<MaxSpeed>(entity).is_some());
        assert!(world.get::<BreakerAcceleration>(entity).is_some());
        assert!(world.get::<BreakerDeceleration>(entity).is_some());
        assert!(world.get::<DecelEasing>(entity).is_some());
        assert!(world.get::<DashSpeedMultiplier>(entity).is_some());
        assert!(world.get::<DashDuration>(entity).is_some());
        assert!(world.get::<DashTilt>(entity).is_some());
        assert!(world.get::<DashTiltEase>(entity).is_some());
        assert!(world.get::<BrakeTilt>(entity).is_some());
        assert!(world.get::<BrakeDecel>(entity).is_some());
        assert!(world.get::<BreakerReflectionSpread>(entity).is_some());
        assert!(world.get::<SettleDuration>(entity).is_some());
        assert!(world.get::<SettleTiltEase>(entity).is_some());
        assert!(world.get::<BumpPerfectWindow>(entity).is_some());
        assert!(world.get::<BumpEarlyWindow>(entity).is_some());
        assert!(world.get::<BumpLateWindow>(entity).is_some());
        assert!(world.get::<BumpPerfectCooldown>(entity).is_some());
        assert!(world.get::<BumpWeakCooldown>(entity).is_some());
        assert!(world.get::<BumpFeedback>(entity).is_some());
    }

    #[test]
    fn init_values_match_config() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                DashState::default(),
                Velocity2D::default(),
                BumpState::default(),
            ))
            .id();

        app.update();

        let config = app.world().resource::<BreakerConfig>();
        let world = app.world();
        assert!((world.get::<MaxSpeed>(entity).unwrap().0 - config.max_speed).abs() < f32::EPSILON);
        assert!(
            (world.get::<BreakerBaseY>(entity).unwrap().0 - config.y_position).abs() < f32::EPSILON
        );
        assert!(
            (world.get::<BumpPerfectWindow>(entity).unwrap().0 - config.perfect_window).abs()
                < f32::EPSILON
        );
        assert!(
            (world.get::<BaseWidth>(entity).unwrap().0 - config.width).abs() < f32::EPSILON,
            "BaseWidth should match config.width"
        );
        assert!(
            (world.get::<BaseHeight>(entity).unwrap().0 - config.height).abs() < f32::EPSILON,
            "BaseHeight should match config.height"
        );
        assert!(
            (world.get::<BreakerReflectionSpread>(entity).unwrap().0
                - config.reflection_spread.to_radians())
            .abs()
                < 1e-5,
            "BreakerReflectionSpread should match config (converted to radians)"
        );
        let params = world.get::<BumpFeedback>(entity).unwrap();
        assert!(
            (params.duration - config.bump_visual_duration).abs() < f32::EPSILON,
            "BumpFeedback.duration should match config"
        );
        assert!(
            (params.peak - config.bump_visual_peak).abs() < f32::EPSILON,
            "BumpFeedback.peak should match config"
        );
    }

    #[test]
    fn skips_already_initialized() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                DashState::default(),
                Velocity2D::default(),
                BumpState::default(),
                MaxSpeed(999.0),
            ))
            .id();

        app.update();

        // Should not overwrite existing value
        let max_speed = app.world().get::<MaxSpeed>(entity).unwrap();
        assert!((max_speed.0 - 999.0).abs() < f32::EPSILON);
    }
}
