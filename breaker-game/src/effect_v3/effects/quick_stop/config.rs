//! `QuickStopConfig` — multiplicative passive breaker deceleration.

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

/// Breaker deceleration multiplier — higher values make the breaker stop faster.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuickStopConfig {
    /// Breaker deceleration multiplier.
    pub multiplier: OrderedFloat<f32>,
}

impl Fireable for QuickStopConfig {
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

impl Reversible for QuickStopConfig {
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

impl PassiveEffect for QuickStopConfig {
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

    /// Builder-format `SourceId` representing a "`ChronoPassive`" chip — used
    /// by multi-source retain/remove tests in this module.
    fn chrono_passive_source() -> SourceId {
        SourceId::chip("ChronoPassive")
            .rarity(Rarity::Common)
            .build()
    }

    #[test]
    fn fire_creates_stack_and_pushes_entry() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = QuickStopConfig {
            multiplier: OrderedFloat(2.0),
        };
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        config.fire(entity, test_source().0.as_ref(), &mut world, &mut rng);

        let stack = world.get::<EffectStack<QuickStopConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
    }

    // ── B53: fire stores SourceId-keyed entry ──

    #[test]
    fn fire_stores_source_as_source_id_value() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = QuickStopConfig {
            multiplier: OrderedFloat(2.0),
        };
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let quickstop = SourceId::chip("QuickStop").rarity(Rarity::Common).build();
        config.fire(entity, quickstop.0.as_ref(), &mut world, &mut rng);

        let stack = world.get::<EffectStack<QuickStopConfig>>(entity).unwrap();
        let entries: Vec<_> = stack.iter().collect();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, quickstop);
    }

    #[test]
    fn fire_multiple_times_stacks_entries() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = QuickStopConfig {
            multiplier: OrderedFloat(2.0),
        };
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        config.fire(entity, test_source().0.as_ref(), &mut world, &mut rng);
        config.fire(entity, test_source().0.as_ref(), &mut world, &mut rng);

        let stack = world.get::<EffectStack<QuickStopConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 2);
        assert!((stack.aggregate() - 4.0).abs() < 1e-5);
    }

    #[test]
    fn reverse_removes_matching_entry() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = QuickStopConfig {
            multiplier: OrderedFloat(2.0),
        };
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        config.fire(entity, test_source().0.as_ref(), &mut world, &mut rng);
        config.reverse(entity, test_source().0.as_ref(), &mut world);

        let stack = world.get::<EffectStack<QuickStopConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn reverse_on_entity_without_stack_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = QuickStopConfig {
            multiplier: OrderedFloat(2.0),
        };

        config.reverse(entity, test_source().0.as_ref(), &mut world);
    }

    // ── reverse_all_by_source ─────────────────────────────────────────

    #[test]
    fn reverse_all_by_source_removes_all_entries_from_matching_source_leaves_others() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        QuickStopConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(
            entity,
            chrono_passive_source().0.as_ref(),
            &mut world,
            &mut rng,
        );
        QuickStopConfig {
            multiplier: OrderedFloat(1.5),
        }
        .fire(entity, "other_chip", &mut world, &mut rng);
        QuickStopConfig {
            multiplier: OrderedFloat(3.0),
        }
        .fire(
            entity,
            chrono_passive_source().0.as_ref(),
            &mut world,
            &mut rng,
        );

        QuickStopConfig {
            multiplier: OrderedFloat(2.0),
        }
        .reverse_all_by_source(entity, chrono_passive_source().0.as_ref(), &mut world);

        let stack = world.get::<EffectStack<QuickStopConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 1.5).abs() < 1e-5);
    }

    #[test]
    fn reverse_all_by_source_on_entity_without_stack_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        QuickStopConfig {
            multiplier: OrderedFloat(2.0),
        }
        .reverse_all_by_source(entity, chrono_passive_source().0.as_ref(), &mut world);
        // No panic.
    }
}
