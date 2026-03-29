use bevy::prelude::*;

use super::super::effect::*;
use crate::effect::core::AttractionType;

// ── Existing tests (updated for new max_force parameter) ──

#[test]
fn fire_inserts_active_attractions_on_fresh_entity() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    fire(entity, AttractionType::Cell, 10.0, None, &mut world);

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
            max_force: None,
            active: true,
        }]))
        .id();

    fire(entity, AttractionType::Breaker, 15.0, None, &mut world);

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
                max_force: None,
                active: true,
            },
            AttractionEntry {
                attraction_type: AttractionType::Wall,
                force: 5.0,
                max_force: None,
                active: true,
            },
        ]))
        .id();

    reverse(entity, AttractionType::Cell, 10.0, None, &mut world);

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
            max_force: None,
            active: true,
        }]))
        .id();

    // Different type -- no match.
    reverse(entity, AttractionType::Breaker, 10.0, None, &mut world);

    let attractions = world.get::<ActiveAttractions>(entity).unwrap();
    assert_eq!(
        attractions.0.len(),
        1,
        "no entry should be removed when no match"
    );
}

// ── Behavior 7: fire() stores max_force in AttractionEntry ──

#[test]
fn fire_stores_max_force_in_attraction_entry() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    fire(entity, AttractionType::Cell, 500.0, Some(300.0), &mut world);

    let attractions = world.get::<ActiveAttractions>(entity).unwrap();
    assert_eq!(attractions.0.len(), 1);
    assert_eq!(attractions.0[0].attraction_type, AttractionType::Cell);
    assert!(
        (attractions.0[0].force - 500.0).abs() < f32::EPSILON,
        "expected force 500.0, got {}",
        attractions.0[0].force
    );
    assert_eq!(
        attractions.0[0].max_force,
        Some(300.0),
        "expected max_force Some(300.0), got {:?}",
        attractions.0[0].max_force
    );
    assert!(attractions.0[0].active);
}

#[test]
fn fire_stores_none_max_force_when_none_passed() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    fire(entity, AttractionType::Cell, 500.0, None, &mut world);

    let attractions = world.get::<ActiveAttractions>(entity).unwrap();
    assert_eq!(
        attractions.0[0].max_force, None,
        "expected max_force None when None passed, got {:?}",
        attractions.0[0].max_force
    );
}

// ── Behavior 8: fire() appends entry with max_force to existing ActiveAttractions ──

#[test]
fn fire_appends_entry_with_max_force_to_existing() {
    let mut world = World::new();
    let entity = world
        .spawn(ActiveAttractions(vec![AttractionEntry {
            attraction_type: AttractionType::Cell,
            force: 200.0,
            max_force: None,
            active: true,
        }]))
        .id();

    fire(entity, AttractionType::Wall, 800.0, Some(400.0), &mut world);

    let attractions = world.get::<ActiveAttractions>(entity).unwrap();
    assert_eq!(
        attractions.0.len(),
        2,
        "should have two entries after appending"
    );
    assert_eq!(attractions.0[1].attraction_type, AttractionType::Wall);
    assert!(
        (attractions.0[1].force - 800.0).abs() < f32::EPSILON,
        "expected force 800.0, got {}",
        attractions.0[1].force
    );
    assert_eq!(
        attractions.0[1].max_force,
        Some(400.0),
        "expected max_force Some(400.0), got {:?}",
        attractions.0[1].max_force
    );
}

// ── Behavior 9: reverse() matches on type + force + max_force ──

#[test]
fn reverse_matches_on_type_force_and_max_force() {
    let mut world = World::new();
    let entity = world
        .spawn(ActiveAttractions(vec![AttractionEntry {
            attraction_type: AttractionType::Cell,
            force: 500.0,
            max_force: Some(300.0),
            active: true,
        }]))
        .id();

    reverse(entity, AttractionType::Cell, 500.0, Some(300.0), &mut world);

    let attractions = world.get::<ActiveAttractions>(entity).unwrap();
    assert!(
        attractions.0.is_empty(),
        "matching entry should be removed when type + force + max_force all match"
    );
}

