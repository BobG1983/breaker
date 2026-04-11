//! Impact trigger bridge systems.
//!
//! Each bridge reads a collision message, builds a [`TriggerContext`], and dispatches
//! the corresponding trigger to entities with bound effects.

use bevy::prelude::*;

use crate::effect_v3::types::{Trigger, TriggerContext};

/// Local bridge: fires `Impacted(entity_kind)` on entities involved in a collision.
///
/// Handles all 6 collision types:
/// - `BoltImpactCell`
/// - `BoltImpactWall`
/// - `BoltImpactBreaker`
/// - `BreakerImpactCell`
/// - `BreakerImpactWall`
/// - `CellImpactWall`
pub fn on_impacted() {
    todo!()
}

/// Global bridge: fires `ImpactOccurred(entity_kind)` on all entities with bound
/// effects when a collision involving the given entity kind happens.
///
/// Handles all 6 collision types.
pub fn on_impact_occurred() {
    todo!()
}
