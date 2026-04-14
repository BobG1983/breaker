//! `PiercingConfig` — additive passive piercing charges.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::effect_v3::{
    stacking::EffectStack,
    traits::{Fireable, PassiveEffect, Reversible},
};

/// Number of cells the bolt can pass through without bouncing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PiercingConfig {
    /// Number of piercing charges granted.
    pub charges: u32,
}

impl Fireable for PiercingConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        let has_stack = world.get::<EffectStack<Self>>(entity).is_some();
        if !has_stack {
            world
                .entity_mut(entity)
                .insert(EffectStack::<Self>::default());
        }
        if let Some(mut stack) = world.get_mut::<EffectStack<Self>>(entity) {
            stack.push(source.to_owned(), self.clone());
        }
    }
}

impl Reversible for PiercingConfig {
    fn reverse(&self, entity: Entity, source: &str, world: &mut World) {
        if let Some(mut stack) = world.get_mut::<EffectStack<Self>>(entity) {
            stack.remove(source, self);
        }
    }

    fn reverse_all_by_source(&self, entity: Entity, source: &str, world: &mut World) {
        if let Some(mut stack) = world.get_mut::<EffectStack<Self>>(entity) {
            stack.retain_by_source(source);
        }
    }
}

impl PassiveEffect for PiercingConfig {
    fn aggregate(entries: &[(String, Self)]) -> f32 {
        entries.iter().map(|(_, c)| c.charges as f32).sum()
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;
    use crate::effect_v3::{
        stacking::EffectStack,
        traits::{Fireable, Reversible},
    };

    #[test]
    fn fire_creates_stack_and_pushes_entry() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = PiercingConfig { charges: 3 };

        config.fire(entity, "test_source", &mut world);

        let stack = world.get::<EffectStack<PiercingConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn fire_multiple_times_stacks_entries() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = PiercingConfig { charges: 3 };

        config.fire(entity, "test_source", &mut world);
        config.fire(entity, "test_source", &mut world);

        let stack = world.get::<EffectStack<PiercingConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 2);
        assert!((stack.aggregate() - 6.0).abs() < 1e-5);
    }

    #[test]
    fn reverse_removes_matching_entry() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = PiercingConfig { charges: 3 };

        config.fire(entity, "test_source", &mut world);
        config.reverse(entity, "test_source", &mut world);

        let stack = world.get::<EffectStack<PiercingConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn reverse_on_entity_without_stack_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = PiercingConfig { charges: 3 };

        config.reverse(entity, "test_source", &mut world);
    }

    // ── reverse_all_by_source ─────────────────────────────────────────

    #[test]
    fn reverse_all_by_source_removes_all_entries_from_matching_source_leaves_others() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        PiercingConfig { charges: 2 }.fire(entity, "splinter", &mut world);
        PiercingConfig { charges: 5 }.fire(entity, "piercing_bolt", &mut world);
        PiercingConfig { charges: 3 }.fire(entity, "splinter", &mut world);

        PiercingConfig { charges: 2 }.reverse_all_by_source(entity, "splinter", &mut world);

        let stack = world.get::<EffectStack<PiercingConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn reverse_all_by_source_on_entity_without_stack_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        PiercingConfig { charges: 2 }.reverse_all_by_source(entity, "splinter", &mut world);
        // No panic.
    }
}
