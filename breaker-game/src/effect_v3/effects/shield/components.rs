//! Shield runtime components.

use bevy::prelude::*;

/// Marker identifying an entity as a shield wall.
#[derive(Component, Debug, Clone)]
pub struct ShieldWall;

/// Entity that owns this shield (for cleanup on reverse).
#[derive(Component, Debug, Clone)]
pub struct ShieldOwner(pub Entity);

/// Remaining duration in seconds before the shield despawns.
#[derive(Component, Debug, Clone)]
pub struct ShieldDuration(pub f32);

/// Duration cost consumed from the shield when a bolt reflects off it.
#[derive(Component, Debug, Clone)]
pub struct ShieldReflectionCost(pub f32);
