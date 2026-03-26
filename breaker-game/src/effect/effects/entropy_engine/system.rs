//! Entropy engine evolution effect — counts cell destructions and fires
//! a random effect from the pool when the threshold is reached.
//!
//! Observes [`EntropyEngineFired`] and maintains [`EntropyEngineCounter`] resource.
//! When the counter reaches the threshold, a weighted random entry from the pool
//! is fired (for leaves) or armed (for non-leaf chains), then the counter resets.

use bevy::prelude::*;
use rand::Rng;

use crate::{
    effect::{
        definition::{EffectNode, EffectTarget},
        typed_events::fire_typed_event,
    },
    shared::GameRng,
};

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when an entropy engine effect needs counting and potential resolution.
#[derive(Event, Clone, Debug)]
pub(crate) struct EntropyEngineFired {
    /// Number of cell destructions needed before firing.
    pub threshold: u32,
    /// Weighted pool of `EffectNode` entries to select from on trigger.
    pub pool: Vec<(f32, EffectNode)>,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Tracks the cumulative cell destruction count for the entropy engine.
#[derive(Resource, Debug, Default)]
pub(crate) struct EntropyEngineCounter {
    /// Number of cell destructions counted since last reset.
    pub count: u32,
}

// ---------------------------------------------------------------------------
// Observer — handles entropy engine counting and firing
// ---------------------------------------------------------------------------

/// Observer: increments the cell destruction counter and fires a random
/// effect from the pool when the threshold is reached.
pub(crate) fn handle_entropy_engine(
    trigger: On<EntropyEngineFired>,
    mut counter: Option<ResMut<EntropyEngineCounter>>,
    mut rng: ResMut<GameRng>,
    mut commands: Commands,
) {
    let event = trigger.event();

    // Get or init the counter
    let new_count = if let Some(ref mut counter) = counter {
        counter.count += 1;
        counter.count
    } else {
        // Counter doesn't exist yet — will insert via commands
        1
    };

    if new_count >= event.threshold {
        // Reset counter
        if let Some(ref mut counter) = counter {
            counter.count = 0;
        } else {
            commands.insert_resource(EntropyEngineCounter { count: 0 });
        }

        // Weighted random selection from pool
        let pool = &event.pool;
        if pool.is_empty() {
            return;
        }

        let total_weight: f32 = pool.iter().map(|(w, _)| *w).sum();
        let roll: f32 = rng.0.random::<f32>() * total_weight;
        let mut cumulative = 0.0;
        let mut selected_idx = pool.len() - 1;
        for (i, (weight, _)) in pool.iter().enumerate() {
            cumulative += weight;
            if roll < cumulative {
                selected_idx = i;
                break;
            }
        }

        let (_, node) = &pool[selected_idx];

        match node {
            EffectNode::Do(effect) => {
                fire_typed_event(
                    effect.clone(),
                    event.targets.clone(),
                    event.source_chip.clone(),
                    &mut commands,
                );
            }
            _ => {
                // TODO(C7W2): arm non-leaf chains on bolt entity
                warn!(
                    "EntropyEngine selected non-leaf chain — arming not yet supported in this handler"
                );
            }
        }
    } else if counter.is_none() {
        // Counter didn't exist, insert with new count
        commands.insert_resource(EntropyEngineCounter { count: new_count });
    }
}

/// Registers all observers and systems for the entropy engine effect.
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_entropy_engine);
}
