//! Shared helpers for `shield_tests` sub-modules.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Spatial2D};

use crate::{
    breaker::components::Breaker, effect::effects::shield::ShieldActive, shared::GameDrawLayer,
};

/// Spawns a breaker WITH `ShieldActive` for shield protection tests.
pub(super) fn spawn_shielded_breaker(app: &mut App, pos: Vec2, charges: u32) -> Entity {
    let entity = app
        .world_mut()
        .spawn((Breaker, Position2D(pos), Spatial2D, GameDrawLayer::Breaker))
        .id();
    app.world_mut()
        .entity_mut(entity)
        .insert(ShieldActive { charges });
    entity
}
