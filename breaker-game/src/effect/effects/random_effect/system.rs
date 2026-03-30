use bevy::prelude::*;
use rand::distr::{Distribution, weighted::WeightedIndex};

use crate::{
    effect::core::{EffectNode, StagedEffects},
    shared::rng::GameRng,
};

/// Selects a random effect from a weighted pool and fires it.
///
/// Uses `GameRng` with `WeightedIndex` for deterministic weighted random selection.
pub(crate) fn fire(
    entity: Entity,
    pool: &[(f32, EffectNode)],
    source_chip: &str,
    world: &mut World,
) {
    if pool.is_empty() {
        warn!("random_effect: empty pool for entity {:?}", entity);
        return;
    }

    let selected_node = {
        let mut rng = world.resource_mut::<GameRng>();
        let weights: Vec<f32> = pool.iter().map(|(w, _)| *w).collect();
        let Ok(dist) = WeightedIndex::new(&weights) else {
            warn!("random_effect: all-zero weights for entity {:?}", entity);
            return;
        };
        let idx = dist.sample(&mut rng.0);
        pool[idx].1.clone()
    };

    match selected_node {
        EffectNode::Do(effect) => effect.fire(entity, source_chip, world),
        other => {
            if let Some(mut staged) = world.get_mut::<StagedEffects>(entity) {
                staged.0.push((source_chip.to_string(), other));
            }
        }
    }
}

/// No-op — inner effects handle their own reversal.
pub(crate) const fn reverse(
    _entity: Entity,
    _pool: &[(f32, EffectNode)],
    _source_chip: &str,
    _world: &mut World,
) {
}

/// Registers systems for `RandomEffect` effect.
pub(crate) const fn register(_app: &mut App) {}
