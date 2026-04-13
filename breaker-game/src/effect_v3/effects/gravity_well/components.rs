//! Gravity well runtime components.

use bevy::prelude::*;

/// Marker identifying an entity as a gravity well source.
#[derive(Component, Debug, Clone)]
pub struct GravityWellSource;

/// Attractive force magnitude of the gravity well.
#[derive(Component, Debug, Clone)]
pub struct GravityWellStrength(pub f32);

/// Radius of the gravity well's influence area.
#[derive(Component, Debug, Clone)]
pub struct GravityWellRadius(pub f32);

/// Remaining lifetime in seconds before the well despawns.
#[derive(Component, Debug, Clone)]
pub struct GravityWellLifetime(pub f32);

/// Entity that spawned this gravity well (for ownership tracking).
#[derive(Component, Debug, Clone)]
pub struct GravityWellOwner(pub Entity);

/// Monotonically increasing counter tracking when each gravity well was spawned.
/// Used for deterministic FIFO eviction — lowest order value = oldest well.
#[derive(Component, Debug, Clone)]
pub struct GravityWellSpawnOrder(pub u32);
