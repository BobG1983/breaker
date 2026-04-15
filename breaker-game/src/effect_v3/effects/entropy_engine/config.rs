//! `EntropyConfig` — random effect trigger based on bump accumulation.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use super::components::EntropyCounter;
use crate::effect_v3::{
    traits::{Fireable, Reversible},
    types::EffectType,
};

/// Configuration for the entropy engine mechanic.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntropyConfig {
    /// Cap on how many effects fire per activation.
    pub max_effects: u32,
    /// Weighted list of effects — each entry is (weight, effect).
    pub pool:        Vec<(OrderedFloat<f32>, Box<EffectType>)>,
}

impl Fireable for EntropyConfig {
    fn fire(&self, entity: Entity, _source: &str, world: &mut World) {
        if world.get_entity(entity).is_err() {
            return;
        }
        world.entity_mut(entity).insert(EntropyCounter {
            count:       0,
            max_effects: self.max_effects,
            pool:        self.pool.clone(),
        });
    }

    fn register(app: &mut App) {
        use super::systems::{reset_entropy_counter, tick_entropy_engine};
        use crate::{effect_v3::EffectV3Systems, prelude::*};

        app.add_systems(
            OnEnter(NodeState::Loading),
            reset_entropy_counter.in_set(EffectV3Systems::Reset),
        );
        app.add_systems(
            FixedUpdate,
            tick_entropy_engine.in_set(EffectV3Systems::Tick),
        );
    }
}

impl Reversible for EntropyConfig {
    fn reverse(&self, entity: Entity, _source: &str, world: &mut World) {
        if world.get_entity(entity).is_ok() {
            world.entity_mut(entity).remove::<EntropyCounter>();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;
    use crate::effect_v3::traits::{Fireable, Reversible};

    fn make_config() -> EntropyConfig {
        EntropyConfig {
            max_effects: 3,
            pool:        vec![],
        }
    }

    fn make_config_with_pool(
        max_effects: u32,
        pool: Vec<(OrderedFloat<f32>, Box<EffectType>)>,
    ) -> EntropyConfig {
        EntropyConfig { max_effects, pool }
    }

    fn shockwave_entry() -> (OrderedFloat<f32>, Box<EffectType>) {
        (
            OrderedFloat(1.0),
            Box::new(EffectType::Shockwave(
                crate::effect_v3::effects::ShockwaveConfig {
                    base_range:      OrderedFloat(48.0),
                    range_per_level: OrderedFloat(0.0),
                    stacks:          1,
                    speed:           OrderedFloat(150.0),
                },
            )),
        )
    }

    fn speed_boost_entry() -> (OrderedFloat<f32>, Box<EffectType>) {
        (
            OrderedFloat(1.0),
            Box::new(EffectType::SpeedBoost(
                crate::effect_v3::effects::SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                },
            )),
        )
    }

    #[test]
    fn reverse_all_by_source_removes_counter_via_default_delegation() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        make_config().fire(entity, "entropy_chip", &mut world);
        assert!(world.get::<EntropyCounter>(entity).is_some());

        make_config().reverse_all_by_source(entity, "entropy_chip", &mut world);
        assert!(
            world.get::<EntropyCounter>(entity).is_none(),
            "EntropyCounter should be removed by default delegation"
        );

        // Calling twice does not panic.
        make_config().reverse_all_by_source(entity, "entropy_chip", &mut world);
    }

    // ── Behavior 1: fire() inserts EntropyCounter with correct values ──

    #[test]
    fn fire_inserts_counter_with_correct_values() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let config = make_config_with_pool(3, vec![shockwave_entry()]);
        config.fire(entity, "entropy_chip", &mut world);

