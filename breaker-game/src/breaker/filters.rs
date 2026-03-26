//! Breaker domain query filters.

use bevy::prelude::*;

use crate::{
    bolt::components::Bolt,
    breaker::components::{Breaker, BumpVisual},
};

/// Query filter for breakers eligible for a new bump visual (not already animating).
pub type BumpTriggerFilter = (With<Breaker>, Without<BumpVisual>);

/// Breaker entities for collision queries (excludes bolt for query disjointness).
pub(crate) type CollisionFilterBreaker = (With<Breaker>, Without<Bolt>);
