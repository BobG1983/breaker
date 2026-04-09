//! `FlashStep` teleport tests -- reversal dash during settling with `FlashStepActive`
//! teleports the breaker instantly instead of doing a normal dash.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{MaxSpeed, Position2D, Velocity2D};

use crate::{
    breaker::{
        components::{
            BaseWidth, BrakeDecel, BrakeTilt, Breaker, BreakerDeceleration, BreakerTilt,
            DashDuration, DashSpeedMultiplier, DashState, DashStateTimer, DashTilt, DashTiltEase,
            DecelEasing, SettleDuration, SettleTiltEase,
        },
        definition::BreakerDefinition,
        systems::dash::system::update_breaker_state,
    },
    effect::effects::flash_step::FlashStepActive,
    input::resources::InputActions,
    shared::PlayfieldConfig,
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
            ease: def.decel_ease,
            strength: def.decel_ease_strength,
        },
        DashSpeedMultiplier(def.dash_speed_multiplier),
        DashDuration(def.dash_duration),
        DashTilt(def.dash_tilt_angle.to_radians()),
        DashTiltEase(def.dash_tilt_ease),
        BrakeTilt {
            angle: def.brake_tilt_angle.to_radians(),
            duration: def.brake_tilt_duration,
            ease: def.brake_tilt_ease,
        },
        BrakeDecel(def.brake_decel_multiplier),
        SettleDuration(def.settle_duration),
        SettleTiltEase(def.settle_tilt_ease),
    )
}

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<InputActions>()
        .init_resource::<PlayfieldConfig>()
        .add_systems(FixedUpdate, update_breaker_state);
    app
}

pub(super) use crate::shared::test_utils::tick;

/// Spawns a breaker in Settling state with a rightward-dash settle tilt
/// (`ease_start` < 0, meaning last dash was rightward).
///
/// Returns the entity ID.
pub(super) fn spawn_settling_breaker_rightward_dash(
    app: &mut App,
    position: Vec2,
    flash_step: bool,
) -> Entity {
    let def = BreakerDefinition::default();
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
        BaseWidth(def.width),
        breaker_param_bundle(&def),
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
    let def = BreakerDefinition::default();
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
        BaseWidth(def.width),
        breaker_param_bundle(&def),
    ));
    if flash_step {
        entity_cmds.insert(FlashStepActive);
    }
    entity_cmds.id()
}
