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

    // ── fire idempotent stack insert ──────────────────────────────────

    #[test]
    fn fire_on_entity_with_existing_stack_appends_without_replacing() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // Pre-populate with 1 entry from "splinter".
        PiercingConfig { charges: 2 }.fire(entity, "splinter", &mut world);

        // Fire from a different source.
        PiercingConfig { charges: 5 }.fire(entity, "piercing_bolt", &mut world);

        let stack = world.get::<EffectStack<PiercingConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 2, "stack should have 2 entries, not replace");
        assert!((stack.aggregate() - 7.0).abs() < 1e-5);
    }

    #[test]
    fn fire_same_source_again_appends_third_entry() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        PiercingConfig { charges: 2 }.fire(entity, "splinter", &mut world);
        PiercingConfig { charges: 5 }.fire(entity, "piercing_bolt", &mut world);
        PiercingConfig { charges: 5 }.fire(entity, "piercing_bolt", &mut world);

        let stack = world.get::<EffectStack<PiercingConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 3, "stack should have 3 entries (2+5+5)");
        assert!((stack.aggregate() - 12.0).abs() < 1e-5);
    }

    // ── aggregate mixed sources ───────────────────────────────────────

    #[test]
    fn aggregate_sums_charges_from_mixed_sources() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        PiercingConfig { charges: 1 }.fire(entity, "chip_a", &mut world);
        PiercingConfig { charges: 4 }.fire(entity, "chip_b", &mut world);
        PiercingConfig { charges: 2 }.fire(entity, "chip_a", &mut world);

        let stack = world.get::<EffectStack<PiercingConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 3);
        assert!((stack.aggregate() - 7.0).abs() < 1e-5);
    }

    #[test]
    fn reverse_one_entry_from_mixed_sources_updates_aggregate() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        PiercingConfig { charges: 1 }.fire(entity, "chip_a", &mut world);
        PiercingConfig { charges: 4 }.fire(entity, "chip_b", &mut world);
        PiercingConfig { charges: 2 }.fire(entity, "chip_a", &mut world);

        // Reverse the first chip_a entry (charges: 1).
        PiercingConfig { charges: 1 }.reverse(entity, "chip_a", &mut world);

        let stack = world.get::<EffectStack<PiercingConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 2);
        assert!((stack.aggregate() - 6.0).abs() < 1e-5);
    }

    // ── reverse with non-empty stack preserves remaining ──────────────

    #[test]
    fn reverse_preserves_remaining_entries_in_stack() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        PiercingConfig { charges: 3 }.fire(entity, "a", &mut world);
        PiercingConfig { charges: 5 }.fire(entity, "b", &mut world);
        PiercingConfig { charges: 3 }.fire(entity, "a", &mut world);

        PiercingConfig { charges: 3 }.reverse(entity, "a", &mut world);

        let stack = world.get::<EffectStack<PiercingConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 2);
        assert!((stack.aggregate() - 8.0).abs() < 1e-5);
    }

    #[test]
    fn reverse_twice_leaves_single_entry() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        PiercingConfig { charges: 3 }.fire(entity, "a", &mut world);
        PiercingConfig { charges: 5 }.fire(entity, "b", &mut world);
        PiercingConfig { charges: 3 }.fire(entity, "a", &mut world);

        PiercingConfig { charges: 3 }.reverse(entity, "a", &mut world);
        PiercingConfig { charges: 3 }.reverse(entity, "a", &mut world);

        let stack = world.get::<EffectStack<PiercingConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 5.0).abs() < 1e-5);
    }
}
