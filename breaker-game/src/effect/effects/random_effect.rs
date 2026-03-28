use bevy::prelude::*;

use crate::effect::core::EffectNode;

/// Selects a random effect from a weighted pool and fires it.
///
/// Uses deterministic fallback (first element) when the pool is non-empty.
/// A proper weighted random implementation will use `GameRng` once wired.
pub(crate) fn fire(entity: Entity, pool: &[(f32, EffectNode)], world: &mut World) {
    if pool.is_empty() {
        warn!("random_effect: empty pool for entity {:?}", entity);
        return;
    }

    // Deterministic fallback: pick first element.
    // TODO: weighted random selection using GameRng.
    let (_weight, ref node) = pool[0];

    match node {
        EffectNode::Do(effect) => {
            info!(
                "random_effect: selected effect {:?} for entity {:?}",
                effect, entity
            );
            // Leaf effect dispatch will be wired through the effect system.
        }
        _ => {
            info!(
                "random_effect: selected non-leaf node for entity {:?}, staging",
                entity
            );
            // Non-leaf nodes would be pushed to StagedEffects on the entity.
        }
    }
}

/// No-op — inner effects handle their own reversal.
pub(crate) fn reverse(_entity: Entity, _pool: &[(f32, EffectNode)], _world: &mut World) {}

/// Registers systems for `RandomEffect` effect.
pub(crate) fn register(_app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::core::EffectKind;

    #[test]
    fn fire_with_single_element_pool_selects_that_element() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

        // Should not panic; selects the single element.
        fire(entity, &pool, &mut world);
    }

    #[test]
    fn fire_with_empty_pool_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let pool: Vec<(f32, EffectNode)> = vec![];

        // Should not panic — early return on empty pool.
        fire(entity, &pool, &mut world);
    }
}
