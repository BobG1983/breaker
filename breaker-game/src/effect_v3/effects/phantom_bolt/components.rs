//! Phantom bolt runtime components.

use bevy::prelude::*;

/// Marker identifying an entity as a phantom bolt.
#[derive(Component, Debug, Clone)]
pub struct PhantomBolt;

/// Remaining lifetime in seconds before the phantom bolt despawns.
#[derive(Component, Debug, Clone)]
pub struct PhantomLifetime(pub f32);

/// Entity that spawned this phantom bolt (for ownership tracking).
#[derive(Component, Debug, Clone)]
pub struct PhantomOwner(pub Entity);
