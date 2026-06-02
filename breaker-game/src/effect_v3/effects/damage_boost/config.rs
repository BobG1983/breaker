//! `DamageBoostConfig` — multiplicative passive damage scaling.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::{
    effect_v3::traits::{Fireable, Reversible},
    prelude::{DamageBoostStack, SourceId},
};

/// Multiplicative damage scaling factor applied to the entity's base damage.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DamageBoostConfig {
    /// Multiplicative damage scaling factor.
    pub multiplier: OrderedFloat<f32>,
}

impl Fireable for DamageBoostConfig {
    fn fire(
        &self,
        entity: Entity,
        source: &str,
        world: &mut World,
        _rng: &mut rand_chacha::ChaCha8Rng,
    ) {
        if world.get::<DamageBoostStack>(entity).is_none() {
            world.entity_mut(entity).insert(DamageBoostStack::default());
        }
        if let Some(mut stack) = world.get_mut::<DamageBoostStack>(entity) {
            stack.add(
                SourceId::from(source.to_owned()),
                self.multiplier.into_inner(),
            );
        }
    }
}

impl Reversible for DamageBoostConfig {
    fn reverse(&self, entity: Entity, source: &str, world: &mut World) {
        if let Some(mut stack) = world.get_mut::<DamageBoostStack>(entity) {
            stack.remove_by_source(&SourceId::from(source.to_owned()));
        }
    }

    fn reverse_all_by_source(&self, entity: Entity, source: &str, world: &mut World) {
        if let Some(mut stack) = world.get_mut::<DamageBoostStack>(entity) {
            stack.remove_by_source(&SourceId::from(source.to_owned()));
        }
    }
}

