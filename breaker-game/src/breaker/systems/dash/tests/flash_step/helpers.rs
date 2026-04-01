//! `FlashStep` teleport tests -- reversal dash during settling with `FlashStepActive`
//! teleports the breaker instantly instead of doing a normal dash.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{MaxSpeed, Position2D, Velocity2D};

use super::super::super::system::update_breaker_state;
use crate::{
    breaker::{
        components::{
            BaseWidth, BrakeDecel, BrakeTilt, Breaker, BreakerDeceleration, BreakerTilt,
            DashDuration, DashSpeedMultiplier, DashState, DashStateTimer, DashTilt, DashTiltEase,
            DecelEasing, SettleDuration, SettleTiltEase,
        },
        resources::BreakerConfig,
    },
    effect::effects::flash_step::FlashStepActive,
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

/// Spawns a breaker in Settling state with a rightward-dash settle tilt
/// (`ease_start` < 0, meaning last dash was rightward).
///
/// Returns the entity ID.
pub(super) fn spawn_settling_breaker_rightward_dash(
    app: &mut App,
    position: Vec2,
    flash_step: bool,
) -> Entity {
    let config = BreakerConfig::default();
    let mut entity_cmds = app.world_mut().spawn((
        Breaker,
        DashState::Settling,
        Velocity2D(Vec2::ZERO),
        BreakerTilt {
            angle: -0.35,
            ease_start: -0.35,
            ease_target: 0.0,
        },
        DashStateTimer { remaining: 0.2 },
        Position2D(position),
        BaseWidth(120.0),
        breaker_param_bundle(&config),
    ));
    if flash_step {
        entity_cmds.insert(FlashStepActive);
    }
    entity_cmds.id()
}

/// Spawns a breaker in Settling state with a leftward-dash settle tilt
/// (`ease_start` > 0, meaning last dash was leftward).
///
/// Returns the entity ID.
pub(super) fn spawn_settling_breaker_leftward_dash(
    app: &mut App,
    position: Vec2,
    flash_step: bool,
) -> Entity {
    let config = BreakerConfig::default();
    let mut entity_cmds = app.world_mut().spawn((
        Breaker,
        DashState::Settling,
        Velocity2D(Vec2::ZERO),
        BreakerTilt {
            angle: 0.35,
            ease_start: 0.35,
            ease_target: 0.0,
        },
        DashStateTimer { remaining: 0.2 },
        Position2D(position),
        BaseWidth(120.0),
        breaker_param_bundle(&config),
    ));
    if flash_step {
        entity_cmds.insert(FlashStepActive);
    }
    entity_cmds.id()
}