        let counter = world
            .get::<EntropyCounter>(entity)
            .expect("EntropyCounter should be inserted by fire()");
        assert_eq!(counter.count, 0, "count should be 0 after fire()");
        assert_eq!(
            counter.max_effects, 3,
            "max_effects should match config value"
        );
        assert_eq!(counter.pool.len(), 1, "pool should have 1 entry");
    }

    #[test]
    fn fire_inserts_counter_with_empty_pool() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let config = make_config_with_pool(5, vec![]);
        config.fire(entity, "entropy_chip", &mut world);

        let counter = world
            .get::<EntropyCounter>(entity)
            .expect("EntropyCounter should be inserted even with empty pool");
        assert_eq!(counter.count, 0, "count should be 0");
        assert_eq!(counter.max_effects, 5, "max_effects should be 5");
        assert!(counter.pool.is_empty(), "pool should be empty");
    }

    // ── Behavior 2: fire() on entity with existing counter overwrites ──

    #[test]
    fn fire_overwrites_existing_counter() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // Insert initial counter manually with count=2
        world.entity_mut(entity).insert(EntropyCounter {
            count:       2,
            max_effects: 3,
            pool:        vec![shockwave_entry()],
        });

        // Fire a new config with different values
        let new_config = make_config_with_pool(5, vec![speed_boost_entry()]);
        new_config.fire(entity, "entropy_chip", &mut world);

        let counter = world
            .get::<EntropyCounter>(entity)
            .expect("EntropyCounter should exist after fire()");
        assert_eq!(counter.count, 0, "count should reset to 0 on overwrite");
        assert_eq!(
            counter.max_effects, 5,
            "max_effects should be new config value"
        );
        assert_eq!(counter.pool.len(), 1, "pool should have new pool");
    }

    #[test]
    fn fire_same_config_twice_resets_count() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let config = make_config_with_pool(3, vec![shockwave_entry()]);
        config.fire(entity, "entropy_chip", &mut world);

        // Manually increment count to simulate usage
        world.get_mut::<EntropyCounter>(entity).unwrap().count = 2;

        // Fire again — should reset count to 0
        config.fire(entity, "entropy_chip", &mut world);

        let counter = world.get::<EntropyCounter>(entity).unwrap();
        assert_eq!(
            counter.count, 0,
            "second fire() should reset count to 0 even if it was incremented"
        );
    }

    // ── Behavior 3: fire() on despawned entity does not panic ──

    #[test]
    fn fire_on_despawned_entity_does_not_panic() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        world.despawn(entity);

        let config = make_config_with_pool(3, vec![shockwave_entry()]);
        // Should not panic
        config.fire(entity, "entropy_chip", &mut world);
    }

    // ── Behavior 4: reverse() removes EntropyCounter ──

    #[test]
    fn reverse_removes_counter() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        world.entity_mut(entity).insert(EntropyCounter {
            count:       2,
            max_effects: 3,
            pool:        vec![shockwave_entry()],
        });

        let config = make_config();
        config.reverse(entity, "entropy_chip", &mut world);

        assert!(
            world.get::<EntropyCounter>(entity).is_none(),
            "EntropyCounter should be removed by reverse()"
        );
    }

    // ── Behavior 5: reverse() on entity without counter does not panic ──

    #[test]
    fn reverse_on_entity_without_counter_does_not_panic() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let config = make_config();
        // Should not panic
        config.reverse(entity, "entropy_chip", &mut world);
    }

    // ── Behavior 6: reverse() on despawned entity does not panic ──

    #[test]
    fn reverse_on_despawned_entity_does_not_panic() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        world.despawn(entity);

        let config = make_config();
        // Should not panic
        config.reverse(entity, "entropy_chip", &mut world);
    }

    // ── Behavior 7: fire() then reverse() round-trips cleanly ──

    #[test]
    fn fire_then_reverse_round_trips() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let config = make_config_with_pool(3, vec![shockwave_entry()]);
        config.fire(entity, "src", &mut world);
        assert!(
            world.get::<EntropyCounter>(entity).is_some(),
            "EntropyCounter should exist after fire()"
        );

        config.reverse(entity, "src", &mut world);
        assert!(
            world.get::<EntropyCounter>(entity).is_none(),
            "EntropyCounter should be removed after reverse()"
        );
    }

    #[test]
    fn fire_reverse_fire_round_trips() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let config = make_config_with_pool(3, vec![shockwave_entry()]);

        config.fire(entity, "src", &mut world);
        config.reverse(entity, "src", &mut world);
        config.fire(entity, "src", &mut world);

        let counter = world
            .get::<EntropyCounter>(entity)
            .expect("EntropyCounter should exist after fire-reverse-fire");
        assert_eq!(
            counter.count, 0,
            "count should be 0 after fire-reverse-fire"
        );
    }
}
