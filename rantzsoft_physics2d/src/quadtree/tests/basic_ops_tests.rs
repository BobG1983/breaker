use bevy::prelude::*;

use super::{small_aabb, spawn_entities, test_tree};
use crate::{aabb::Aabb2D, collision_layers::CollisionLayers, quadtree::Quadtree};

#[test]
fn new_creates_empty_tree() {
    let tree = test_tree();
    assert_eq!(tree.len(), 0);
    assert!(tree.is_empty());
}

#[test]
fn insert_increases_len() {
    let mut tree = test_tree();
    let entities = spawn_entities(1);
    tree.insert(
        entities[0],
        small_aabb(10.0, 10.0),
        CollisionLayers::default(),
    );
    assert_eq!(tree.len(), 1);
    assert!(!tree.is_empty());
}

#[test]
#[expect(
    clippy::cast_precision_loss,
    reason = "test loop index is always small"
)]
fn insert_multiple_entities() {
    let mut tree = test_tree();
    let entities = spawn_entities(5);
    for (i, &e) in entities.iter().enumerate() {
        tree.insert(
            e,
            small_aabb(i as f32 * 10.0, 0.0),
            CollisionLayers::default(),
        );
    }
    assert_eq!(tree.len(), 5);
}

#[test]
fn remove_decreases_len() {
    let mut tree = test_tree();
    let entities = spawn_entities(3);
    tree.insert(
        entities[0],
        small_aabb(0.0, 0.0),
        CollisionLayers::default(),
    );
    tree.insert(
        entities[1],
        small_aabb(10.0, 0.0),
        CollisionLayers::default(),
    );
    tree.insert(
        entities[2],
        small_aabb(20.0, 0.0),
        CollisionLayers::default(),
    );

    let removed = tree.remove(entities[1]);
    assert!(removed);
    assert_eq!(tree.len(), 2);
}

#[test]
fn remove_returns_false_for_missing_entity() {
    let mut tree = test_tree();
    let entities = spawn_entities(2);
    tree.insert(
        entities[0],
        small_aabb(0.0, 0.0),
        CollisionLayers::default(),
    );
    // entities[1] was never inserted
    let removed = tree.remove(entities[1]);
    assert!(!removed);
    assert_eq!(tree.len(), 1);
}

#[test]
fn query_aabb_finds_overlapping_entity() {
    let mut tree = test_tree();
    let entities = spawn_entities(1);
    tree.insert(
        entities[0],
        small_aabb(10.0, 10.0),
        CollisionLayers::default(),
    );

    let query_region = Aabb2D::new(Vec2::new(10.0, 10.0), Vec2::new(5.0, 5.0));
    let results = tree.query_aabb(&query_region);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], entities[0]);
}

#[test]
fn query_aabb_misses_distant_entity() {
    let mut tree = test_tree();
    let entities = spawn_entities(1);
    tree.insert(
        entities[0],
        small_aabb(10.0, 10.0),
        CollisionLayers::default(),
    );

    let query_region = Aabb2D::new(Vec2::new(100.0, 100.0), Vec2::new(5.0, 5.0));
    let results = tree.query_aabb(&query_region);
    assert!(results.is_empty());
}

#[test]
fn query_aabb_finds_multiple_overlapping_entities() {
    let mut tree = test_tree();
    let entities = spawn_entities(3);
    tree.insert(
        entities[0],
        small_aabb(0.0, 0.0),
        CollisionLayers::default(),
    );
    tree.insert(
        entities[1],
        small_aabb(2.0, 0.0),
        CollisionLayers::default(),
    );
    tree.insert(
        entities[2],
        small_aabb(0.0, 2.0),
        CollisionLayers::default(),
    );

    // Query region covering all three
    let query_region = Aabb2D::new(Vec2::new(1.0, 1.0), Vec2::new(5.0, 5.0));
    let mut results = tree.query_aabb(&query_region);
    results.sort();
    let mut expected = vec![entities[0], entities[1], entities[2]];
    expected.sort();
    assert_eq!(results.len(), 3);
    assert_eq!(results, expected);
}

