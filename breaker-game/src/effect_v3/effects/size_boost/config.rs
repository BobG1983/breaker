//! `SizeBoostConfig` — multiplicative passive size scaling.

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

/// Multiplicative size scaling factor applied to the entity's base size.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SizeBoostConfig {
    /// Multiplicative size scaling factor.
    pub multiplier: OrderedFloat<f32>,
}

impl Fireable for SizeBoostConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
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

impl Reversible for SizeBoostConfig {
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

impl PassiveEffect for SizeBoostConfig {
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

    /// Builder-format `SourceId` representing a generic "Augment" upgrade —
    /// used as the multi-source key in retain/remove tests.
    fn augment_source() -> SourceId {
        SourceId::chip("Augment").rarity(Rarity::Common).build()
    }

    #[test]
    fn fire_creates_stack_and_pushes_entry() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = SizeBoostConfig {
            multiplier: OrderedFloat(1.2),
        };

        config.fire(entity, test_source().0.as_ref(), &mut world);

        let stack = world.get::<EffectStack<SizeBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
    }

    // ── B52: fire stores SourceId-keyed entry ──

    #[test]
    fn fire_stores_source_as_source_id_value() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = SizeBoostConfig {
            multiplier: OrderedFloat(1.2),
        };

        let pulse_common = SourceId::chip("Pulse").rarity(Rarity::Common).build();
        config.fire(entity, pulse_common.0.as_ref(), &mut world);

        let stack = world.get::<EffectStack<SizeBoostConfig>>(entity).unwrap();
        let entries: Vec<_> = stack.iter().collect();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, pulse_common);
    }

    #[test]
    fn fire_multiple_times_stacks_entries() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = SizeBoostConfig {
            multiplier: OrderedFloat(1.2),
        };

        config.fire(entity, test_source().0.as_ref(), &mut world);
        config.fire(entity, test_source().0.as_ref(), &mut world);

        let stack = world.get::<EffectStack<SizeBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 2);
        assert!((stack.aggregate() - 1.44).abs() < 1e-5);
    }

    #[test]
    fn reverse_removes_matching_entry() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = SizeBoostConfig {
            multiplier: OrderedFloat(1.2),
        };

        config.fire(entity, test_source().0.as_ref(), &mut world);
        config.reverse(entity, test_source().0.as_ref(), &mut world);

        let stack = world.get::<EffectStack<SizeBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn reverse_on_entity_without_stack_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = SizeBoostConfig {
            multiplier: OrderedFloat(1.2),
        };

        config.reverse(entity, test_source().0.as_ref(), &mut world);
    }

    // ── reverse_all_by_source ─────────────────────────────────────────

    #[test]
    fn reverse_all_by_source_removes_all_entries_from_matching_source_leaves_others() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        SizeBoostConfig {
            multiplier: OrderedFloat(1.2),
        }
        .fire(entity, augment_source().0.as_ref(), &mut world);
        SizeBoostConfig {
            multiplier: OrderedFloat(1.3),
        }
        .fire(entity, "other", &mut world);
        SizeBoostConfig {
            multiplier: OrderedFloat(1.4),
        }
        .fire(entity, augment_source().0.as_ref(), &mut world);

        SizeBoostConfig {
            multiplier: OrderedFloat(1.2),
        }
        .reverse_all_by_source(entity, augment_source().0.as_ref(), &mut world);

        let stack = world.get::<EffectStack<SizeBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 1.3).abs() < 1e-5);
    }

    #[test]
    fn reverse_all_by_source_on_entity_without_stack_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        SizeBoostConfig {
            multiplier: OrderedFloat(1.2),
        }
        .reverse_all_by_source(entity, test_source().0.as_ref(), &mut world);
        // No panic.
        assert!(world.get::<EffectStack<SizeBoostConfig>>(entity).is_none());
    }
}
