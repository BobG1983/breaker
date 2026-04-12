//! `ShieldActive` condition evaluator.

use bevy::prelude::*;

use crate::effect_v3::effects::shield::ShieldWall;

/// Evaluate whether the `ShieldActive` condition is currently true.
///
/// Returns true while at least one `ShieldWall` entity exists in the world.
pub fn is_shield_active(world: &World) -> bool {
    // Use component_id to check if any entities have the ShieldWall component
    // without needing &mut World for query_filtered.
    let Some(component_id) = world.component_id::<ShieldWall>() else {
        return false;
    };
    world
        .archetypes()
        .iter()
        .any(|archetype| archetype.contains(component_id) && !archetype.is_empty())
}
