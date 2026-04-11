//! Second wind runtime components.

use bevy::prelude::*;

/// Marker identifying an entity as a second-wind wall.
#[derive(Component, Debug, Clone)]
pub struct SecondWindWall;

/// Entity that owns this second-wind wall (for cleanup on reverse).
#[derive(Component, Debug, Clone)]
pub struct SecondWindOwner(pub Entity);
