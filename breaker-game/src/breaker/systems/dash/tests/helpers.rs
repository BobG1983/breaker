use bevy::prelude::*;
use rantzsoft_spatial2d::components::MaxSpeed;

use crate::{
    breaker::{
        components::{
            BrakeDecel, BrakeTilt, BreakerDeceleration, BreakerTilt, DashDuration,
            DashSpeedMultiplier, DashState, DashStateTimer, DashTilt, DashTiltEase, DecelEasing,
            SettleDuration, SettleTiltEase,
        },
        definition::BreakerDefinition,
        systems::dash::system::*,
        test_utils::default_breaker_definition,
    },
    prelude::*,
};

pub(super) fn breaker_param_bundle(
    def: &BreakerDefinition,
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
        MaxSpeed(def.max_speed),
        BreakerDeceleration(def.deceleration),
        DecelEasing {
            ease:     def.decel_ease,
            strength: def.decel_ease_strength,
        },
        DashSpeedMultiplier(def.dash_speed_multiplier),
        DashDuration(def.dash_duration),
        DashTilt(def.dash_tilt_angle.to_radians()),
        DashTiltEase(def.dash_tilt_ease),
        BrakeTilt {
            angle:    def.brake_tilt_angle.to_radians(),
            duration: def.brake_tilt_duration,
            ease:     def.brake_tilt_ease,
        },
        BrakeDecel(def.brake_decel_multiplier),
        SettleDuration(def.settle_duration),
        SettleTiltEase(def.settle_tilt_ease),
    )
}

pub(super) fn spawn_test_breaker(app: &mut App) -> Entity {
    let def = default_breaker_definition();
    app.world_mut()
        .spawn((
            Breaker,
            DashState::Idle,
            Velocity2D(Vec2::ZERO),
            BreakerTilt::default(),
            DashStateTimer { remaining: 0.0 },
            breaker_param_bundle(&def),
        ))
        .id()
}

pub(super) fn test_app() -> App {
    TestAppBuilder::new()
        .with_resource::<InputActions>()
        .with_resource::<PlayfieldConfig>()
        .with_system(FixedUpdate, update_breaker_state)
        .build()
}
