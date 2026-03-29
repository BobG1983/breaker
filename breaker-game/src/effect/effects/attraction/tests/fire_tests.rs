use bevy::prelude::*;

use super::super::effect::*;
use crate::effect::core::AttractionType;

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

    // Different type -- no match.
    reverse(entity, AttractionType::Breaker, 10.0, &mut world);

    let attractions = world.get::<ActiveAttractions>(entity).unwrap();
    assert_eq!(
        attractions.0.len(),
        1,
        "no entry should be removed when no match"
    );
}
