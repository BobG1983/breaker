use bevy::prelude::*;

use super::{spawn_entities, test_tree};
use crate::{aabb::Aabb2D, collision_layers::CollisionLayers};

// ── Behavior 5: cast_circle finds entity in path ──

#[test]
fn cast_circle_finds_entity_in_path() {
    let mut tree = test_tree();
    let entities = spawn_entities(1);

    // Entity at (0, 50) with half_extents (10, 10), membership=0x01
    let aabb = Aabb2D::new(Vec2::new(0.0, 50.0), Vec2::new(10.0, 10.0));
    tree.insert(entities[0], aabb, CollisionLayers::new(0x01, 0x01));

    // Cast circle from origin upward, radius=5
    // Stored AABB: center=50, half=10. Expanded by radius 5 → half=15.
    // Bottom face of expanded AABB at y = 50 - 15 = 35.
    // Ray from y=0 going up → hits at distance 35.0.
    let hits = tree.cast_circle(
        Vec2::new(0.0, 0.0),
        Vec2::Y,
        200.0,
        5.0,
        CollisionLayers::new(0x00, 0x01),
    );

    assert_eq!(hits.len(), 1, "should find one entity in path");
    assert_eq!(hits[0].entity, entities[0]);
    // Position should be safe stop point just before contact (~35.0 along Y)
    assert!(
        (hits[0].position.y - 35.0).abs() < 1.0,
        "position.y should be ~35.0, got {}",
        hits[0].position.y
    );
    assert_eq!(
        hits[0].normal,
        Vec2::NEG_Y,
        "should report bottom face normal"
    );
    // Remaining = max_dist - distance = 200.0 - 35.0 = 165.0
    assert!(
        (hits[0].remaining - 165.0).abs() < 1.0,
        "remaining should be ~165.0, got {}",
        hits[0].remaining
    );
}

#[test]
fn cast_circle_entity_not_on_matching_layer_returns_empty() {
    let mut tree = test_tree();
    let entities = spawn_entities(1);

    // Entity on layer 0x02
    let aabb = Aabb2D::new(Vec2::new(0.0, 50.0), Vec2::new(10.0, 10.0));
    tree.insert(entities[0], aabb, CollisionLayers::new(0x02, 0x02));

    // Cast with mask=0x01 — does not match membership=0x02
    let hits = tree.cast_circle(
        Vec2::new(0.0, 0.0),
        Vec2::Y,
        200.0,
        5.0,
        CollisionLayers::new(0x00, 0x01),
    );

    assert!(
        hits.is_empty(),
        "entity on non-matching layer should not be hit"
    );
}

// ── Behavior 6: cast_circle returns hits sorted by distance ──

#[test]
fn cast_circle_returns_hits_sorted_nearest_first() {
    let mut tree = test_tree();
    let entities = spawn_entities(2);

    // entity_a at (0, 30) with half_extents (5, 5)
    let aabb_a = Aabb2D::new(Vec2::new(0.0, 30.0), Vec2::new(5.0, 5.0));
    tree.insert(entities[0], aabb_a, CollisionLayers::new(0x01, 0x01));

    // entity_b at (0, 80) with half_extents (5, 5)
    let aabb_b = Aabb2D::new(Vec2::new(0.0, 80.0), Vec2::new(5.0, 5.0));
    tree.insert(entities[1], aabb_b, CollisionLayers::new(0x01, 0x01));

    // Cast ray (radius=0) from origin upward
    // entity_a bottom face at y=25, entity_b bottom face at y=75
    let hits = tree.cast_circle(
        Vec2::new(0.0, 0.0),
        Vec2::Y,
        200.0,
        0.0,
        CollisionLayers::new(0x00, 0x01),
    );

    assert_eq!(hits.len(), 2, "should find both entities");
    assert_eq!(
        hits[0].entity, entities[0],
        "first hit should be entity_a (closer)"
    );
    assert_eq!(
        hits[1].entity, entities[1],
        "second hit should be entity_b (farther)"
    );
}

// ── Behavior 7: cast_circle misses entity outside path ──

#[test]
fn cast_circle_misses_entity_outside_path() {
    let mut tree = test_tree();
    let entities = spawn_entities(1);

    // Entity far to the right at (100, 50)
    let aabb = Aabb2D::new(Vec2::new(100.0, 50.0), Vec2::new(5.0, 5.0));
    tree.insert(entities[0], aabb, CollisionLayers::new(0x01, 0x01));

    // Cast circle straight up with radius=5 — should not reach entity at x=100
    let hits = tree.cast_circle(
        Vec2::new(0.0, 0.0),
        Vec2::Y,
        200.0,
        5.0,
        CollisionLayers::new(0x00, 0x01),
    );

    assert!(
        hits.is_empty(),
        "entity at x=100 should not be hit by ray along Y axis"
    );
}

// ── Behavior 8: cast_circle respects collision layers ──

#[test]
fn cast_circle_respects_collision_layers() {
    let mut tree = test_tree();
    let entities = spawn_entities(2);

    // entity_a on layer 0x01, directly in path at (0, 30)
    let aabb_a = Aabb2D::new(Vec2::new(0.0, 30.0), Vec2::new(5.0, 5.0));
    tree.insert(entities[0], aabb_a, CollisionLayers::new(0x01, 0x01));

    // entity_b on layer 0x02, also in path at (0, 60)
    let aabb_b = Aabb2D::new(Vec2::new(0.0, 60.0), Vec2::new(5.0, 5.0));
    tree.insert(entities[1], aabb_b, CollisionLayers::new(0x02, 0x02));

    // Cast with mask=0x01 — should only hit entity_a
    let hits = tree.cast_circle(
        Vec2::new(0.0, 0.0),
        Vec2::Y,
        200.0,
        5.0,
        CollisionLayers::new(0x00, 0x01),
    );

    assert_eq!(hits.len(), 1, "should only hit entity on matching layer");
    assert_eq!(
        hits[0].entity, entities[0],
        "should hit entity_a (layer 0x01)"
    );
}

// ── Behavior 9: cast_ray works (zero radius) ──

#[test]
fn cast_ray_hits_unexpanded_aabb() {
    let mut tree = test_tree();
    let entities = spawn_entities(1);

    // Entity at (0, 50) with half_extents (10, 10)
    // Unexpanded bottom face at y=40
    let aabb = Aabb2D::new(Vec2::new(0.0, 50.0), Vec2::new(10.0, 10.0));
    tree.insert(entities[0], aabb, CollisionLayers::new(0x01, 0x01));

    // cast_ray = cast_circle with radius 0
    let hits = tree.cast_ray(
        Vec2::new(0.0, 0.0),
        Vec2::Y,
        200.0,
        CollisionLayers::new(0x00, 0x01),
    );

    assert_eq!(hits.len(), 1, "ray should hit entity");
    assert_eq!(hits[0].entity, entities[0]);
    // Ray hits unexpanded AABB at y=40 (no Minkowski expansion)
    assert!(
        (hits[0].position.y - 40.0).abs() < 1.0,
        "position.y should be ~40.0 (unexpanded), got {}",
        hits[0].position.y
    );
    assert_eq!(hits[0].normal, Vec2::NEG_Y);
}
