//! Escalating chaos — fires multiple random effects on the primary bolt per cell destroyed.

use bevy::prelude::*;

use crate::effect::EffectNode;

/// Tracks the kill count within the current node for entropy scaling.
#[derive(Component, Debug, Clone)]
pub struct EntropyEngineState {
    /// Cells destroyed this node (resets between nodes).
    pub kill_count: u32,
}

/// Fires multiple random effects from the weighted pool.
///
/// Number of effects scales with kill count up to `max_effects`.
/// Resets between nodes.
pub(crate) fn fire(
    entity: Entity,
    max_effects: u32,
    pool: &[(f32, EffectNode)],
    world: &mut World,
) {
    let _ = (entity, max_effects, pool, world);
    // Placeholder — real implementation in Wave 8
}

/// No-op — inner effects handle their own reversal.
pub(crate) fn reverse(_entity: Entity, _world: &mut World) {}

/// Registers systems for `EntropyEngine` effect.
pub(crate) fn register(_app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_completes_without_panic() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        fire(entity, 3, &[], &mut world);
    }

    #[test]
    fn reverse_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        fire(entity, 3, &[], &mut world);
        reverse(entity, &mut world);
    }
}
