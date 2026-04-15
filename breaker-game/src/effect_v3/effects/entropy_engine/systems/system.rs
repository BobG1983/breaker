//! Entropy engine systems — reset counter on node start and tick bump counting.

use bevy::prelude::*;
use rand_chacha::ChaCha8Rng;

use super::super::components::EntropyCounter;
use crate::{
    breaker::messages::BumpPerformed,
    effect_v3::{commands::EffectCommandsExt, components::EffectSourceChip, types::EffectType},
    shared::rng::GameRng,
};

/// Resets all `EntropyCounter` components to zero at the start of each node.
pub fn reset_entropy_counter(mut query: Query<&mut EntropyCounter>) {
    for mut counter in &mut query {
        counter.count = 0;
    }
}

/// Processes `BumpPerformed` messages to increment entropy counters and
/// fire escalating random effects.
///
/// For each bump: increments count (capped at `max_effects`), then queues
/// N deferred fires where N = current count, each selected from the
/// weighted pool via `GameRng`. Fires are queued through
/// `commands.fire_effect`, which runs during the next command flush.
pub fn tick_entropy_engine(
    mut commands: Commands,
    mut counters: Query<(Entity, &mut EntropyCounter, Option<&EffectSourceChip>)>,
    mut bumps: MessageReader<BumpPerformed>,
    mut rng: ResMut<GameRng>,
) {
    let bump_count = bumps.read().count();
    if bump_count == 0 {
        return;
    }

    for (entity, mut counter, chip) in &mut counters {
        // Resolve the source string once per entity.
        // EffectSourceChip(None) and the absence of the component both map to "".
        let source: String = chip.and_then(|c| c.0.clone()).unwrap_or_default();

        for _ in 0..bump_count {
            // Increment count (capped at max_effects).
            if counter.count < counter.max_effects {
                counter.count += 1;
            }

            // Fire `count` random effects from pool.
            if counter.pool.is_empty() {
                continue;
            }

            let total_weight: f32 = counter.pool.iter().map(|(w, _)| w.0).sum();
            if total_weight <= 0.0 {
                continue;
            }

            for _ in 0..counter.count {
                if let Some(effect_type) =
                    pick_weighted_effect(&counter.pool, &mut rng.0, total_weight)
                {
                    commands.fire_effect(entity, effect_type, source.clone());
                }
            }
        }
    }
}

/// Pick a random effect from a weighted pool using the supplied RNG.
fn pick_weighted_effect(
    pool: &[(ordered_float::OrderedFloat<f32>, Box<EffectType>)],
    rng: &mut ChaCha8Rng,
    total_weight: f32,
) -> Option<EffectType> {
    use rand::Rng;

    if pool.is_empty() || total_weight <= 0.0 {
        return None;
    }

    let roll: f32 = rng.random::<f32>() * total_weight;
    let mut accumulated = 0.0;
    for (weight, effect) in pool {
        accumulated += weight.0;
        if roll < accumulated {
            return Some((**effect).clone());
        }
    }
    // Fallback to last entry if floating-point imprecision causes no match.
    Some((*pool[pool.len() - 1].1).clone())
}
