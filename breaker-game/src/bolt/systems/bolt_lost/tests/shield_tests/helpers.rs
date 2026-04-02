//! Shared helpers for `shield_tests` sub-modules.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    breaker::{components::Breaker, definition::BreakerDefinition},
    effect::effects::shield::ShieldActive,
};

/// Spawns a breaker WITH `ShieldActive` for shield protection tests.
pub(super) fn spawn_shielded_breaker(app: &mut App, pos: Vec2, charges: u32) -> Entity {
    let def = BreakerDefinition::default();
    let entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut()
        .entity_mut(entity)
        .insert((Position2D(pos), ShieldActive { charges }));
    entity
}
