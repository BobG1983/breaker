//! Tests for chain lightning arc visual components: `Spatial`, `Mesh2d`, `MeshMaterial2d`,
//! `GameDrawLayer::Fx`, `Scale2D`, `Position2D` at spawn and during travel.

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Scale2D, Spatial};

use super::helpers::*;
use crate::shared::GameDrawLayer;

// ── Helper: App with asset resources + tick_chain_lightning for arc spawn tests ──

fn visual_chain_test_app() -> App {
    let mut app = chain_lightning_test_app();
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Mesh>();
    app.init_asset::<ColorMaterial>();
    app.add_systems(Update, tick_chain_lightning);
    app
}

// ── Behavior 24: tick_chain_lightning spawns arc with Spatial marker ──

#[test]
fn tick_chain_lightning_spawns_arc_with_spatial_marker() {
    let mut app = visual_chain_test_app();

    let cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 40.0, 0.0);
    tick(&mut app);

    let mut hit_set = HashSet::new();
    hit_set.insert(cell_a);

    spawn_chain(
        &mut app,
        SpawnChainConfig {
            source: Vec2::new(20.0, 0.0),
            remaining_jumps: 2,
            damage: 15.0,
            range: 25.0,
            arc_speed: 200.0,
            hit_set,
            state: ChainState::Idle,
            source_chip: None,
        },
    );

    tick(&mut app);

    let mut query = app.world_mut().query::<(&ChainLightningArc, &Spatial)>();
    let count = query.iter(app.world()).count();
    assert_eq!(count, 1, "expected one arc entity with Spatial marker");
}

// ── Behavior 25: tick_chain_lightning spawns arc with Mesh2d ──

#[test]
fn tick_chain_lightning_spawns_arc_with_mesh2d() {
    let mut app = visual_chain_test_app();

    let cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 40.0, 0.0);
    tick(&mut app);

    let mut hit_set = HashSet::new();
    hit_set.insert(cell_a);

    spawn_chain(
        &mut app,
        SpawnChainConfig {
            source: Vec2::new(20.0, 0.0),
            remaining_jumps: 2,
            damage: 15.0,
            range: 25.0,
            arc_speed: 200.0,
            hit_set,
            state: ChainState::Idle,
            source_chip: None,
        },
    );

    tick(&mut app);

    let mut query = app.world_mut().query::<(&ChainLightningArc, &Mesh2d)>();
    let count = query.iter(app.world()).count();
    assert_eq!(count, 1, "expected one arc entity with Mesh2d");
}

// ── Behavior 26: tick_chain_lightning spawns arc with MeshMaterial2d<ColorMaterial> ──

#[test]
fn tick_chain_lightning_spawns_arc_with_mesh_material() {
    let mut app = visual_chain_test_app();

    let cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 40.0, 0.0);
    tick(&mut app);

    let mut hit_set = HashSet::new();
    hit_set.insert(cell_a);

    spawn_chain(
        &mut app,
        SpawnChainConfig {
            source: Vec2::new(20.0, 0.0),
            remaining_jumps: 2,
            damage: 15.0,
            range: 25.0,
            arc_speed: 200.0,
            hit_set,
            state: ChainState::Idle,
            source_chip: None,
        },
    );

    tick(&mut app);

    let mut query = app
        .world_mut()
        .query::<(&ChainLightningArc, &MeshMaterial2d<ColorMaterial>)>();
    let count = query.iter(app.world()).count();
    assert_eq!(
        count, 1,
        "expected one arc entity with MeshMaterial2d<ColorMaterial>"
    );
}

// ── Behavior 27: tick_chain_lightning spawns arc with GameDrawLayer::Fx ──

#[test]
fn tick_chain_lightning_spawns_arc_with_game_draw_layer_fx() {
    let mut app = visual_chain_test_app();

    let cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 40.0, 0.0);
    tick(&mut app);

    let mut hit_set = HashSet::new();
    hit_set.insert(cell_a);

    spawn_chain(
        &mut app,
        SpawnChainConfig {
            source: Vec2::new(20.0, 0.0),
            remaining_jumps: 2,
            damage: 15.0,
            range: 25.0,
            arc_speed: 200.0,
            hit_set,
            state: ChainState::Idle,
            source_chip: None,
        },
    );

    tick(&mut app);

    let mut query = app
        .world_mut()
        .query::<(&ChainLightningArc, &GameDrawLayer)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "expected one arc with GameDrawLayer");

    let (_, draw_layer) = results[0];
    assert!(
        matches!(draw_layer, GameDrawLayer::Fx),
        "expected GameDrawLayer::Fx, got {draw_layer:?}",
    );
}

