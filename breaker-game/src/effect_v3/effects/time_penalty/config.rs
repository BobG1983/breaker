//! `TimePenaltyConfig` — fire-and-forget time subtraction.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::{effect_v3::traits::Fireable, prelude::*};

/// Subtracts seconds from the node timer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimePenaltyConfig {
    /// Number of seconds subtracted from the node timer.
    pub seconds: OrderedFloat<f32>,
}

impl Fireable for TimePenaltyConfig {
    fn fire(&self, _entity: Entity, _source: &str, world: &mut World) {
        if let Some(mut timer) = world.get_resource_mut::<NodeTimer>() {
            timer.remaining = (timer.remaining - self.seconds.0).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::traits::Fireable;

    #[test]
    fn fire_subtracts_configured_seconds_from_remaining() {
        let mut world = World::new();
        world.insert_resource(NodeTimer {
            remaining: 30.0,
            total:     60.0,
        });
        let entity = world.spawn_empty().id();

        let config = TimePenaltyConfig {
            seconds: OrderedFloat(5.0),
        };
        config.fire(entity, "test_source", &mut world);

        let timer = world.resource::<NodeTimer>();
        assert!(
            (timer.remaining - 25.0).abs() < f32::EPSILON,
            "remaining should be 25.0 (30.0 - 5.0), got {}",
            timer.remaining,
        );
        assert!(
            (timer.total - 60.0).abs() < f32::EPSILON,
            "total should be unchanged at 60.0"
        );
    }

    #[test]
    fn fire_clamps_remaining_at_zero() {
        let mut world = World::new();
        world.insert_resource(NodeTimer {
            remaining: 3.0,
            total:     60.0,
        });
        let entity = world.spawn_empty().id();

        let config = TimePenaltyConfig {
            seconds: OrderedFloat(10.0),
        };
        config.fire(entity, "test_source", &mut world);

        let timer = world.resource::<NodeTimer>();
        assert!(
            timer.remaining.abs() < f32::EPSILON,
            "remaining should be clamped to 0.0, not {}",
            timer.remaining,
        );
    }

    #[test]
    fn fire_with_no_node_timer_resource_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let config = TimePenaltyConfig {
            seconds: OrderedFloat(5.0),
        };
        // Should not panic.
        config.fire(entity, "test_source", &mut world);
    }

    #[test]
    fn fire_with_remaining_already_zero_stays_at_zero() {
        let mut world = World::new();
        world.insert_resource(NodeTimer {
            remaining: 0.0,
            total:     60.0,
        });
        let entity = world.spawn_empty().id();

        let config = TimePenaltyConfig {
            seconds: OrderedFloat(5.0),
        };
        config.fire(entity, "test_source", &mut world);

        let timer = world.resource::<NodeTimer>();
        assert!(
            timer.remaining.abs() < f32::EPSILON,
            "remaining should stay at 0.0"
        );
    }

    #[test]
    fn multiple_fires_stack_subtractively() {
        let mut world = World::new();
        world.insert_resource(NodeTimer {
            remaining: 20.0,
            total:     60.0,
        });
        let entity = world.spawn_empty().id();

        let config = TimePenaltyConfig {
            seconds: OrderedFloat(7.0),
        };
        config.fire(entity, "test_source", &mut world);
        config.fire(entity, "test_source", &mut world);

        let timer = world.resource::<NodeTimer>();
        assert!(
            (timer.remaining - 6.0).abs() < f32::EPSILON,
            "remaining should be 6.0 (20.0 - 7.0 - 7.0), got {}",
            timer.remaining,
        );
    }
}
