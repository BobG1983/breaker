//! `FlashStep` teleport tests -- reversal dash during settling with `FlashStepActive`
//! teleports the breaker instantly instead of doing a normal dash.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

pub(super) use super::super::helpers::breaker_param_bundle;
use crate::{
    breaker::{
        components::{BaseWidth, Breaker, BreakerTilt, DashState, DashStateTimer},
        systems::dash::system::update_breaker_state,
        test_utils::default_breaker_definition,
    },
    effect_v3::effects::flash_step::FlashStepActive,
    input::resources::InputActions,
    shared::{PlayfieldConfig, test_utils::TestAppBuilder},
};

pub(super) fn test_app() -> App {
    TestAppBuilder::new()
        .with_resource::<InputActions>()
        .with_resource::<PlayfieldConfig>()
        .with_system(FixedUpdate, update_breaker_state)
        .build()
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
    let def = default_breaker_definition();
    let mut entity_cmds = app.world_mut().spawn((
        Breaker,
        DashState::Settling,
        Velocity2D(Vec2::ZERO),
        BreakerTilt {
            angle:       -0.35,
            ease_start:  -0.35,
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
    let def = default_breaker_definition();
    let mut entity_cmds = app.world_mut().spawn((
        Breaker,
        DashState::Settling,
        Velocity2D(Vec2::ZERO),
        BreakerTilt {
            angle:       0.35,
            ease_start:  0.35,
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
