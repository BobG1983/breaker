//! Breaker domain query filters.

use bevy::prelude::*;

use crate::breaker::components::{Breaker, BumpVisual};

/// Query filter for breakers eligible for a new bump visual (not already animating).
pub type BumpTriggerFilter = (With<Breaker>, Without<BumpVisual>);
