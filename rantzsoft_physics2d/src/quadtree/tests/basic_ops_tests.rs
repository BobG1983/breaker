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

// ── D2: Remove entity stored at branch level (spans quadrants) after split ──

#[test]
fn remove_branch_level_item_after_split() {
    // Capacity 2 forces split with 3+ entities in distinct quadrants
    let mut tree = Quadtree::new(Aabb2D::new(Vec2::ZERO, Vec2::new(500.0, 500.0)), 2, 8);
    let entities = spawn_entities(4);

    // entities[0]: large AABB spanning all quadrants → stored at branch level after split
    tree.insert(
        entities[0],
        Aabb2D::new(Vec2::ZERO, Vec2::new(100.0, 100.0)),
        CollisionLayers::default(),
    );
    // Three small entities in distinct quadrants to trigger the split
    tree.insert(
        entities[1],
        small_aabb(200.0, 200.0),
        CollisionLayers::default(),
    );
    tree.insert(
        entities[2],
        small_aabb(-200.0, -200.0),
        CollisionLayers::default(),
    );
    tree.insert(
        entities[3],
        small_aabb(200.0, -200.0),
        CollisionLayers::default(),
    );
    assert_eq!(tree.len(), 4);

    // Remove the branch-level entity
    let removed = tree.remove(entities[0]);
    assert!(removed, "remove should return true for branch-level entity");
    assert_eq!(tree.len(), 3);

    // The spanning entity should no longer appear in queries
    let results = tree.query_aabb(&Aabb2D::new(Vec2::ZERO, Vec2::new(50.0, 50.0)));
    assert!(
        !results.contains(&entities[0]),
        "removed branch-level entity should not appear in query results"
    );

    // Edge case: other entities remain queryable
    let results_tr = tree.query_aabb(&Aabb2D::new(Vec2::new(200.0, 200.0), Vec2::new(5.0, 5.0)));
    assert_eq!(
        results_tr,
        vec![entities[1]],
        "entities[1] should still be queryable after removing branch-level entity"
    );
}

// ── D3: Remove entity in child leaf after split ──

#[test]
fn remove_child_leaf_item_after_split() {
    let mut tree = Quadtree::new(Aabb2D::new(Vec2::ZERO, Vec2::new(500.0, 500.0)), 2, 8);
    let entities = spawn_entities(4);

    // Same setup as D2
    tree.insert(
        entities[0],
        Aabb2D::new(Vec2::ZERO, Vec2::new(100.0, 100.0)),
        CollisionLayers::default(),
    );
    tree.insert(
        entities[1],
        small_aabb(200.0, 200.0),
        CollisionLayers::default(),
    );
    tree.insert(
        entities[2],
        small_aabb(-200.0, -200.0),
        CollisionLayers::default(),
    );
    tree.insert(
        entities[3],
        small_aabb(200.0, -200.0),
        CollisionLayers::default(),
    );
    assert_eq!(tree.len(), 4);

    // Remove a child leaf entity
    let removed = tree.remove(entities[1]);
    assert!(removed, "remove should return true for child leaf entity");
    assert_eq!(tree.len(), 3);

    // The removed entity should no longer appear in queries
    let results = tree.query_aabb(&Aabb2D::new(Vec2::new(200.0, 200.0), Vec2::new(5.0, 5.0)));
    assert!(
        results.is_empty(),
        "removed child leaf entity should not appear in query results"
    );

    // Edge case: the branch-level entity at origin still exists
    let results_origin = tree.query_aabb(&Aabb2D::new(Vec2::ZERO, Vec2::new(50.0, 50.0)));
    assert!(
        results_origin.contains(&entities[0]),
        "branch-level entity should still be queryable after removing child leaf entity"
    );
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

// ── D7: Remove from split tree maintains correct len() count ──

#[test]
fn remove_from_split_tree_maintains_correct_len() {
    let mut tree = Quadtree::new(Aabb2D::new(Vec2::ZERO, Vec2::new(500.0, 500.0)), 2, 8);
    let entities = spawn_entities(5);

    // entities[0]: branch-level (spans quadrants)
    tree.insert(
        entities[0],
        Aabb2D::new(Vec2::ZERO, Vec2::new(100.0, 100.0)),
        CollisionLayers::default(),
    );
    // entities[1..4]: in distinct quadrants
    tree.insert(
        entities[1],
        small_aabb(200.0, 200.0),
        CollisionLayers::default(),
    );
    tree.insert(
        entities[2],
        small_aabb(-200.0, -200.0),
        CollisionLayers::default(),
    );
    tree.insert(
        entities[3],
        small_aabb(200.0, -200.0),
        CollisionLayers::default(),
    );
    tree.insert(
        entities[4],
        small_aabb(-200.0, 200.0),
        CollisionLayers::default(),
    );
    assert_eq!(tree.len(), 5);

    // First remove: child leaf item
    let removed1 = tree.remove(entities[2]);
    assert!(removed1, "first remove should return true");
    assert_eq!(tree.len(), 4);

    // Second remove: branch-level item
    let removed2 = tree.remove(entities[0]);
    assert!(removed2, "second remove should return true");
    assert_eq!(tree.len(), 3);

    // Third remove: already removed entity
    let removed3 = tree.remove(entities[2]);
    assert!(
        !removed3,
        "removing already-removed entity should return false"
    );
    assert_eq!(tree.len(), 3);

    // Edge case: remaining entities are still queryable
    let top_right = tree.query_aabb(&Aabb2D::new(Vec2::new(200.0, 200.0), Vec2::new(5.0, 5.0)));
    assert_eq!(
        top_right,
        vec![entities[1]],
        "entities[1] should still be queryable"
    );

    let top_left = tree.query_aabb(&Aabb2D::new(Vec2::new(-200.0, 200.0), Vec2::new(5.0, 5.0)));
    assert_eq!(
        top_left,
        vec![entities[4]],
        "entities[4] should still be queryable"
    );
}
