use bevy::prelude::*;

use super::super::{small_aabb, spawn_entities, test_tree};
use crate::{aabb::Aabb2D, collision_layers::CollisionLayers};

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

// ── D4: query_circle excludes entity whose AABB is outside circle radius at corner ──

#[test]
fn query_circle_excludes_aabb_outside_radius_at_corner() {
    let mut tree = test_tree();
    let entities = spawn_entities(2);

    // Excluded entity: AABB at (9, 9) with half-extents (1.5, 1.5) → spans (7.5, 7.5) to (10.5, 10.5)
    // Closest point to origin = (7.5, 7.5), distance = sqrt(56.25 + 56.25) = sqrt(112.5) ~ 10.607
    tree.insert(
        entities[0],
        Aabb2D::new(Vec2::new(9.0, 9.0), Vec2::new(1.5, 1.5)),
        CollisionLayers::default(),
    );

    // Included entity: AABB at (8, 8) with half-extents (1.5, 1.5) → spans (6.5, 6.5) to (9.5, 9.5)
    // Closest point to origin = (6.5, 6.5), distance = sqrt(42.25 + 42.25) = sqrt(84.5) ~ 9.192
    tree.insert(
        entities[1],
        Aabb2D::new(Vec2::new(8.0, 8.0), Vec2::new(1.5, 1.5)),
        CollisionLayers::default(),
    );

    // Circle centered at origin with radius 10.0
    let results = tree.query_circle(Vec2::ZERO, 10.0);
    assert_eq!(
        results.len(),
        1,
        "only the closer entity should be within circle radius 10.0"
    );
    assert_eq!(
        results[0], entities[1],
        "the included entity at (8,8) should be in results"
    );

    // Edge case: radius exactly matching the excluded entity's distance (~10.607)
    // should include it (boundary is <=)
    let results_boundary = tree.query_circle(Vec2::ZERO, 10.607);
    assert_eq!(
        results_boundary.len(),
        2,
        "at radius ~10.607 both entities should be included (boundary is <=)"
    );
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

// ── D6: clear() resets tree; insert + query works correctly after clear ──

#[test]
fn clear_resets_tree_insert_and_query_after_clear() {
    let mut tree = test_tree();
    let entities = spawn_entities(3);

    tree.insert(
        entities[0],
        small_aabb(10.0, 10.0),
        CollisionLayers::default(),
    );
    tree.insert(
        entities[1],
        small_aabb(20.0, 20.0),
        CollisionLayers::default(),
    );
    tree.insert(
        entities[2],
        small_aabb(30.0, 30.0),
        CollisionLayers::default(),
    );
    assert_eq!(tree.len(), 3);

    tree.clear();
    assert_eq!(tree.len(), 0);
    assert!(tree.is_empty());

    // Insert new entities after clear
    let new_entities = spawn_entities(2);
    tree.insert(
        new_entities[0],
        small_aabb(50.0, 50.0),
        CollisionLayers::default(),
    );
    tree.insert(
        new_entities[1],
        small_aabb(60.0, 60.0),
        CollisionLayers::default(),
    );
    assert_eq!(tree.len(), 2);

    // New entities are queryable
    let mut results = tree.query_aabb(&Aabb2D::new(Vec2::new(55.0, 55.0), Vec2::new(20.0, 20.0)));
    results.sort();
    let mut expected = vec![new_entities[0], new_entities[1]];
    expected.sort();
    assert_eq!(
        results, expected,
        "new entities should be queryable after clear + re-insert"
    );

    // Old entity positions return nothing
    let old_results = tree.query_aabb(&Aabb2D::new(Vec2::new(10.0, 10.0), Vec2::new(5.0, 5.0)));
    assert!(
        old_results.is_empty(),
        "old entity positions should return empty after clear"
    );
}
