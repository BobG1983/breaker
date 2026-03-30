use bevy::prelude::*;

use super::{small_aabb, spawn_entities, test_tree};
use crate::{aabb::Aabb2D, collision_layers::CollisionLayers};

// ── Behavior 7: insert takes (Entity, Aabb2D, CollisionLayers) ──

#[test]
fn insert_with_layers_increases_len() {
    let mut tree = test_tree();
    let entities = spawn_entities(1);
    let aabb = Aabb2D::new(Vec2::new(10.0, 10.0), Vec2::new(1.0, 1.0));
    let layers = CollisionLayers::new(0x01, 0x02);
    tree.insert(entities[0], aabb, layers);
    assert_eq!(tree.len(), 1);
}

#[test]
fn insert_with_default_layers_stores_but_invisible_to_filtered_queries() {
    let mut tree = test_tree();
    let entities = spawn_entities(1);
    let aabb = small_aabb(10.0, 10.0);
    // Insert with default layers (membership=0, mask=0)
    tree.insert(entities[0], aabb, CollisionLayers::default());
    assert_eq!(tree.len(), 1);

    // Filtered query with any mask should not return it
    let region = Aabb2D::new(Vec2::new(10.0, 10.0), Vec2::new(5.0, 5.0));
    let results = tree.query_aabb_filtered(&region, CollisionLayers::new(0xFF, 0xFF));
    assert!(
        results.is_empty(),
        "entity with default layers (membership=0) should be invisible to filtered queries"
    );
}

// ── Behavior 8: query_aabb still returns all (unfiltered, backward compat) ──

#[test]
fn query_aabb_unfiltered_returns_all_regardless_of_layers() {
    let mut tree = test_tree();
    let entities = spawn_entities(3);
    tree.insert(
        entities[0],
        small_aabb(10.0, 10.0),
        CollisionLayers::new(0x01, 0x02),
    );
    tree.insert(
        entities[1],
        small_aabb(12.0, 10.0),
        CollisionLayers::new(0x02, 0x01),
    );
    // Entity with default (invisible) layers
    tree.insert(
        entities[2],
        small_aabb(10.0, 12.0),
        CollisionLayers::default(),
    );

    let region = Aabb2D::new(Vec2::new(11.0, 11.0), Vec2::new(10.0, 10.0));
    let mut results = tree.query_aabb(&region);
    results.sort();
    let mut expected = vec![entities[0], entities[1], entities[2]];
    expected.sort();
    assert_eq!(
        results, expected,
        "unfiltered query_aabb should return all spatially overlapping entities"
    );
}

// ── Behavior 9: query_aabb_filtered returns only matching layers ──

#[test]
fn query_aabb_filtered_returns_only_entities_whose_membership_matches_query_mask() {
    let mut tree = test_tree();
    let entities = spawn_entities(2);
    // entity_a: membership=0x01, mask=0x02
    tree.insert(
        entities[0],
        small_aabb(10.0, 10.0),
        CollisionLayers::new(0x01, 0x02),
    );
    // entity_b: membership=0x02, mask=0x00
    tree.insert(
        entities[1],
        small_aabb(12.0, 10.0),
        CollisionLayers::new(0x02, 0x00),
    );

    let region = Aabb2D::new(Vec2::new(11.0, 10.0), Vec2::new(10.0, 10.0));
    // Query with mask=0x02 → should match entity_b (membership=0x02)
    let results = tree.query_aabb_filtered(&region, CollisionLayers::new(0x00, 0x02));
    assert_eq!(results.len(), 1, "only entity_b should match mask=0x02");
    assert_eq!(results[0], entities[1]);
}

#[test]
fn query_aabb_filtered_with_zero_mask_returns_empty() {
    let mut tree = test_tree();
    let entities = spawn_entities(1);
    tree.insert(
        entities[0],
        small_aabb(10.0, 10.0),
        CollisionLayers::new(0xFF, 0xFF),
    );

    let region = Aabb2D::new(Vec2::new(10.0, 10.0), Vec2::new(5.0, 5.0));
    let results = tree.query_aabb_filtered(&region, CollisionLayers::new(0x00, 0x00));
    assert!(
        results.is_empty(),
        "query with mask=0 should return nothing"
    );
}

// ── Behavior 10: query_aabb_filtered with game-like layer config ──

#[test]
fn query_aabb_filtered_game_like_layers_from_querier_perspective() {
    // Layer constants (game-agnostic, just bit positions)
    const LAYER_A: u32 = 1 << 0; // 0x01
    const LAYER_B: u32 = 1 << 1; // 0x02
    const LAYER_C: u32 = 1 << 2; // 0x04
    const LAYER_D: u32 = 1 << 3; // 0x08

    let mut tree = test_tree();
    let entities = spawn_entities(4);

    // "querier" entity at (0,0) — membership=LAYER_A, mask=LAYER_B|LAYER_C
    tree.insert(
        entities[0],
        small_aabb(0.0, 0.0),
        CollisionLayers::new(LAYER_A, LAYER_B | LAYER_C),
    );
    // "target_b" entity at (5,0) — membership=LAYER_B, mask=LAYER_A
    tree.insert(
        entities[1],
        small_aabb(5.0, 0.0),
        CollisionLayers::new(LAYER_B, LAYER_A),
    );
    // "target_c" entity at (10,0) — membership=LAYER_C, mask=LAYER_A
    tree.insert(
        entities[2],
        small_aabb(10.0, 0.0),
        CollisionLayers::new(LAYER_C, LAYER_A),
    );
    // "unrelated_d" entity at (15,0) — membership=LAYER_D, mask=LAYER_A
    tree.insert(
        entities[3],
        small_aabb(15.0, 0.0),
        CollisionLayers::new(LAYER_D, LAYER_A),
    );

    let big_region = Aabb2D::new(Vec2::new(7.5, 0.0), Vec2::new(20.0, 5.0));

    // Query "from querier's perspective": mask = LAYER_B | LAYER_C
    let mut results = tree.query_aabb_filtered(
        &big_region,
        CollisionLayers::new(LAYER_A, LAYER_B | LAYER_C),
    );
    results.sort();

    let mut expected = vec![entities[1], entities[2]];
    expected.sort();

    assert_eq!(
        results, expected,
        "should find target_b and target_c but not querier itself or unrelated_d"
    );
}

