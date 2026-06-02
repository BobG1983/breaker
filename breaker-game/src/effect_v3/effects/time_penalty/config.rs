//! `TimePenaltyConfig` — fire-and-forget time subtraction.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::{effect_v3::traits::Fireable, state::run::node::messages::ReduceNodeTimer};

/// Subtracts seconds from the node timer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimePenaltyConfig {
    /// Number of seconds subtracted from the node timer.
    pub seconds: OrderedFloat<f32>,
}

impl Fireable for TimePenaltyConfig {
    fn fire(
        &self,
        _entity: Entity,
        _source: &str,
        world: &mut World,
        _rng: &mut rand_chacha::ChaCha8Rng,
    ) {
        world
            .resource_mut::<Messages<ReduceNodeTimer>>()
            .write(ReduceNodeTimer {
                delta: self.seconds.0,
            });
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use super::*;
    use crate::{
        effect_v3::traits::Fireable,
        state::run::node::{messages::ReduceNodeTimer, resources::NodeTimer},
    };

    /// Collects the `ReduceNodeTimer` messages currently buffered in the world.
    fn collect_reduce_messages(world: &World) -> Vec<ReduceNodeTimer> {
        world
            .resource::<Messages<ReduceNodeTimer>>()
            .iter_current_update_messages()
            .cloned()
            .collect()
    }

    #[test]
    fn fire_writes_reduce_node_timer_message_with_configured_delta() {
        let mut world = World::new();
        world.init_resource::<Messages<ReduceNodeTimer>>();
        world.insert_resource(NodeTimer {
            remaining: 30.0,
            total:     30.0,
        });
        let entity = world.spawn_empty().id();

        let config = TimePenaltyConfig {
            seconds: OrderedFloat(5.0),
        };
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        config.fire(entity, "test_source", &mut world, &mut rng);

        let messages = collect_reduce_messages(&world);
        assert_eq!(
            messages.len(),
            1,
            "fire() must write exactly one ReduceNodeTimer message, got {}",
            messages.len(),
        );
        assert!(
            (messages[0].delta - 5.0).abs() < f32::EPSILON,
            "ReduceNodeTimer.delta should mirror config.seconds (5.0), got {}",
            messages[0].delta,
        );

        // fire() must NOT mutate NodeTimer directly — that's the consumer
        // system's job, run later in the same fixed step.
        let timer = world.resource::<NodeTimer>();
        assert!(
            (timer.remaining - 30.0).abs() < f32::EPSILON,
            "fire() must not mutate NodeTimer.remaining; expected 30.0, got {}",
            timer.remaining,
        );
        assert!(
            (timer.total - 30.0).abs() < f32::EPSILON,
            "fire() must not mutate NodeTimer.total; expected 30.0, got {}",
            timer.total,
        );
    }

    #[test]
    fn fire_writes_message_even_when_node_timer_resource_absent() {
        let mut world = World::new();
        world.init_resource::<Messages<ReduceNodeTimer>>();
        // No NodeTimer inserted — fire() must still emit the message and
        // must not panic. (The consumer's existence is its concern, not
        // fire()'s.)
        let entity = world.spawn_empty().id();

        let config = TimePenaltyConfig {
            seconds: OrderedFloat(7.0),
        };
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        config.fire(entity, "test_source", &mut world, &mut rng);

        let messages = collect_reduce_messages(&world);
        assert_eq!(
            messages.len(),
            1,
            "fire() must write exactly one ReduceNodeTimer message even when \
             NodeTimer resource is absent, got {}",
            messages.len(),
        );
        assert!(
            (messages[0].delta - 7.0).abs() < f32::EPSILON,
            "ReduceNodeTimer.delta should mirror config.seconds (7.0), got {}",
            messages[0].delta,
        );
    }

    #[test]
    fn fire_with_zero_seconds_writes_message_with_zero_delta() {
        let mut world = World::new();
        world.init_resource::<Messages<ReduceNodeTimer>>();
        world.insert_resource(NodeTimer {
            remaining: 30.0,
            total:     30.0,
        });
        let entity = world.spawn_empty().id();

        let config = TimePenaltyConfig {
            seconds: OrderedFloat(0.0),
        };
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        config.fire(entity, "test_source", &mut world, &mut rng);

        let messages = collect_reduce_messages(&world);
        assert_eq!(
            messages.len(),
            1,
            "fire() must write exactly one ReduceNodeTimer message even when \
             seconds is 0.0, got {}",
            messages.len(),
        );
        assert!(
            messages[0].delta.abs() < f32::EPSILON,
            "ReduceNodeTimer.delta should be 0.0 (faithful to config.seconds), got {}",
            messages[0].delta,
        );
    }
}
