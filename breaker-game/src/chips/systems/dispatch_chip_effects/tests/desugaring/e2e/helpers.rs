//! Shared helpers for end-to-end dispatch tests.
//!
//! The old e2e tests tested desugaring + trigger evaluation. In the new system,
//! trigger evaluation happens via `effect_v3` bridges. These helpers are retained
//! for future e2e integration tests.

use bevy::prelude::*;

use crate::prelude::*;

/// Assert that an entity has exactly `count` `BoundEffects` entries.
pub(super) fn assert_bound_count(world: &World, entity: Entity, count: usize) {
    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound.0.len(),
        count,
        "Expected {count} BoundEffects entries, got {}",
        bound.0.len()
    );
}