// `impl PassiveEffect for DamageBoostConfig` DELETED — `DamageBoostConfig` no
// longer participates in the generic `EffectStack<T>` aggregate path. The
// crate-owned `DamageBoostStack` handles aggregation in-place via
// `aggregate_persistent()`.

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use super::*;
    use crate::{
        chips::definition::Rarity,
        effect_v3::traits::{Fireable, Reversible},
        prelude::{DamageBoostStack, SourceIdExt},
    };

    /// Builder-format `SourceId` used as the canonical opaque test fixture.
    fn test_source() -> SourceId {
        SourceId::chip("Test").rarity(Rarity::Common).build()
    }

    /// Builder-format `SourceId` representing an "Amp" damage-boost chip.
    fn amp_source() -> SourceId {
        SourceId::chip("Amp").rarity(Rarity::Common).build()
    }

    /// Builder-format `SourceId` representing a "Loop" damage-boost chip
    /// (alternate source used for multi-source aggregation tests).
    fn loop_source() -> SourceId {
        SourceId::chip("Loop").rarity(Rarity::Common).build()
    }

    /// Builder-format `SourceId` representing a "Boom" chip used by tests
    /// that need a third distinct source.
    fn boom_source() -> SourceId {
        SourceId::chip("Boom").rarity(Rarity::Common).build()
    }

    // ── Behavior 1: `fire` inserts `DamageBoostStack` on a fresh entity ──

    #[test]
    fn fire_creates_stack_and_pushes_entry() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        };
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        config.fire(entity, amp_source().0.as_ref(), &mut world, &mut rng);

        let stack = world
            .get::<DamageBoostStack>(entity)
            .expect("DamageBoostStack should be inserted by fire");
        assert!(!stack.is_empty());
        assert!((stack.aggregate_persistent(None) - 2.0).abs() <= f32::EPSILON);
    }

    #[test]
    fn fire_twice_on_same_entity_does_not_insert_second_stack_component() {
        // Edge case for Behavior 1: firing a second config with a different
        // source must append to the existing stack, not insert a new component.
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(entity, amp_source().0.as_ref(), &mut world, &mut rng);
        DamageBoostConfig {
            multiplier: OrderedFloat(1.5),
        }
        .fire(entity, loop_source().0.as_ref(), &mut world, &mut rng);

        let stack = world.get::<DamageBoostStack>(entity).unwrap();
        assert!((stack.aggregate_persistent(None) - 3.0).abs() <= f32::EPSILON);
    }

    // ── Behavior 2: `fire` twice same source multiplies aggregate ──

    #[test]
    fn fire_multiple_times_stacks_entries() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        };
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        config.fire(entity, test_source().0.as_ref(), &mut world, &mut rng);
        config.fire(entity, test_source().0.as_ref(), &mut world, &mut rng);

        let stack = world.get::<DamageBoostStack>(entity).unwrap();
        assert!((stack.aggregate_persistent(None) - 4.0).abs() < 1e-5);
    }

    #[test]
    fn fire_five_times_same_source_pow_five() {
        // Edge case for Behavior 2: five fires with multiplier 2.0 must
        // produce 2^5 = 32.0 — confirms Vec-not-HashMap semantic.
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        };
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        for _ in 0..5 {
            config.fire(entity, test_source().0.as_ref(), &mut world, &mut rng);
        }

        let stack = world.get::<DamageBoostStack>(entity).unwrap();
        assert!((stack.aggregate_persistent(None) - 32.0).abs() < 1e-5);
    }

    // ── Behavior 3.a: `reverse(entity, source)` removes entries by source ──

    #[test]
    fn reverse_removes_matching_entry() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        };
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        config.fire(entity, amp_source().0.as_ref(), &mut world, &mut rng);
        config.reverse(entity, amp_source().0.as_ref(), &mut world);

        let stack = world.get::<DamageBoostStack>(entity).unwrap();
        assert!(stack.is_empty());
        assert!((stack.aggregate_persistent(None) - 1.0).abs() <= f32::EPSILON);
    }

    // ── Behavior 3.b: NEW — `reverse` removes EVERY entry with that source ──

    #[test]
    fn reverse_removes_all_entries_sharing_source_not_just_one() {
        // NEW regression-lock: pre-W3, `reverse` removed exactly ONE entry
        // matching `(source, config)`. After W3, `reverse` collapses to
        // `remove_by_source` — it removes EVERY entry with that source,
        // regardless of multiplier.
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(entity, amp_source().0.as_ref(), &mut world, &mut rng);
        DamageBoostConfig {
            multiplier: OrderedFloat(3.0),
        }
        .fire(entity, amp_source().0.as_ref(), &mut world, &mut rng);
        DamageBoostConfig {
            multiplier: OrderedFloat(4.0),
        }
        .fire(entity, amp_source().0.as_ref(), &mut world, &mut rng);

        // Reverse with a config whose multiplier (2.0) matches only one of
        // the three entries by value — but because reverse is now
        // by-source-only, all three must be removed.
        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .reverse(entity, amp_source().0.as_ref(), &mut world);

        let stack = world.get::<DamageBoostStack>(entity).unwrap();
        assert!(stack.is_empty());
        assert!((stack.aggregate_persistent(None) - 1.0).abs() <= f32::EPSILON);
    }

    #[test]
    fn reverse_leaves_entries_of_other_sources_intact() {
        // Edge case for Behavior 3.b: mixed-source stack, reverse only one
        // source, assert the other source's entry is untouched.
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(entity, amp_source().0.as_ref(), &mut world, &mut rng);
        DamageBoostConfig {
            multiplier: OrderedFloat(1.5),
        }
        .fire(entity, loop_source().0.as_ref(), &mut world, &mut rng);

        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .reverse(entity, amp_source().0.as_ref(), &mut world);

        let stack = world.get::<DamageBoostStack>(entity).unwrap();
        assert!((stack.aggregate_persistent(None) - 1.5).abs() <= f32::EPSILON);
    }

    // ── Behavior 4: `reverse` on a stackless entity is a silent no-op ──

    #[test]
    fn reverse_on_entity_without_stack_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        };

        config.reverse(entity, test_source().0.as_ref(), &mut world);

        assert!(
            world.get::<DamageBoostStack>(entity).is_none(),
            "reverse must NOT insert the stack just to remove from it"
        );
    }

    #[test]
    fn reverse_twice_in_a_row_on_empty_entity_does_not_panic() {
        // Edge case for Behavior 4.
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        };

        config.reverse(entity, test_source().0.as_ref(), &mut world);
        config.reverse(entity, test_source().0.as_ref(), &mut world);

        assert!(world.get::<DamageBoostStack>(entity).is_none());
    }

    // ── Behavior 5: `reverse_all_by_source` — same as reverse post-W3 ──

    #[test]
    fn reverse_all_by_source_removes_all_entries_from_matching_source_leaves_others() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(entity, amp_source().0.as_ref(), &mut world, &mut rng);
        DamageBoostConfig {
            multiplier: OrderedFloat(1.5),
        }
        .fire(entity, "feedback_loop", &mut world, &mut rng);
        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(entity, amp_source().0.as_ref(), &mut world, &mut rng);

        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .reverse_all_by_source(entity, amp_source().0.as_ref(), &mut world);

        let stack = world.get::<DamageBoostStack>(entity).unwrap();
        assert!((stack.aggregate_persistent(None) - 1.5).abs() < 1e-5);
    }

    #[test]
    fn reverse_all_by_source_for_unknown_source_is_noop() {
        // Edge case for Behavior 5: reverse_all_by_source for a source that
        // was never fired is a no-op — aggregate unchanged.
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(entity, amp_source().0.as_ref(), &mut world, &mut rng);

        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .reverse_all_by_source(entity, "nonexistent", &mut world);

        let stack = world.get::<DamageBoostStack>(entity).unwrap();
        assert!((stack.aggregate_persistent(None) - 2.0).abs() < 1e-5);
    }

    // ── Behavior 6: `reverse_all_by_source` on a stackless entity is noop ──

    #[test]
    fn reverse_all_by_source_on_entity_without_stack_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .reverse_all_by_source(entity, amp_source().0.as_ref(), &mut world);

        assert!(world.get::<DamageBoostStack>(entity).is_none());
    }

    // ── Behavior 7: NEW — `reverse` ≡ `reverse_all_by_source` after W3 ──

    #[test]
    fn reverse_equals_reverse_all_by_source_after_w3() {
        // Regression-lock: both methods must produce identical state after
        // W3. This blocks any future reviewer from "restoring" the old
        // by-config-value semantic to `reverse`.
        let mut world_a = World::new();
        let entity_a = world_a.spawn_empty().id();
        let mut rng_a = ChaCha8Rng::seed_from_u64(42);
        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(entity_a, amp_source().0.as_ref(), &mut world_a, &mut rng_a);
        DamageBoostConfig {
            multiplier: OrderedFloat(3.0),
        }
        .fire(entity_a, amp_source().0.as_ref(), &mut world_a, &mut rng_a);
        DamageBoostConfig {
            multiplier: OrderedFloat(1.5),
        }
        .fire(entity_a, boom_source().0.as_ref(), &mut world_a, &mut rng_a);

        let mut world_b = World::new();
        let entity_b = world_b.spawn_empty().id();
        let mut rng_b = ChaCha8Rng::seed_from_u64(42);
        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(entity_b, amp_source().0.as_ref(), &mut world_b, &mut rng_b);
        DamageBoostConfig {
            multiplier: OrderedFloat(3.0),
        }
        .fire(entity_b, amp_source().0.as_ref(), &mut world_b, &mut rng_b);
        DamageBoostConfig {
            multiplier: OrderedFloat(1.5),
        }
        .fire(entity_b, boom_source().0.as_ref(), &mut world_b, &mut rng_b);

        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .reverse(entity_a, amp_source().0.as_ref(), &mut world_a);
        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .reverse_all_by_source(entity_b, amp_source().0.as_ref(), &mut world_b);

        let stack_a = world_a.get::<DamageBoostStack>(entity_a).unwrap();
        let stack_b = world_b.get::<DamageBoostStack>(entity_b).unwrap();
        assert!((stack_a.aggregate_persistent(None) - 1.5).abs() < 1e-5);
        assert!((stack_b.aggregate_persistent(None) - 1.5).abs() < 1e-5);
        assert_eq!(stack_a.is_empty(), stack_b.is_empty());
    }
}

#[cfg(test)]
mod negative_contracts {
    // ── Behavior 55: NEW — `DamageBoostConfig` no longer implements PassiveEffect ──
    //
    // Commented-out negative compile-time contract. Uncommenting the line
    // below must cause the file to fail to compile after W3 — production
    // code must NOT add back `impl PassiveEffect for DamageBoostConfig`.
    // Mirror of the pattern at
    // `rantzsoft_dmg/src/components/damage_boost_stack.rs:11-16`.
    //
    // #[test]
    // fn damage_boost_config_must_not_impl_passive_effect() {
    //     let _ = <super::DamageBoostConfig as crate::effect_v3::traits::PassiveEffect>::aggregate(&[]);
    // }
}
