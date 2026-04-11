//! Escalating chaos — fires multiple random effects on the primary bolt per cell destroyed.

use bevy::prelude::*;
use rand::distr::{Distribution, weighted::WeightedIndex};

use crate::{
    effect::{EffectNode, StagedEffects},
    shared::rng::GameRng,
    state::types::NodeState,
};

/// Tracks cells destroyed within the current node for entropy scaling.
#[derive(Component, Debug, Clone)]
pub(crate) struct EntropyEngineState {
    /// Cells destroyed this node (resets between nodes).
    pub cells_destroyed: u32,
}

/// Fires multiple random effects from the weighted pool.
///
/// Number of effects scales with cells destroyed up to `max_effects`.
/// Resets between nodes.
pub(crate) fn fire(
    entity: Entity,
    max_effects: u32,
    pool: &[(f32, EffectNode)],
    source_chip: &str,
    world: &mut World,
) {
    // Step 1: Insert EntropyEngineState if absent
    if world.get::<EntropyEngineState>(entity).is_none() {
        world
            .entity_mut(entity)
            .insert(EntropyEngineState { cells_destroyed: 0 });
    }

    // Step 2: Increment cells_destroyed and compute effects to fire
    let effects_to_fire = {
        // EntropyEngineState was just inserted above if absent
        let Some(mut state) = world.get_mut::<EntropyEngineState>(entity) else {
            return;
        };
        state.cells_destroyed = state.cells_destroyed.saturating_add(1);
        state.cells_destroyed.min(max_effects)
    };

    // Step 3: Empty pool guard AFTER cells_destroyed increment
    if pool.is_empty() {
        warn!("entropy_engine: empty pool for entity {:?}", entity);
        return;
    }

    if effects_to_fire == 0 {
        return;
    }

    // Step 4: Pre-sample all indices from GameRng before dispatching
    let selected_indices: Vec<usize> = {
        let mut rng = world.resource_mut::<GameRng>();
        let weights: Vec<f32> = pool.iter().map(|(w, _)| *w).collect();
        let Ok(dist) = WeightedIndex::new(&weights) else {
            warn!("entropy_engine: all-zero weights for entity {:?}", entity);
            return;
        };
        (0..effects_to_fire)
            .map(|_| dist.sample(&mut rng.0))
            .collect()
    };

    // Step 5: Dispatch effects
    for idx in selected_indices {
        let node = pool[idx].1.clone();
        match node {
            EffectNode::Do(effect) => effect.fire(entity, source_chip, world),
            other => {
                if let Some(mut staged) = world.get_mut::<StagedEffects>(entity) {
                    staged.0.push((source_chip.to_string(), other));
                } else {
                    world
                        .entity_mut(entity)
                        .insert(StagedEffects(vec![(source_chip.to_string(), other)]));
                }
            }
        }
    }
}

/// No-op — inner effects handle their own reversal.
pub(crate) const fn reverse(_entity: Entity, _source_chip: &str, _world: &mut World) {}

/// Registers systems for `EntropyEngine` effect.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        OnEnter(NodeState::Playing),
        reset_entropy_engine_on_node_start,
    );
}

fn reset_entropy_engine_on_node_start(mut query: Query<&mut EntropyEngineState>) {
    for mut state in &mut query {
        state.cells_destroyed = 0;
    }
}