#[test]
fn reverse_removes_only_matching_max_force_entry() {
    let mut world = World::new();
    let entity = world
        .spawn(ActiveAttractions(vec![
            AttractionEntry {
                attraction_type: AttractionType::Cell,
                force: 500.0,
                max_force: Some(300.0),
                active: true,
            },
            AttractionEntry {
                attraction_type: AttractionType::Cell,
                force: 500.0,
                max_force: Some(999.0),
                active: true,
            },
        ]))
        .id();

    reverse(entity, AttractionType::Cell, 500.0, Some(300.0), &mut world);

    let attractions = world.get::<ActiveAttractions>(entity).unwrap();
    assert_eq!(
        attractions.0.len(),
        1,
        "only the entry with matching max_force should be removed"
    );
    assert_eq!(
        attractions.0[0].max_force,
        Some(999.0),
        "remaining entry should be the one with different max_force"
    );
}

// ── Behavior 10: reverse() negative match — different max_force is NOT removed ──

#[test]
fn reverse_does_not_remove_entry_with_different_max_force() {
    let mut world = World::new();
    let entity = world
        .spawn(ActiveAttractions(vec![AttractionEntry {
            attraction_type: AttractionType::Cell,
            force: 500.0,
            max_force: Some(300.0),
            active: true,
        }]))
        .id();

    reverse(entity, AttractionType::Cell, 500.0, Some(999.0), &mut world);

    let attractions = world.get::<ActiveAttractions>(entity).unwrap();
    assert_eq!(
        attractions.0.len(),
        1,
        "entry should NOT be removed when max_force does not match"
    );
}

#[test]
fn reverse_none_max_force_does_not_match_some_max_force() {
    let mut world = World::new();
    let entity = world
        .spawn(ActiveAttractions(vec![AttractionEntry {
            attraction_type: AttractionType::Cell,
            force: 500.0,
            max_force: Some(300.0),
            active: true,
        }]))
        .id();

    reverse(entity, AttractionType::Cell, 500.0, None, &mut world);

    let attractions = world.get::<ActiveAttractions>(entity).unwrap();
    assert_eq!(
        attractions.0.len(),
        1,
        "reverse with max_force: None should not match entry with max_force: Some(300.0)"
    );
}

// ── Behavior 11: EffectKind::Attraction::reverse() dispatch forwards max_force ──

#[test]
fn effect_kind_attraction_reverse_dispatch_forwards_max_force() {
    let mut world = World::new();
    let entity = world
        .spawn(ActiveAttractions(vec![AttractionEntry {
            attraction_type: AttractionType::Cell,
            force: 500.0,
            max_force: Some(300.0),
            active: true,
        }]))
        .id();

    let effect = crate::effect::core::EffectKind::Attraction {
        attraction_type: AttractionType::Cell,
        force: 500.0,
        max_force: Some(300.0),
    };
    effect.reverse(entity, &mut world);

    let attractions = world.get::<ActiveAttractions>(entity).unwrap();
    assert!(
        attractions.0.is_empty(),
        "EffectKind::Attraction reverse dispatch should forward max_force to attraction::reverse() and remove matching entry"
    );
}

// ── Behavior 17: EffectKind::Attraction::fire() dispatch forwards max_force ──

#[test]
fn effect_kind_attraction_fire_dispatch_forwards_max_force() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

    let effect = crate::effect::core::EffectKind::Attraction {
        attraction_type: AttractionType::Cell,
        force: 500.0,
        max_force: Some(300.0),
    };
    effect.fire(entity, &mut world);

    let attractions = world
        .get::<ActiveAttractions>(entity)
        .expect("entity should have ActiveAttractions after fire()");
    assert_eq!(attractions.0.len(), 1);
    assert_eq!(
        attractions.0[0].max_force,
        Some(300.0),
        "EffectKind::Attraction fire dispatch should forward max_force to attraction::fire(), got {:?}",
        attractions.0[0].max_force
    );
}