#[test]
fn query_aabb_filtered_only_one_layer_from_multi_layer_query() {
    const LAYER_B: u32 = 1 << 1; // 0x02
    const LAYER_C: u32 = 1 << 2; // 0x04

    let mut tree = test_tree();
    let entities = spawn_entities(2);

    tree.insert(
        entities[0],
        small_aabb(5.0, 0.0),
        CollisionLayers::new(LAYER_B, 0x01),
    );
    tree.insert(
        entities[1],
        small_aabb(10.0, 0.0),
        CollisionLayers::new(LAYER_C, 0x01),
    );

    let big_region = Aabb2D::new(Vec2::new(7.5, 0.0), Vec2::new(20.0, 5.0));

    // Query with only LAYER_B mask — should only find first entity
    let results = tree.query_aabb_filtered(&big_region, CollisionLayers::new(0x00, LAYER_B));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], entities[0]);
}

// ── Behavior 11: query_circle_filtered returns only matching entities in radius ──

#[test]
fn query_circle_filtered_returns_only_interacting_entities_within_radius() {
    let mut tree = test_tree();
    let entities = spawn_entities(2);

    // entity_a: membership=0x02, mask=0x01
    tree.insert(
        entities[0],
        small_aabb(5.0, 0.0),
        CollisionLayers::new(0x02, 0x01),
    );
    // entity_b: membership=0x04, mask=0x01
    tree.insert(
        entities[1],
        small_aabb(5.0, 0.0),
        CollisionLayers::new(0x04, 0x01),
    );

    // Query with mask=0x02 only — should match entity_a (membership=0x02)
    let results = tree.query_circle_filtered(Vec2::ZERO, 10.0, CollisionLayers::new(0x01, 0x02));
    assert_eq!(results.len(), 1, "only entity_a should match");
    assert_eq!(results[0], entities[0]);
}

#[test]
fn query_circle_filtered_excludes_entity_outside_radius_on_correct_layer() {
    let mut tree = test_tree();
    let entities = spawn_entities(2);

    // entity_a in range: membership=0x02 at (5,0)
    tree.insert(
        entities[0],
        small_aabb(5.0, 0.0),
        CollisionLayers::new(0x02, 0x01),
    );
    // entity_b far away: membership=0x02 at (200,0)
    tree.insert(
        entities[1],
        small_aabb(200.0, 0.0),
        CollisionLayers::new(0x02, 0x01),
    );

    // Both on the right layer, but only entity_a is within radius
    let results = tree.query_circle_filtered(Vec2::ZERO, 10.0, CollisionLayers::new(0x01, 0x02));
    assert_eq!(
        results.len(),
        1,
        "only in-range entity_a should be returned"
    );
    assert_eq!(results[0], entities[0]);
}

// ── D5: query_aabb_filtered finds branch-level items after split ──

#[test]
fn query_aabb_filtered_finds_branch_level_items_after_split() {
    use crate::quadtree::Quadtree;

    // Capacity 2 forces split with 3+ entities
    let mut tree = Quadtree::new(Aabb2D::new(Vec2::ZERO, Vec2::new(500.0, 500.0)), 2, 8);
    let entities = spawn_entities(4);

    // entities[0]: large AABB spanning quadrants, membership = 0x01
    tree.insert(
        entities[0],
        Aabb2D::new(Vec2::ZERO, Vec2::new(100.0, 100.0)),
        CollisionLayers::new(0x01, 0x02),
    );
    // Small entities in distinct quadrants with membership = 0x02
    tree.insert(
        entities[1],
        small_aabb(200.0, 200.0),
        CollisionLayers::new(0x02, 0x01),
    );
    tree.insert(
        entities[2],
        small_aabb(-200.0, -200.0),
        CollisionLayers::new(0x02, 0x01),
    );
    tree.insert(
        entities[3],
        small_aabb(200.0, -200.0),
        CollisionLayers::new(0x02, 0x01),
    );

    // Query with mask = 0x01, matching only entities with membership 0x01 (entities[0])
    let results = tree.query_aabb_filtered(
        &Aabb2D::new(Vec2::ZERO, Vec2::new(50.0, 50.0)),
        CollisionLayers::new(0x00, 0x01),
    );
    assert_eq!(
        results.len(),
        1,
        "should find exactly the branch-level entity with membership 0x01"
    );
    assert_eq!(
        results[0], entities[0],
        "result should be entities[0] (branch-level item)"
    );

    // Edge case: query with mask = 0x04 (no entity has membership 0x04)
    let results_empty = tree.query_aabb_filtered(
        &Aabb2D::new(Vec2::ZERO, Vec2::new(50.0, 50.0)),
        CollisionLayers::new(0x00, 0x04),
    );
    assert!(
        results_empty.is_empty(),
        "query with mask=0x04 should return empty — no entity has membership 0x04"
    );
}
