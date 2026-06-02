//! `SpeedBoostConfig` — multiplicative passive speed scaling.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::{
    effect_v3::{
        stacking::EffectStack,
        traits::{Fireable, PassiveEffect, Reversible},
    },
    prelude::SourceId,
};

/// Multiplicative speed scaling factor applied to the entity's base speed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpeedBoostConfig {
    /// Multiplicative speed scaling factor.
    pub multiplier: OrderedFloat<f32>,
}

impl Fireable for SpeedBoostConfig {
    fn fire(
        &self,
        entity: Entity,
        source: &str,
        world: &mut World,
        _rng: &mut rand_chacha::ChaCha8Rng,
    ) {
        let has_stack = world.get::<EffectStack<Self>>(entity).is_some();
        if !has_stack {
            world
                .entity_mut(entity)
                .insert(EffectStack::<Self>::default());
        }
        if let Some(mut stack) = world.get_mut::<EffectStack<Self>>(entity) {
            let source_id = SourceId::from(source.to_owned());
            stack.push(source_id, self.clone());
        }
    }
}

impl Reversible for SpeedBoostConfig {
    fn reverse(&self, entity: Entity, source: &str, world: &mut World) {
        if let Some(mut stack) = world.get_mut::<EffectStack<Self>>(entity) {
            let source_id = SourceId::from(source.to_owned());
            stack.remove(&source_id, self);
        }
    }

    fn reverse_all_by_source(&self, entity: Entity, source: &str, world: &mut World) {
        if let Some(mut stack) = world.get_mut::<EffectStack<Self>>(entity) {
            let source_id = SourceId::from(source.to_owned());
            stack.retain_by_source(&source_id);
        }
    }
}

impl PassiveEffect for SpeedBoostConfig {
    fn aggregate(entries: &[(SourceId, Self)]) -> f32 {
        entries
            .iter()
            .map(|(_, c)| c.multiplier.into_inner())
            .product::<f32>()
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
        chips::definition::Rarity,
        effect_v3::{
            stacking::EffectStack,
            traits::{Fireable, Reversible},
        },
        prelude::SourceIdExt,
    };

    /// Builder-format `SourceId` used as the canonical opaque test fixture.
    fn test_source() -> SourceId {
        SourceId::chip("Test").rarity(Rarity::Common).build()
    }

    /// Builder-format `SourceId` representing an "Overclock" chip — used by
    /// multi-source retain/remove tests in this module.
    fn overclock_source() -> SourceId {
        SourceId::chip("Overclock").rarity(Rarity::Common).build()
    }

    /// Builder-format `SourceId` representing a "`FeedbackLoop`" chip — used by
    /// multi-source retain/remove tests in this module.
    fn feedback_loop_source() -> SourceId {
        SourceId::chip("FeedbackLoop")
            .rarity(Rarity::Common)
            .build()
    }

    #[test]
    fn fire_creates_stack_and_pushes_entry() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        };
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        config.fire(entity, test_source().0.as_ref(), &mut world, &mut rng);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
    }

    // ── B49: fire pushes a (SourceId, SpeedBoostConfig) entry ──

    #[test]
    fn fire_stores_source_as_source_id_value() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        };
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let piercing_common = SourceId::chip("Piercing").rarity(Rarity::Common).build();
        config.fire(entity, piercing_common.0.as_ref(), &mut world, &mut rng);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        let entries: Vec<_> = stack.iter().collect();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, piercing_common);
    }

    #[test]
    fn fire_multiple_times_stacks_entries() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        };
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        config.fire(entity, test_source().0.as_ref(), &mut world, &mut rng);
        config.fire(entity, test_source().0.as_ref(), &mut world, &mut rng);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 2);
        assert!((stack.aggregate() - 2.25).abs() < 1e-5);
    }

    #[test]
    fn reverse_removes_matching_entry() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        };
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        config.fire(entity, test_source().0.as_ref(), &mut world, &mut rng);
        config.reverse(entity, test_source().0.as_ref(), &mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn reverse_on_entity_without_stack_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        };

        config.reverse(entity, test_source().0.as_ref(), &mut world);
        // No panic — operation is a no-op.
    }

    // ── reverse_all_by_source ─────────────────────────────────────────

    #[test]
    fn reverse_all_by_source_removes_all_entries_from_matching_source_leaves_others() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }
        .fire(entity, overclock_source().0.as_ref(), &mut world, &mut rng);
        SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(
            entity,
            feedback_loop_source().0.as_ref(),
            &mut world,
            &mut rng,
        );
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.3),
        }
        .fire(entity, overclock_source().0.as_ref(), &mut world, &mut rng);

        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }
        .reverse_all_by_source(entity, overclock_source().0.as_ref(), &mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn reverse_all_by_source_on_entity_without_stack_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }
        .reverse_all_by_source(entity, test_source().0.as_ref(), &mut world);
        // No panic.
        assert!(world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none());
    }
}
