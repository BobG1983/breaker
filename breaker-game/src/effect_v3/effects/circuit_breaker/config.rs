//! `CircuitBreakerConfig` — bump counter toward automatic shockwave.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use super::components::CircuitBreakerCounter;
use crate::effect_v3::traits::{Fireable, Reversible};

/// Configuration for the circuit breaker counter mechanic.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Bumps needed per cycle before the reward fires.
    pub bumps_required:  u32,
    /// Number of extra bolts spawned as the reward.
    pub spawn_count:     u32,
    /// Whether spawned bolts inherit effect trees.
    pub inherit:         bool,
    /// Maximum radius of the reward shockwave.
    pub shockwave_range: OrderedFloat<f32>,
    /// Expansion speed of the reward shockwave.
    pub shockwave_speed: OrderedFloat<f32>,
}

impl Fireable for CircuitBreakerConfig {
    fn register(app: &mut App) {
        use super::systems::tick_circuit_breaker;
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            tick_circuit_breaker.in_set(EffectV3Systems::Tick),
        );
    }

    fn fire(&self, entity: Entity, _source: &str, world: &mut World) {
        if world.get_entity(entity).is_err() {
            return;
        }
        world.entity_mut(entity).insert(CircuitBreakerCounter {
            remaining:       self.bumps_required,
            bumps_required:  self.bumps_required,
            spawn_count:     self.spawn_count,
            inherit:         self.inherit,
            shockwave_range: self.shockwave_range.0,
            shockwave_speed: self.shockwave_speed.0,
        });
    }
}

impl Reversible for CircuitBreakerConfig {
    fn reverse(&self, entity: Entity, _source: &str, world: &mut World) {
        if world.get_entity(entity).is_ok() {
            world.entity_mut(entity).remove::<CircuitBreakerCounter>();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::traits::{Fireable, Reversible};

    fn make_config() -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            bumps_required:  5,
            spawn_count:     2,
            inherit:         false,
            shockwave_range: OrderedFloat(64.0),
            shockwave_speed: OrderedFloat(200.0),
        }
    }

    #[test]
    fn reverse_all_by_source_removes_counter_via_default_delegation() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        make_config().fire(entity, "circuit_chip", &mut world);
        assert!(world.get::<CircuitBreakerCounter>(entity).is_some());

        make_config().reverse_all_by_source(entity, "circuit_chip", &mut world);
        assert!(
            world.get::<CircuitBreakerCounter>(entity).is_none(),
            "CircuitBreakerCounter should be removed by default delegation"
        );

        // Calling twice does not panic.
        make_config().reverse_all_by_source(entity, "circuit_chip", &mut world);
    }
}
