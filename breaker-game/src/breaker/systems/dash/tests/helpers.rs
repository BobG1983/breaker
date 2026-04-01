use bevy::prelude::*;
use rantzsoft_spatial2d::components::MaxSpeed;

use super::super::system::*;
use crate::{
    breaker::{
        components::{
            BrakeDecel, BrakeTilt, Breaker, BreakerDeceleration, BreakerState, BreakerStateTimer,
            BreakerTilt, BreakerVelocity, DashDuration, DashSpeedMultiplier, DashTilt,
            DashTiltEase, DecelEasing, SettleDuration, SettleTiltEase,
        },
        resources::BreakerConfig,
    },
    input::resources::InputActions,
    shared::PlayfieldConfig,
};

pub(super) fn breaker_param_bundle(
    config: &BreakerConfig,
) -> (
    MaxSpeed,
    BreakerDeceleration,
    DecelEasing,
    DashSpeedMultiplier,
    DashDuration,
    DashTilt,
    DashTiltEase,
    BrakeTilt,
    BrakeDecel,
    SettleDuration,
    SettleTiltEase,
) {
    (
        MaxSpeed(config.max_speed),
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
        SettleDuration(config.settle_duration),
        SettleTiltEase(config.settle_tilt_ease),
    )
}

pub(super) fn spawn_test_breaker(app: &mut App) -> Entity {
    let config = BreakerConfig::default();
    app.world_mut()
        .spawn((
            Breaker,
            BreakerState::Idle,
            BreakerVelocity { x: 0.0 },
            BreakerTilt::default(),
            BreakerStateTimer { remaining: 0.0 },
            breaker_param_bundle(&config),
        ))
        .id()
}

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<BreakerConfig>()
        .init_resource::<InputActions>()
        .init_resource::<PlayfieldConfig>()
        .add_systems(FixedUpdate, update_breaker_state);
    app
}

/// Accumulates one fixed timestep of overstep, then runs one update.
pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}
