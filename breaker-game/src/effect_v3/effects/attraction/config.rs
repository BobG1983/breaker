//! `AttractionConfig` — attraction steering toward entities.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use super::components::{ActiveAttractions, AttractionEntry};
use crate::effect_v3::{
    traits::{Fireable, Reversible},
    types::AttractionType,
};

/// Configuration for bolt attraction toward a target entity type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttractionConfig {
    /// Which entity type the bolt steers toward.
    pub attraction_type: AttractionType,
    /// Attraction strength per tick.
    pub force:           OrderedFloat<f32>,
    /// Optional cap on the per-tick steering delta (None = uncapped).
    pub max_force:       Option<OrderedFloat<f32>>,
}

impl Fireable for AttractionConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        if world.get_entity(entity).is_err() {
            return;
        }

        let entry = AttractionEntry {
            source:          source.to_owned(),
            attraction_type: self.attraction_type,
            force:           self.force.0,
            max_force:       self.max_force.map(|f| f.0),
        };

        if let Some(mut active) = world.get_mut::<ActiveAttractions>(entity) {
            active.0.push(entry);
        } else {
            world
                .entity_mut(entity)
                .insert(ActiveAttractions(vec![entry]));
        }
    }

    fn register(app: &mut App) {
        use super::systems::apply_attraction_forces;
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            apply_attraction_forces.in_set(EffectV3Systems::Tick),
        );
    }
}

impl Reversible for AttractionConfig {
    fn reverse(&self, entity: Entity, source: &str, world: &mut World) {
        if let Some(mut active) = world.get_mut::<ActiveAttractions>(entity)
            && let Some(idx) = active
                .0
                .iter()
                .position(|e| e.source == source && e.attraction_type == self.attraction_type)
        {
            active.0.remove(idx);
        }
    }

    fn reverse_all_by_source(&self, entity: Entity, source: &str, world: &mut World) {
        if let Some(mut active) = world.get_mut::<ActiveAttractions>(entity) {
            active.0.retain(|e| e.source != source);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{
        effects::attraction::components::ActiveAttractions,
        traits::{Fireable, Reversible},
        types::AttractionType,
    };

    // ── fire ──────────────────────────────────────────────────────────

    #[test]
    fn fire_creates_active_attractions_with_correct_fields() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        AttractionConfig {
            attraction_type: AttractionType::Cell,
            force:           OrderedFloat(100.0),
            max_force:       Some(OrderedFloat(200.0)),
        }
        .fire(entity, "magnet", &mut world);

        let active = world.get::<ActiveAttractions>(entity).unwrap();
        assert_eq!(active.0.len(), 1);
        assert_eq!(active.0[0].source, "magnet");
        assert_eq!(active.0[0].attraction_type, AttractionType::Cell);
        assert!((active.0[0].force - 100.0).abs() < f32::EPSILON);
        assert_eq!(active.0[0].max_force, Some(200.0));
    }

    #[test]
    fn fire_appends_to_existing_active_attractions() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        AttractionConfig {
            attraction_type: AttractionType::Cell,
            force:           OrderedFloat(100.0),
            max_force:       None,
        }
        .fire(entity, "magnet_a", &mut world);
        AttractionConfig {
            attraction_type: AttractionType::Breaker,
            force:           OrderedFloat(50.0),
            max_force:       Some(OrderedFloat(75.0)),
        }
        .fire(entity, "magnet_b", &mut world);

        let active = world.get::<ActiveAttractions>(entity).unwrap();
        assert_eq!(active.0.len(), 2);
        assert_eq!(active.0[0].source, "magnet_a");
        assert_eq!(active.0[1].source, "magnet_b");
    }

    #[test]
    fn fire_on_despawned_entity_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        world.despawn(entity);

        AttractionConfig {
            attraction_type: AttractionType::Cell,
            force:           OrderedFloat(100.0),
            max_force:       None,
        }
        .fire(entity, "magnet", &mut world);
        // No panic.
    }

    // ── reverse ──────────────────────────────────────────────────────

    #[test]
    fn reverse_removes_first_matching_entry_by_source_and_type() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        AttractionConfig {
            attraction_type: AttractionType::Cell,
            force:           OrderedFloat(100.0),
            max_force:       None,
        }
        .fire(entity, "magnet", &mut world);
        AttractionConfig {
            attraction_type: AttractionType::Breaker,
            force:           OrderedFloat(50.0),
            max_force:       None,
        }
        .fire(entity, "magnet", &mut world);

        AttractionConfig {
            attraction_type: AttractionType::Cell,
            force:           OrderedFloat(100.0),
            max_force:       None,
        }
        .reverse(entity, "magnet", &mut world);

        let active = world.get::<ActiveAttractions>(entity).unwrap();
        assert_eq!(active.0.len(), 1);
        assert_eq!(active.0[0].attraction_type, AttractionType::Breaker);
    }

    #[test]
    fn reverse_on_entity_without_active_attractions_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        AttractionConfig {
            attraction_type: AttractionType::Cell,
            force:           OrderedFloat(100.0),
            max_force:       None,
        }
        .reverse(entity, "magnet", &mut world);
        // No panic.
    }

    // ── reverse_all_by_source ─────────────────────────────────────────

    #[test]
    fn reverse_all_by_source_removes_all_entries_from_matching_source() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        AttractionConfig {
            attraction_type: AttractionType::Cell,
            force:           OrderedFloat(100.0),
            max_force:       None,
        }
        .fire(entity, "magnet", &mut world);
        AttractionConfig {
            attraction_type: AttractionType::Wall,
            force:           OrderedFloat(50.0),
            max_force:       Some(OrderedFloat(200.0)),
        }
        .fire(entity, "gravity_chip", &mut world);
        AttractionConfig {
            attraction_type: AttractionType::Breaker,
            force:           OrderedFloat(75.0),
            max_force:       None,
        }
        .fire(entity, "magnet", &mut world);

        AttractionConfig {
            attraction_type: AttractionType::Cell,
            force:           OrderedFloat(100.0),
            max_force:       None,
        }
        .reverse_all_by_source(entity, "magnet", &mut world);

        let active = world.get::<ActiveAttractions>(entity).unwrap();
        assert_eq!(active.0.len(), 1);
        assert_eq!(active.0[0].source, "gravity_chip");
        assert_eq!(active.0[0].attraction_type, AttractionType::Wall);
    }

    #[test]
    fn reverse_all_by_source_on_entity_without_active_attractions_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        AttractionConfig {
            attraction_type: AttractionType::Cell,
            force:           OrderedFloat(100.0),
            max_force:       None,
        }
        .reverse_all_by_source(entity, "magnet", &mut world);
        // No panic.
        assert!(world.get::<ActiveAttractions>(entity).is_none());
    }

    #[test]
    fn reverse_all_by_source_removes_all_entries_when_all_share_same_source() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        AttractionConfig {
            attraction_type: AttractionType::Cell,
            force:           OrderedFloat(100.0),
            max_force:       None,
        }
        .fire(entity, "magnet", &mut world);
        AttractionConfig {
            attraction_type: AttractionType::Breaker,
            force:           OrderedFloat(75.0),
            max_force:       None,
        }
        .fire(entity, "magnet", &mut world);

        AttractionConfig {
            attraction_type: AttractionType::Cell,
            force:           OrderedFloat(100.0),
            max_force:       None,
        }
        .reverse_all_by_source(entity, "magnet", &mut world);

        let active = world.get::<ActiveAttractions>(entity).unwrap();
        assert_eq!(active.0.len(), 0);
    }
}
