//! Tests for gravity well visual components: `Spatial`, `Mesh2d`, `MeshMaterial2d`,
//! `GameDrawLayer::Fx`, `Scale2D` at spawn, and `sync_gravity_well_visual` system.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Scale2D, Spatial};

use super::super::effect::*;
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

// ── Behavior 16: GravityWell fire() spawns entity with Spatial marker ──

#[test]
fn fire_spawns_gravity_well_with_spatial_marker() {
    let mut app = visual_test_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(50.0, 75.0)))
        .id();

    fire(entity, 100.0, 10.0, 50.0, 4, "", app.world_mut());

    let mut query = app.world_mut().query::<(&GravityWell, &Spatial)>();
    let count = query.iter(app.world()).count();
    assert_eq!(count, 1, "expected one gravity well entity with Spatial");
}

#[test]
fn fire_with_max_zero_spawns_no_entity() {
    let mut app = visual_test_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(50.0, 75.0)))
        .id();

    fire(entity, 100.0, 10.0, 50.0, 0, "", app.world_mut());

    let mut query = app.world_mut().query::<&GravityWell>();
    let count = query.iter(app.world()).count();
    assert_eq!(count, 0, "max=0 should not spawn any entity");
}

// ── Behavior 17: GravityWell fire() spawns entity with Mesh2d ──

#[test]
fn fire_spawns_gravity_well_with_mesh2d() {
    let mut app = visual_test_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(50.0, 75.0)))
        .id();

    fire(entity, 100.0, 10.0, 50.0, 4, "", app.world_mut());

    let mut query = app.world_mut().query::<(&GravityWell, &Mesh2d)>();
    let count = query.iter(app.world()).count();
    assert_eq!(count, 1, "expected one gravity well entity with Mesh2d");
}

#[test]
fn fire_twice_spawns_two_wells_each_with_mesh2d() {
    let mut app = visual_test_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(50.0, 75.0)))
        .id();

    fire(entity, 100.0, 10.0, 50.0, 4, "", app.world_mut());
    fire(entity, 100.0, 10.0, 50.0, 4, "", app.world_mut());

    let mut query = app.world_mut().query::<(&GravityWell, &Mesh2d)>();
    let count = query.iter(app.world()).count();
    assert_eq!(
        count, 2,
        "two fire() calls should produce two wells with Mesh2d"
    );
}

// ── Behavior 18: GravityWell fire() spawns entity with MeshMaterial2d<ColorMaterial> ──

#[test]
fn fire_spawns_gravity_well_with_mesh_material() {
    let mut app = visual_test_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(50.0, 75.0)))
        .id();

    fire(entity, 100.0, 10.0, 50.0, 4, "", app.world_mut());

    let mut query = app
        .world_mut()
        .query::<(&GravityWell, &MeshMaterial2d<ColorMaterial>)>();
    let count = query.iter(app.world()).count();
    assert_eq!(
        count, 1,
        "expected one gravity well entity with MeshMaterial2d<ColorMaterial>"
    );
}

// ── Behavior 19: GravityWell fire() spawns entity with GameDrawLayer::Fx ──

#[test]
fn fire_spawns_gravity_well_with_game_draw_layer_fx() {
    let mut app = visual_test_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(50.0, 75.0)))
        .id();

    fire(entity, 100.0, 10.0, 50.0, 4, "", app.world_mut());

    let mut query = app.world_mut().query::<(&GravityWell, &GameDrawLayer)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one gravity well with GameDrawLayer"
    );

    let (_, draw_layer) = results[0];
    assert!(
        matches!(draw_layer, GameDrawLayer::Fx),
        "expected GameDrawLayer::Fx, got {draw_layer:?}",
    );
}

// Note: Behavior 20 (Position2D matching source) is already tested in fire_tests.rs
// and would pass in RED. Omitted to satisfy RED gate requirement.

// ── Behavior 21: GravityWell fire() spawns entity with Scale2D matching radius ──

#[test]
fn fire_spawns_gravity_well_with_scale_matching_radius() {
    let mut app = visual_test_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(50.0, 75.0)))
        .id();

    fire(entity, 100.0, 10.0, 50.0, 4, "", app.world_mut());

    let mut query = app.world_mut().query::<(&GravityWell, &Scale2D)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "expected one gravity well with Scale2D");

    let (_, scale) = results[0];
    assert!(
        (scale.x - 50.0).abs() < f32::EPSILON,
        "expected Scale2D.x=50.0 matching radius, got {}",
        scale.x
    );
    assert!(
        (scale.y - 50.0).abs() < f32::EPSILON,
        "expected Scale2D.y=50.0 matching radius, got {}",
        scale.y
    );
}

