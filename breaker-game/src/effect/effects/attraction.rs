use bevy::prelude::*;

use crate::effect::core::AttractionType;

/// An individual attraction entry tracking type, force, and active state.
#[derive(Clone, Debug, PartialEq)]
pub struct AttractionEntry {
    /// Which entity type to attract toward.
    pub attraction_type: AttractionType,
    /// Attraction strength.
    pub force: f32,
    /// Whether this attraction is currently active (deactivates on hit).
    pub active: bool,
}

/// Component holding all active attractions on an entity.
#[derive(Component, Debug, Default, Clone)]
pub struct ActiveAttractions(pub Vec<AttractionEntry>);

/// Adds an attraction entry to the entity.
///
/// Inserts `ActiveAttractions` if not already present.
pub(crate) fn fire(entity: Entity, attraction_type: AttractionType, force: f32, world: &mut World) {
    let entry = AttractionEntry {
        attraction_type,
        force,
        active: true,
    };

    if let Some(mut attractions) = world.get_mut::<ActiveAttractions>(entity) {
        attractions.0.push(entry);
    } else if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
        entity_mut.insert(ActiveAttractions(vec![entry]));
    }
}

/// Removes a matching attraction entry from the entity.
pub(crate) fn reverse(
    entity: Entity,
    attraction_type: AttractionType,
    force: f32,
    world: &mut World,
) {
    if let Some(mut attractions) = world.get_mut::<ActiveAttractions>(entity)
        && let Some(idx) = attractions.0.iter().position(|e| {
            e.attraction_type == attraction_type && (e.force - force).abs() < f32::EPSILON
        })
    {
        attractions.0.remove(idx);
    }
}

/// Placeholder — steers entity toward nearest target of the attracted type.
fn apply_attraction() {
    // Will query ActiveAttractions + Transform, find nearest target, apply force.
}

/// Placeholder — deactivates attraction on hit, reactivates on bounce off
/// non-attracted type.
fn manage_attraction_types() {
    // Will query ActiveAttractions + collision events.
}

/// Registers attraction systems in `FixedUpdate`.
pub(crate) fn register(app: &mut App) {
    app.add_systems(FixedUpdate, (apply_attraction, manage_attraction_types));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_inserts_active_attractions_on_fresh_entity() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        fire(entity, AttractionType::Cell, 10.0, &mut world);

        let attractions = world.get::<ActiveAttractions>(entity).unwrap();
        assert_eq!(attractions.0.len(), 1);
        assert_eq!(attractions.0[0].attraction_type, AttractionType::Cell);
        assert!((attractions.0[0].force - 10.0).abs() < f32::EPSILON);
        assert!(attractions.0[0].active);
    }

    #[test]
    fn fire_appends_entry_to_existing_active_attractions() {
        let mut world = World::new();
        let entity = world
            .spawn(ActiveAttractions(vec![AttractionEntry {
                attraction_type: AttractionType::Wall,
                force: 5.0,
                active: true,
            }]))
            .id();

        fire(entity, AttractionType::Breaker, 15.0, &mut world);

        let attractions = world.get::<ActiveAttractions>(entity).unwrap();
        assert_eq!(
            attractions.0.len(),
            2,
            "should have two entries after appending"
        );
        assert_eq!(attractions.0[1].attraction_type, AttractionType::Breaker);
        assert!((attractions.0[1].force - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn reverse_removes_matching_entry() {
        let mut world = World::new();
        let entity = world
            .spawn(ActiveAttractions(vec![
                AttractionEntry {
                    attraction_type: AttractionType::Cell,
                    force: 10.0,
                    active: true,
                },
                AttractionEntry {
                    attraction_type: AttractionType::Wall,
                    force: 5.0,
                    active: true,
                },
            ]))
            .id();

        reverse(entity, AttractionType::Cell, 10.0, &mut world);

        let attractions = world.get::<ActiveAttractions>(entity).unwrap();
        assert_eq!(attractions.0.len(), 1, "matching entry should be removed");
        assert_eq!(attractions.0[0].attraction_type, AttractionType::Wall);
    }

    #[test]
    fn reverse_with_no_match_is_noop() {
        let mut world = World::new();
        let entity = world
            .spawn(ActiveAttractions(vec![AttractionEntry {
                attraction_type: AttractionType::Cell,
                force: 10.0,
                active: true,
            }]))
            .id();

        // Different type — no match.
        reverse(entity, AttractionType::Breaker, 10.0, &mut world);

        let attractions = world.get::<ActiveAttractions>(entity).unwrap();
        assert_eq!(
            attractions.0.len(),
            1,
            "no entry should be removed when no match"
        );
    }
}
