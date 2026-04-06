//! Tests for shockwave visual components: `Spatial`, `Mesh2d`, `MeshMaterial2d`,
//! `GameDrawLayer::Fx`, `Scale2D` at spawn, and `sync_shockwave_visual` system.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Scale2D, Spatial};

use super::helpers::*;
use crate::shared::GameDrawLayer;

// ── Helper: App with asset resources for fire()-based visual tests ──

fn visual_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Mesh>();
    app.init_asset::<ColorMaterial>();
    app.update();
    app
}

// ── Behavior 1: Shockwave fire() spawns entity with Spatial marker ──

#[test]
fn fire_spawns_shockwave_with_spatial_marker() {
    let mut app = visual_test_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(100.0, 200.0)))
        .id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", app.world_mut());

    let mut query = app.world_mut().query::<(&ShockwaveSource, &Spatial)>();
    let count = query.iter(app.world()).count();
    assert_eq!(count, 1, "expected one shockwave entity with Spatial");
}

#[test]
fn fire_twice_spawns_two_entities_each_with_spatial_marker() {
    let mut app = visual_test_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(100.0, 200.0)))
        .id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", app.world_mut());
    fire(entity, 24.0, 8.0, 1, 50.0, "", app.world_mut());

    let mut query = app.world_mut().query::<(&ShockwaveSource, &Spatial)>();
    let count = query.iter(app.world()).count();
    assert_eq!(
        count, 2,
        "two fire() calls should produce two entities with Spatial"
    );
}

// ── Behavior 2: Shockwave fire() spawns entity with Mesh2d ──

#[test]
fn fire_spawns_shockwave_with_mesh2d() {
    let mut app = visual_test_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(100.0, 200.0)))
        .id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", app.world_mut());

    let mut query = app.world_mut().query::<(&ShockwaveSource, &Mesh2d)>();
    let count = query.iter(app.world()).count();
    assert_eq!(count, 1, "expected one shockwave entity with Mesh2d");
}

#[test]
fn fire_twice_spawns_two_entities_each_with_mesh2d() {
    let mut app = visual_test_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(100.0, 200.0)))
        .id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", app.world_mut());
    fire(entity, 24.0, 8.0, 1, 50.0, "", app.world_mut());

    let mut query = app.world_mut().query::<(&ShockwaveSource, &Mesh2d)>();
    let count = query.iter(app.world()).count();
    assert_eq!(
        count, 2,
        "two fire() calls should produce two entities with Mesh2d"
    );
}

// ── Behavior 3: Shockwave fire() spawns entity with MeshMaterial2d<ColorMaterial> ──

#[test]
fn fire_spawns_shockwave_with_mesh_material() {
    let mut app = visual_test_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(100.0, 200.0)))
        .id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", app.world_mut());

    let mut query = app
        .world_mut()
        .query::<(&ShockwaveSource, &MeshMaterial2d<ColorMaterial>)>();
    let count = query.iter(app.world()).count();
    assert_eq!(
        count, 1,
        "expected one shockwave entity with MeshMaterial2d<ColorMaterial>"
    );
}

#[test]
fn fire_twice_spawns_two_entities_each_with_mesh_material() {
    let mut app = visual_test_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(100.0, 200.0)))
        .id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", app.world_mut());
    fire(entity, 24.0, 8.0, 1, 50.0, "", app.world_mut());

    let mut query = app
        .world_mut()
        .query::<(&ShockwaveSource, &MeshMaterial2d<ColorMaterial>)>();
    let count = query.iter(app.world()).count();
    assert_eq!(
        count, 2,
        "two fire() calls should produce two entities with MeshMaterial2d"
    );
}

// ── Behavior 4: Shockwave fire() spawns entity with GameDrawLayer::Fx ──

#[test]
fn fire_spawns_shockwave_with_game_draw_layer_fx() {
    let mut app = visual_test_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(100.0, 200.0)))
        .id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", app.world_mut());

    let mut query = app
        .world_mut()
        .query::<(&ShockwaveSource, &GameDrawLayer)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one shockwave entity with GameDrawLayer"
    );

    let (_, draw_layer) = results[0];
    assert!(
        matches!(draw_layer, GameDrawLayer::Fx),
        "expected GameDrawLayer::Fx, got {draw_layer:?}",
    );
}

// Note: Behavior 5 (Position2D matching source) is already tested in fire_tests.rs
// and would pass in RED. Omitted to satisfy RED gate requirement.