#[test]
fn fire_spawns_gravity_well_with_scale_matching_small_radius() {
    let mut app = visual_test_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(50.0, 75.0)))
        .id();

    fire(entity, 100.0, 10.0, 1.0, 4, "", app.world_mut());

    let mut query = app.world_mut().query::<(&GravityWell, &Scale2D)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1);

    let (_, scale) = results[0];
    assert!(
        (scale.x - 1.0).abs() < f32::EPSILON,
        "expected Scale2D.x=1.0 for radius=1.0, got {}",
        scale.x
    );
    assert!(
        (scale.y - 1.0).abs() < f32::EPSILON,
        "expected Scale2D.y=1.0 for radius=1.0, got {}",
        scale.y
    );
}

// ── Behavior 22: sync_gravity_well_visual updates Scale2D from config.radius ──

#[test]
fn sync_gravity_well_visual_updates_scale_from_config_radius() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, sync_gravity_well_visual);

    // Need a dummy owner entity
    let owner = app.world_mut().spawn_empty().id();

    app.world_mut().spawn((
        GravityWell,
        GravityWellConfig {
            strength: 100.0,
            radius: 60.0,
            remaining: 5.0,
            owner,
        },
        Spatial::builder().at_position(Vec2::ZERO).build(),
        Scale2D { x: 1.0, y: 1.0 },
    ));

    app.update();

    let mut query = app.world_mut().query::<(&GravityWell, &Scale2D)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1);

    let (_, scale) = results[0];
    assert!(
        (scale.x - 60.0).abs() < f32::EPSILON,
        "expected Scale2D.x=60.0 matching config.radius, got {}",
        scale.x
    );
    assert!(
        (scale.y - 60.0).abs() < f32::EPSILON,
        "expected Scale2D.y=60.0 matching config.radius, got {}",
        scale.y
    );
}

// ── Behavior 23: sync_gravity_well_visual handles multiple wells independently ──

#[test]
fn sync_gravity_well_visual_handles_multiple_wells_independently() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, sync_gravity_well_visual);

    let owner = app.world_mut().spawn_empty().id();

    let well_a = app
        .world_mut()
        .spawn((
            GravityWell,
            GravityWellConfig {
                strength: 100.0,
                radius: 40.0,
                remaining: 5.0,
                owner,
            },
            Spatial::builder().at_position(Vec2::ZERO).build(),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    let well_b = app
        .world_mut()
        .spawn((
            GravityWell,
            GravityWellConfig {
                strength: 100.0,
                radius: 80.0,
                remaining: 5.0,
                owner,
            },
            Spatial::builder().at_position(Vec2::ZERO).build(),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    app.update();

    let scale_a = app.world().get::<Scale2D>(well_a).unwrap();
    assert!(
        (scale_a.x - 40.0).abs() < f32::EPSILON,
        "well_a Scale2D.x should be 40.0, got {}",
        scale_a.x
    );
    assert!(
        (scale_a.y - 40.0).abs() < f32::EPSILON,
        "well_a Scale2D.y should be 40.0, got {}",
        scale_a.y
    );

    let scale_b = app.world().get::<Scale2D>(well_b).unwrap();
    assert!(
        (scale_b.x - 80.0).abs() < f32::EPSILON,
        "well_b Scale2D.x should be 80.0, got {}",
        scale_b.x
    );
    assert!(
        (scale_b.y - 80.0).abs() < f32::EPSILON,
        "well_b Scale2D.y should be 80.0, got {}",
        scale_b.y
    );
}

#[test]
fn sync_gravity_well_visual_syncs_near_expiry() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, sync_gravity_well_visual);

    let owner = app.world_mut().spawn_empty().id();

    app.world_mut().spawn((
        GravityWell,
        GravityWellConfig {
            strength: 100.0,
            radius: 40.0,
            remaining: 0.01,
            owner,
        },
        Spatial::builder().at_position(Vec2::ZERO).build(),
        Scale2D { x: 1.0, y: 1.0 },
    ));

    app.update();

    let mut query = app.world_mut().query::<(&GravityWell, &Scale2D)>();
    let scale = query.iter(app.world()).next().unwrap().1;
    assert!(
        (scale.x - 40.0).abs() < f32::EPSILON,
        "near-expiry well should still sync Scale2D.x to 40.0, got {}",
        scale.x
    );
}
