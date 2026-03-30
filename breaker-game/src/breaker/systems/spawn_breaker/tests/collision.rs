//! Tests for `Aabb2D` dimensions and `CollisionLayers` on the spawned breaker.

use bevy::prelude::*;

use super::helpers::*;
use crate::breaker::{components::Breaker, resources::BreakerConfig};

// --- Aabb2D + CollisionLayers tests ---

#[test]
fn spawned_breaker_has_aabb2d_matching_breaker_dimensions() {
    // Given: BreakerConfig default (width=120.0, height=20.0)
    // When: spawn_breaker runs
    // Then: breaker entity has Aabb2D { center: Vec2::ZERO, half_extents: Vec2::new(60.0, 10.0) }
    use rantzsoft_physics2d::aabb::Aabb2D;

    let mut app = test_app();
    app.update();

    let config = BreakerConfig::default();
    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    let aabb = app
        .world()
        .get::<Aabb2D>(entity)
        .expect("breaker should have Aabb2D");
    assert_eq!(
        aabb.center,
        Vec2::ZERO,
        "breaker Aabb2D center should be ZERO (local space)"
    );
    let expected_half_w = config.width / 2.0; // 60.0
    let expected_half_h = config.height / 2.0; // 10.0
    assert!(
        (aabb.half_extents.x - expected_half_w).abs() < f32::EPSILON
            && (aabb.half_extents.y - expected_half_h).abs() < f32::EPSILON,
        "breaker Aabb2D half_extents should be ({expected_half_w}, {expected_half_h}), got ({}, {})",
        aabb.half_extents.x,
        aabb.half_extents.y,
    );
}

#[test]
fn spawned_breaker_has_collision_layers_breaker_membership_bolt_mask() {
    // Given: spawn_breaker runs
    // Then: CollisionLayers { membership: BREAKER_LAYER (0x08), mask: BOLT_LAYER (0x01) }
    use rantzsoft_physics2d::collision_layers::CollisionLayers;

    use crate::shared::{BOLT_LAYER, BREAKER_LAYER};

    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    let layers = app
        .world()
        .get::<CollisionLayers>(entity)
        .expect("breaker should have CollisionLayers");
    assert_eq!(
        layers.membership, BREAKER_LAYER,
        "breaker membership should be BREAKER_LAYER (0x{:02X}), got 0x{:02X}",
        BREAKER_LAYER, layers.membership,
    );
    assert_eq!(
        layers.mask, BOLT_LAYER,
        "breaker mask should be BOLT_LAYER (0x{:02X}), got 0x{:02X}",
        BOLT_LAYER, layers.mask,
    );
}
