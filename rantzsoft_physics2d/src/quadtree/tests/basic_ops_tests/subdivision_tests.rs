use bevy::prelude::*;

use crate::{
    aabb::Aabb2D,
    collision_layers::CollisionLayers,
    quadtree::{
        Quadtree,
        tests::{small_aabb, spawn_entities},
    },
};

#[test]
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
