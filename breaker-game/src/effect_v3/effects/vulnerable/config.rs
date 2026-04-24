//! `VulnerableConfig` — multiplicative passive incoming damage scaling.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::{
    effect_v3::traits::{Fireable, Reversible},
    prelude::{SourceId, VulnerableStack},
};

/// Incoming damage multiplier — values above 1.0 increase damage taken.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VulnerableConfig {
    /// Incoming damage multiplier.
    pub multiplier: OrderedFloat<f32>,
}

impl Fireable for VulnerableConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        if world.get::<VulnerableStack>(entity).is_none() {
            world.entity_mut(entity).insert(VulnerableStack::default());
        }
        if let Some(mut stack) = world.get_mut::<VulnerableStack>(entity) {
            stack.add(
                SourceId::from(source.to_owned()),
                self.multiplier.into_inner(),
            );
        }
    }
}

impl Reversible for VulnerableConfig {
    fn reverse(&self, entity: Entity, source: &str, world: &mut World) {
        if let Some(mut stack) = world.get_mut::<VulnerableStack>(entity) {
            stack.remove_by_source(&SourceId::from(source.to_owned()));
        }
    }

    fn reverse_all_by_source(&self, entity: Entity, source: &str, world: &mut World) {
        if let Some(mut stack) = world.get_mut::<VulnerableStack>(entity) {
            stack.remove_by_source(&SourceId::from(source.to_owned()));
        }
    }
}

// `impl PassiveEffect for VulnerableConfig` DELETED — `VulnerableConfig` no
// longer participates in the generic `EffectStack<T>` aggregate path. The
// crate-owned `VulnerableStack` handles aggregation in-place via
// `aggregate_persistent()`.

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::{
        effect_v3::traits::{Fireable, Reversible},
        prelude::VulnerableStack,
    };

    // ── Behavior 8: `fire` inserts `VulnerableStack` on a fresh entity ──

    #[test]
    fn fire_creates_stack_and_pushes_entry() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        };

        config.fire(entity, "decay", &mut world);

        let stack = world
            .get::<VulnerableStack>(entity)
            .expect("VulnerableStack should be inserted by fire");
        assert!(!stack.is_empty());
        assert!((stack.aggregate_persistent() - 1.5).abs() <= f32::EPSILON);
    }

    #[test]
    fn fire_twice_on_same_entity_does_not_insert_second_stack_component() {
        // Edge case for Behavior 8.
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        }
        .fire(entity, "decay", &mut world);
        VulnerableConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(entity, "vulnerable_again", &mut world);

        let stack = world.get::<VulnerableStack>(entity).unwrap();
        assert!((stack.aggregate_persistent() - 3.0).abs() <= f32::EPSILON);
    }

    // ── Behavior 9: `fire` twice same source multiplies aggregate ──

    #[test]
    fn fire_multiple_times_stacks_entries() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        };

        config.fire(entity, "test_source", &mut world);
        config.fire(entity, "test_source", &mut world);

        let stack = world.get::<VulnerableStack>(entity).unwrap();
        assert!((stack.aggregate_persistent() - 2.25).abs() < 1e-5);
    }

    // ── Behavior 10.a: `reverse(source)` removes entries by source ──

    #[test]
    fn reverse_removes_matching_entry() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        };

        config.fire(entity, "decay", &mut world);
        config.reverse(entity, "decay", &mut world);

        let stack = world.get::<VulnerableStack>(entity).unwrap();
        assert!(stack.is_empty());
        assert!((stack.aggregate_persistent() - 1.0).abs() <= f32::EPSILON);
    }

    // ── Behavior 10.b: NEW — `reverse` removes EVERY entry with that source ──

    #[test]
    fn reverse_removes_all_entries_sharing_source_not_just_one() {
        // NEW regression-lock for the semantic change.
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        }
        .fire(entity, "decay", &mut world);
        VulnerableConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(entity, "decay", &mut world);
        VulnerableConfig {
            multiplier: OrderedFloat(3.0),
        }
        .fire(entity, "decay", &mut world);

        VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        }
        .reverse(entity, "decay", &mut world);

        let stack = world.get::<VulnerableStack>(entity).unwrap();
        assert!(stack.is_empty());
        assert!((stack.aggregate_persistent() - 1.0).abs() <= f32::EPSILON);
    }

    // ── Behavior 11: `reverse` on a stackless entity is a silent no-op ──

    #[test]
    fn reverse_on_entity_without_stack_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        };

        config.reverse(entity, "test_source", &mut world);

        assert!(world.get::<VulnerableStack>(entity).is_none());
    }

    // ── Behavior 12: `reverse_all_by_source` ────────────────────────────

    #[test]
    fn reverse_all_by_source_removes_all_entries_from_matching_source_leaves_others() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        }
        .fire(entity, "decay", &mut world);
        VulnerableConfig {
            multiplier: OrderedFloat(0.5),
        }
        .fire(entity, "shield_effect", &mut world);
        VulnerableConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(entity, "decay", &mut world);

        VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        }
        .reverse_all_by_source(entity, "decay", &mut world);

        let stack = world.get::<VulnerableStack>(entity).unwrap();
        assert!((stack.aggregate_persistent() - 0.5).abs() < 1e-5);
    }

    // ── Behavior 13: `reverse_all_by_source` on a stackless entity is noop ──

    #[test]
    fn reverse_all_by_source_on_entity_without_stack_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        }
        .reverse_all_by_source(entity, "decay", &mut world);

        assert!(world.get::<VulnerableStack>(entity).is_none());
    }

    // ── Behavior 14: NEW — `reverse` ≡ `reverse_all_by_source` after W3 ──

    #[test]
    fn reverse_equals_reverse_all_by_source_after_w3() {
        let mut world_a = World::new();
        let entity_a = world_a.spawn_empty().id();
        VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        }
        .fire(entity_a, "decay", &mut world_a);
        VulnerableConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(entity_a, "decay", &mut world_a);
        VulnerableConfig {
            multiplier: OrderedFloat(0.5),
        }
        .fire(entity_a, "shield_effect", &mut world_a);

        let mut world_b = World::new();
        let entity_b = world_b.spawn_empty().id();
        VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        }
        .fire(entity_b, "decay", &mut world_b);
        VulnerableConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(entity_b, "decay", &mut world_b);
        VulnerableConfig {
            multiplier: OrderedFloat(0.5),
        }
        .fire(entity_b, "shield_effect", &mut world_b);

        VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        }
        .reverse(entity_a, "decay", &mut world_a);
        VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        }
        .reverse_all_by_source(entity_b, "decay", &mut world_b);

        let stack_a = world_a.get::<VulnerableStack>(entity_a).unwrap();
        let stack_b = world_b.get::<VulnerableStack>(entity_b).unwrap();
        assert!((stack_a.aggregate_persistent() - 0.5).abs() < 1e-5);
        assert!((stack_b.aggregate_persistent() - 0.5).abs() < 1e-5);
        assert_eq!(stack_a.is_empty(), stack_b.is_empty());
    }
}

#[cfg(test)]
mod negative_contracts {
    // ── Behavior 55 (mirror): NEW — `VulnerableConfig` no longer implements PassiveEffect ──
    //
    // Commented-out negative compile-time contract. Uncommenting the line
    // below must cause the file to fail to compile after W3. Mirror of the
    // pattern at `rantzsoft_dmg/src/components/damage_boost_stack.rs:11-16`.
    //
    // #[test]
    // fn vulnerable_config_must_not_impl_passive_effect() {
    //     let _ = <super::VulnerableConfig as crate::effect_v3::traits::PassiveEffect>::aggregate(&[]);
    // }
}
