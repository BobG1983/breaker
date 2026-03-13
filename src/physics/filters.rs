//! Physics domain query filters.
//!
//! Disambiguation filters for collision systems that query multiple entity types
//! in the same system (bolt, breaker, cell, wall).

use bevy::prelude::*;

use crate::{
    bolt::components::Bolt, breaker::components::Breaker, cells::components::Cell,
    wall::components::Wall,
};

/// Breaker entities for collision queries (excludes bolt for query disjointness).
pub type BreakerCollisionFilter = (With<Breaker>, Without<Bolt>);

/// Cell entities for collision queries (excludes bolt and wall for query disjointness).
pub type CellCollisionFilter = (With<Cell>, Without<Bolt>, Without<Wall>);

/// Wall entities for collision queries (excludes bolt and cell for query disjointness).
pub type WallCollisionFilter = (With<Wall>, Without<Bolt>, Without<Cell>);