#[test]
fn query_circle_finds_entity_in_range() {
    let mut tree = test_tree();
    let entities = spawn_entities(1);
    tree.insert(
        entities[0],
        small_aabb(5.0, 0.0),
        CollisionLayers::default(),
    );

    let results = tree.query_circle(Vec2::ZERO, 10.0);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], entities[0]);
}

#[test]
fn query_circle_misses_entity_out_of_range() {
    let mut tree = test_tree();
    let entities = spawn_entities(1);
    tree.insert(
        entities[0],
        small_aabb(100.0, 0.0),
        CollisionLayers::default(),
    );

    let results = tree.query_circle(Vec2::ZERO, 10.0);
    assert!(results.is_empty());
}

#[test]
#[expect(
    clippy::cast_precision_loss,
    reason = "test loop index is always small"
)]
fn clear_empties_the_tree() {
    let mut tree = test_tree();
    let entities = spawn_entities(5);
    for (i, &e) in entities.iter().enumerate() {
        tree.insert(
            e,
            small_aabb(i as f32 * 10.0, 0.0),
            CollisionLayers::default(),
        );
    }
    assert_eq!(tree.len(), 5);

    tree.clear();
    assert_eq!(tree.len(), 0);
    assert!(tree.is_empty());
}

#[test]
fn insert_then_remove_then_query_finds_nothing() {
    let mut tree = test_tree();
    let entities = spawn_entities(1);
    tree.insert(
        entities[0],
        small_aabb(10.0, 10.0),
        CollisionLayers::default(),
    );
    tree.remove(entities[0]);

    let query_region = Aabb2D::new(Vec2::new(10.0, 10.0), Vec2::new(5.0, 5.0));
    let results = tree.query_aabb(&query_region);
    assert!(
        results.is_empty(),
        "removed entity should not appear in query"
    );
}

#[test]
#[expect(
    clippy::cast_precision_loss,
    reason = "test loop index is always small"
)]
fn handles_splitting_when_exceeding_leaf_capacity() {
    // Use a tree with max 4 items per leaf to force splits
    let mut tree = Quadtree::new(Aabb2D::new(Vec2::ZERO, Vec2::new(500.0, 500.0)), 4, 8);

    let entities = spawn_entities(10);
    // Insert 10 entities in the same area to trigger splitting
    for (i, &e) in entities.iter().enumerate() {
        tree.insert(
            e,
            small_aabb(i as f32 * 0.5, i as f32 * 0.5),
            CollisionLayers::default(),
        );
    }
    assert_eq!(tree.len(), 10);

    // All should still be queryable
    let query_region = Aabb2D::new(Vec2::ZERO, Vec2::new(20.0, 20.0));
    let results = tree.query_aabb(&query_region);
    assert_eq!(
        results.len(),
        10,
        "all entities should be found after splits"
    );
}

#[test]
#[expect(
    clippy::cast_precision_loss,
    reason = "test loop index is always small"
)]
fn query_returns_no_duplicates_for_entity_spanning_quadrants() {
    let mut tree = Quadtree::new(Aabb2D::new(Vec2::ZERO, Vec2::new(500.0, 500.0)), 4, 8);

    let entities = spawn_entities(6);

    // Insert a large entity centered at the origin that spans all four quadrants
    let big_aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
    tree.insert(entities[0], big_aabb, CollisionLayers::default());

    // Also insert some small entities to potentially trigger splits
    for (i, &e) in entities[1..6].iter().enumerate() {
        tree.insert(
            e,
            small_aabb(200.0 + i as f32, 200.0),
            CollisionLayers::default(),
        );
    }

    // Query the area containing the large entity
    let query_region = Aabb2D::new(Vec2::ZERO, Vec2::new(50.0, 50.0));
    let results = tree.query_aabb(&query_region);

    let count = results.iter().filter(|&&e| e == entities[0]).count();
    assert_eq!(
        count, 1,
        "entity spanning multiple quadrants should appear exactly once"
    );
}
