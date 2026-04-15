//! Circuit breaker systems — bump counting toward automatic shockwave + bolt spawn.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::super::components::CircuitBreakerCounter;
use crate::{
    breaker::messages::BumpPerformed,
    effect_v3::{
        commands::FireEffectCommand,
        effects::{ShockwaveConfig, SpawnBoltsConfig},
        types::EffectType,
    },
};

/// Processes `BumpPerformed` messages to decrement circuit breaker counters.
///
/// When a counter reaches zero, queues reward effects (shockwave + spawn bolts)
/// via deferred [`FireEffectCommand`] and resets the counter. Processes bumps
/// sequentially per entity (fire-reset-continue within frame).
pub fn tick_circuit_breaker(
    mut bumps: MessageReader<BumpPerformed>,
    mut counter_query: Query<(Entity, &mut CircuitBreakerCounter)>,
    mut commands: Commands,
) {
    let bump_count = bumps.read().count();
    if bump_count == 0 {
        return;
    }

    for (entity, mut counter) in &mut counter_query {
        for _ in 0..bump_count {
            counter.remaining -= 1;
            if counter.remaining == 0 {
                // Queue reward shockwave.
                commands.queue(FireEffectCommand {
                    entity,
                    effect: EffectType::Shockwave(ShockwaveConfig {
                        base_range:      OrderedFloat(counter.shockwave_range),
                        range_per_level: OrderedFloat(0.0),
                        stacks:          1,
                        speed:           OrderedFloat(counter.shockwave_speed),
                    }),
                    source: String::new(),
                });

                // Queue reward bolt spawn.
                commands.queue(FireEffectCommand {
                    entity,
                    effect: EffectType::SpawnBolts(SpawnBoltsConfig {
                        count:    counter.spawn_count,
                        lifespan: None,
                        inherit:  counter.inherit,
                    }),
                    source: String::new(),
                });

                // Reset counter.
                counter.remaining = counter.bumps_required;
            }
        }
    }
}
