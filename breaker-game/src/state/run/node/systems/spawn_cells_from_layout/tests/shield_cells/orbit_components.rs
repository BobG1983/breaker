//! Tests for orbit child components: count, `CellHealth`, `Scale2D`, `Aabb2D`,
//! `CollisionLayers`, `OrbitAngle`, and `OrbitConfig`.

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::Scale2D;

use super::helpers::*;
use crate::{
    cells::components::*,
    shared::{BOLT_LAYER, CELL_LAYER},
};

// -- Behavior 2: Orbit children spawned with correct components --

#[test]
fn orbit_cells_spawned_with_correct_count() {
    // Given: shield with orbit_count=3
    // When: spawn runs
    // Then: 3 OrbitCell entities exist
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    let orbit_count = app
        .world_mut()
        .query::<(&Cell, &OrbitCell)>()
        .iter(app.world())
        .count();
    assert_eq!(
        orbit_count, 3,
        "should spawn 3 orbit cells, got {orbit_count}"
    );
}

#[test]
fn orbit_cells_have_cell_health_from_shield_behavior() {
    // Given: shield with orbit_hp=10.0
    // When: spawn runs
    // Then: each OrbitCell has CellHealth { current: 10.0, max: 10.0 }
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    for health in app
        .world_mut()
        .query_filtered::<&CellHealth, With<OrbitCell>>()
        .iter(app.world())
    {
        assert!(
            (health.current - 10.0).abs() < f32::EPSILON,
            "orbit cell current HP should be 10.0, got {}",
            health.current
        );
        assert!(
            (health.max - 10.0).abs() < f32::EPSILON,
            "orbit cell max HP should be 10.0, got {}",
            health.max
        );
    }
}

#[test]
fn orbit_cells_have_square_20x20_scale() {
    // Given: orbit dimensions 20.0 x 20.0
    // When: spawn runs
    // Then: each OrbitCell has Scale2D { x: 20.0, y: 20.0 }
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    for scale in app
        .world_mut()
        .query_filtered::<&Scale2D, With<OrbitCell>>()
        .iter(app.world())
    {
        assert!(
            (scale.x - 20.0).abs() < f32::EPSILON,
            "orbit cell Scale2D.x should be 20.0, got {}",
            scale.x
        );
        assert!(
            (scale.y - 20.0).abs() < f32::EPSILON,
            "orbit cell Scale2D.y should be 20.0, got {}",
            scale.y
        );
    }
}

#[test]
fn orbit_cells_have_aabb2d_matching_square_size() {
    // Given: orbit dimensions 20.0 x 20.0
    // When: spawn runs
    // Then: each OrbitCell has Aabb2D with half_extents (10.0, 10.0)
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    for aabb in app
        .world_mut()
        .query_filtered::<&Aabb2D, With<OrbitCell>>()
        .iter(app.world())
    {
        assert_eq!(
            aabb.center,
            Vec2::ZERO,
            "orbit cell Aabb2D center should be ZERO"
        );
        assert!(
            (aabb.half_extents.x - 10.0).abs() < f32::EPSILON,
            "orbit cell Aabb2D half_extents.x should be 10.0, got {}",
            aabb.half_extents.x
        );
        assert!(
            (aabb.half_extents.y - 10.0).abs() < f32::EPSILON,
            "orbit cell Aabb2D half_extents.y should be 10.0, got {}",
            aabb.half_extents.y
        );
    }
}

#[test]
fn orbit_cells_have_collision_layers() {
    // Given: orbit cells are collidable
    // When: spawn runs
    // Then: each OrbitCell has CollisionLayers { membership: CELL_LAYER, mask: BOLT_LAYER }

    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    for layers in app
        .world_mut()
        .query_filtered::<&CollisionLayers, With<OrbitCell>>()
        .iter(app.world())
    {
        assert_eq!(
            layers.membership, CELL_LAYER,
            "orbit cell membership should be CELL_LAYER"
        );
        assert_eq!(
            layers.mask, BOLT_LAYER,
            "orbit cell mask should be BOLT_LAYER"
        );
    }
}

#[test]
fn orbit_cells_have_orbit_angle_evenly_spaced() {
    // Given: shield with orbit_count=3
    // When: spawn runs
    // Then: OrbitAngle values are 0, 2*PI/3, 4*PI/3 (evenly spaced)
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    let mut angles: Vec<f32> = app
        .world_mut()
        .query_filtered::<&OrbitAngle, With<OrbitCell>>()
        .iter(app.world())
        .map(|a| a.0)
        .collect();
    angles.sort_by(f32::total_cmp);

    assert_eq!(angles.len(), 3, "should have 3 orbit angles");
    let expected = [
        0.0,
        2.0 * std::f32::consts::PI / 3.0,
        4.0 * std::f32::consts::PI / 3.0,
    ];
    for (i, (actual, exp)) in angles.iter().zip(expected.iter()).enumerate() {
        assert!(
            (actual - exp).abs() < 1e-5,
            "orbit {i} angle should be {exp:.4}, got {actual:.4}"
        );
    }
}

#[test]
fn orbit_cells_have_orbit_config_from_shield_behavior() {
    // Given: shield with orbit_radius=60.0, orbit_speed=PI/2
    // When: spawn runs
    // Then: each OrbitCell has OrbitConfig { radius: 60.0, speed: PI/2 }
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    for config in app
        .world_mut()
        .query_filtered::<&OrbitConfig, With<OrbitCell>>()
        .iter(app.world())
    {
        assert!(
            (config.radius - 60.0).abs() < f32::EPSILON,
            "orbit config radius should be 60.0, got {}",
            config.radius
        );
        assert!(
            (config.speed - std::f32::consts::FRAC_PI_2).abs() < f32::EPSILON,
            "orbit config speed should be PI/2, got {}",
            config.speed
        );
    }
}