// ── Behavior 6: Shockwave fire() spawns entity with Scale2D { x: 0.0, y: 0.0 } ──

#[test]
fn fire_spawns_shockwave_with_initial_scale_zero() {
    let mut app = visual_test_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(100.0, 200.0)))
        .id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", app.world_mut());

    let mut query = app.world_mut().query::<(&ShockwaveSource, &Scale2D)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one shockwave entity with Scale2D"
    );

    let (_, scale) = results[0];
    // Spatial required chain defaults Scale2D to { x: 1.0, y: 1.0 }.
    // The explicit override should set it to { x: 0.0, y: 0.0 }.
    assert!(
        scale.x.abs() < f32::EPSILON,
        "expected Scale2D.x=0.0 (overriding Spatial default of 1.0), got {}",
        scale.x
    );
    assert!(
        scale.y.abs() < f32::EPSILON,
        "expected Scale2D.y=0.0 (overriding Spatial default of 1.0), got {}",
        scale.y
    );
}

// ── Behavior 7: sync_shockwave_visual updates Scale2D to match ShockwaveRadius ──

#[test]
fn sync_shockwave_visual_updates_scale_to_match_radius() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, sync_shockwave_visual);

    app.world_mut().spawn((
        ShockwaveSource,
        ShockwaveRadius(45.0),
        Spatial::builder().at_position(Vec2::ZERO).build(),
        Scale2D { x: 0.0, y: 0.0 },
    ));

    app.update();

    let mut query = app.world_mut().query::<(&ShockwaveSource, &Scale2D)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1);

    let (_, scale) = results[0];
    assert!(
        (scale.x - 45.0).abs() < f32::EPSILON,
        "expected Scale2D.x=45.0 matching ShockwaveRadius, got {}",
        scale.x
    );
    assert!(
        (scale.y - 45.0).abs() < f32::EPSILON,
        "expected Scale2D.y=45.0 matching ShockwaveRadius, got {}",
        scale.y
    );
}

#[test]
fn sync_shockwave_visual_handles_zero_radius() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, sync_shockwave_visual);

    app.world_mut().spawn((
        ShockwaveSource,
        ShockwaveRadius(0.0),
        Spatial::builder().at_position(Vec2::ZERO).build(),
        Scale2D { x: 10.0, y: 10.0 },
    ));

    app.update();

    let mut query = app.world_mut().query::<(&ShockwaveSource, &Scale2D)>();
    let scale = query.iter(app.world()).next().unwrap().1;
    assert!(
        scale.x.abs() < f32::EPSILON,
        "expected Scale2D.x=0.0 for zero radius, got {}",
        scale.x
    );
    assert!(
        scale.y.abs() < f32::EPSILON,
        "expected Scale2D.y=0.0 for zero radius, got {}",
        scale.y
    );
}

// ── Behavior 8: sync_shockwave_visual handles multiple shockwaves independently ──

#[test]
fn sync_shockwave_visual_handles_multiple_shockwaves_independently() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, sync_shockwave_visual);

    let entity_a = app
        .world_mut()
        .spawn((
            ShockwaveSource,
            ShockwaveRadius(10.0),
            Spatial::builder().at_position(Vec2::ZERO).build(),
            Scale2D { x: 0.0, y: 0.0 },
        ))
        .id();

    let entity_b = app
        .world_mut()
        .spawn((
            ShockwaveSource,
            ShockwaveRadius(30.0),
            Spatial::builder().at_position(Vec2::ZERO).build(),
            Scale2D { x: 0.0, y: 0.0 },
        ))
        .id();

    app.update();

    let scale_a = app.world().get::<Scale2D>(entity_a).unwrap();
    assert!(
        (scale_a.x - 10.0).abs() < f32::EPSILON,
        "entity_a Scale2D.x should be 10.0, got {}",
        scale_a.x
    );
    assert!(
        (scale_a.y - 10.0).abs() < f32::EPSILON,
        "entity_a Scale2D.y should be 10.0, got {}",
        scale_a.y
    );

    let scale_b = app.world().get::<Scale2D>(entity_b).unwrap();
    assert!(
        (scale_b.x - 30.0).abs() < f32::EPSILON,
        "entity_b Scale2D.x should be 30.0, got {}",
        scale_b.x
    );
    assert!(
        (scale_b.y - 30.0).abs() < f32::EPSILON,
        "entity_b Scale2D.y should be 30.0, got {}",
        scale_b.y
    );
}