// ── Behavior 28: tick_chain_lightning spawns arc with Position2D at source ──

#[test]
fn tick_chain_lightning_spawns_arc_with_position_at_source() {
    let mut app = visual_chain_test_app();

    let cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 40.0, 0.0);
    tick(&mut app);

    let mut hit_set = HashSet::new();
    hit_set.insert(cell_a);

    spawn_chain(
        &mut app,
        SpawnChainConfig {
            source: Vec2::new(20.0, 0.0),
            remaining_jumps: 2,
            damage: 15.0,
            range: 25.0,
            arc_speed: 200.0,
            hit_set,
            state: ChainState::Idle,
            source_chip: None,
        },
    );

    tick(&mut app);

    let mut query = app.world_mut().query::<(&ChainLightningArc, &Position2D)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "expected one arc entity");

    let (_, pos) = results[0];
    assert!(
        (pos.0.x - 20.0).abs() < 0.01,
        "expected arc Position2D.x=20.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - 0.0).abs() < 0.01,
        "expected arc Position2D.y=0.0, got {}",
        pos.0.y
    );
}

// ── Behavior 29: tick_chain_lightning spawns arc with Scale2D { x: 6.0, y: 6.0 } ──

#[test]
fn tick_chain_lightning_spawns_arc_with_fixed_scale() {
    let mut app = visual_chain_test_app();

    let cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 40.0, 0.0);
    tick(&mut app);

    let mut hit_set = HashSet::new();
    hit_set.insert(cell_a);

    spawn_chain(
        &mut app,
        SpawnChainConfig {
            source: Vec2::new(20.0, 0.0),
            remaining_jumps: 2,
            damage: 15.0,
            range: 25.0,
            arc_speed: 200.0,
            hit_set,
            state: ChainState::Idle,
            source_chip: None,
        },
    );

    tick(&mut app);

    let mut query = app.world_mut().query::<(&ChainLightningArc, &Scale2D)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "expected one arc with Scale2D");

    let (_, scale) = results[0];
    assert!(
        (scale.x - 6.0).abs() < f32::EPSILON,
        "expected Scale2D.x=6.0 for fixed arc dot size, got {}",
        scale.x
    );
    assert!(
        (scale.y - 6.0).abs() < f32::EPSILON,
        "expected Scale2D.y=6.0 for fixed arc dot size, got {}",
        scale.y
    );
}

// ── Behavior 30: tick_chain_lightning updates Position2D when advancing arc ──

#[test]
fn tick_chain_lightning_updates_position2d_when_arc_traveling() {
    let mut app = visual_chain_test_app();

    let cell = spawn_test_cell(&mut app, 120.0, 0.0);
    tick(&mut app);

    // Spawn arc entity at (20, 0)
    let arc = spawn_arc(&mut app, Vec2::new(20.0, 0.0));

    // Spawn chain in ArcTraveling state pointing at (120, 0)
    spawn_chain(
        &mut app,
        SpawnChainConfig {
            source: Vec2::new(20.0, 0.0),
            remaining_jumps: 2,
            damage: 15.0,
            range: 25.0,
            arc_speed: 200.0,
            hit_set: HashSet::new(),
            state: ChainState::ArcTraveling {
                target: cell,
                target_pos: Vec2::new(120.0, 0.0),
                arc_entity: arc,
                arc_pos: Vec2::new(20.0, 0.0),
            },
            source_chip: None,
        },
    );

    tick(&mut app);

    // Arc should move toward (120, 0) by arc_speed * dt
    // dt = 1/64 = 0.015625, distance_per_tick = 200.0 * 0.015625 = 3.125
    let arc_pos = app.world().get::<Position2D>(arc).unwrap();
    assert!(
        arc_pos.0.x > 20.0 && arc_pos.0.x < 120.0,
        "arc Position2D.x should be between 20.0 and 120.0 after traveling, got {}",
        arc_pos.0.x
    );
    assert!(
        (arc_pos.0.y - 0.0).abs() < 0.01,
        "arc Position2D.y should remain ~0.0, got {}",
        arc_pos.0.y
    );
}
