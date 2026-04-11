//! Tests for pulse ring visual components: `Spatial`, `Mesh2d`, `MeshMaterial2d`,
//! `GameDrawLayer::Fx`, `Scale2D` at spawn, and `sync_pulse_visual` system.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Scale2D, Spatial};

use super::helpers::*;
use crate::shared::GameDrawLayer;

// ── Helper: App with asset resources + tick_pulse_emitter for spawn tests ──

fn visual_emitter_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Mesh>();
    app.init_asset::<ColorMaterial>();
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<crate::state::types::AppState>();
    app.add_sub_state::<crate::state::types::GameState>();
    app.add_sub_state::<crate::state::types::RunState>();
    app.add_sub_state::<crate::state::types::NodeState>();
    app.add_systems(Update, tick_pulse_emitter);
    app
}

/// Spawns a `PulseEmitter` bolt entity with timer nearly elapsed.
fn spawn_emitter(app: &mut App, position: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Position2D(position),
            PulseEmitter {
                base_range: 32.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 50.0,
                interval: 0.5,
                timer: 0.49,
            },
        ))
        .id()
}

// ── Behavior 9: tick_pulse_emitter spawns PulseRing with Spatial marker ──

#[test]
fn tick_pulse_emitter_spawns_ring_with_spatial_marker() {
    let mut app = visual_emitter_test_app();
    enter_playing(&mut app);
    spawn_emitter(&mut app, Vec2::new(80.0, 120.0));

    app.update();

    let mut query = app.world_mut().query::<(&PulseRing, &Spatial)>();
    let count = query.iter(app.world()).count();
    assert_eq!(
        count, 1,
        "expected one PulseRing entity with Spatial marker"
    );
}

// ── Behavior 10: tick_pulse_emitter spawns PulseRing with Mesh2d ──

#[test]
fn tick_pulse_emitter_spawns_ring_with_mesh2d() {
    let mut app = visual_emitter_test_app();
    enter_playing(&mut app);
    spawn_emitter(&mut app, Vec2::new(80.0, 120.0));

    app.update();

    let mut query = app.world_mut().query::<(&PulseRing, &Mesh2d)>();
    let count = query.iter(app.world()).count();
    assert_eq!(count, 1, "expected one PulseRing entity with Mesh2d");
}

// ── Behavior 11: tick_pulse_emitter spawns PulseRing with MeshMaterial2d ──

#[test]
fn tick_pulse_emitter_spawns_ring_with_mesh_material() {
    let mut app = visual_emitter_test_app();
    enter_playing(&mut app);
    spawn_emitter(&mut app, Vec2::new(80.0, 120.0));

    app.update();

    let mut query = app
        .world_mut()
        .query::<(&PulseRing, &MeshMaterial2d<ColorMaterial>)>();
    let count = query.iter(app.world()).count();
    assert_eq!(
        count, 1,
        "expected one PulseRing entity with MeshMaterial2d<ColorMaterial>"
    );
}

// ── Behavior 12: tick_pulse_emitter spawns PulseRing with GameDrawLayer::Fx ──

#[test]
fn tick_pulse_emitter_spawns_ring_with_game_draw_layer_fx() {
    let mut app = visual_emitter_test_app();
    enter_playing(&mut app);
    spawn_emitter(&mut app, Vec2::new(80.0, 120.0));

    app.update();

    let mut query = app.world_mut().query::<(&PulseRing, &GameDrawLayer)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one PulseRing with GameDrawLayer"
    );

    let (_, draw_layer) = results[0];
    assert!(
        matches!(draw_layer, GameDrawLayer::Fx),
        "expected GameDrawLayer::Fx, got {draw_layer:?}",
    );
}

// Note: Behavior 13 (Position2D matching emitter) is already tested in emitter_tests.rs
// and would pass in RED. Omitted to satisfy RED gate requirement.

// ── Behavior 14: tick_pulse_emitter spawns PulseRing with Scale2D { x: 0.0, y: 0.0 } ──

#[test]
fn tick_pulse_emitter_spawns_ring_with_initial_scale_zero() {
    let mut app = visual_emitter_test_app();
    enter_playing(&mut app);
    spawn_emitter(&mut app, Vec2::new(80.0, 120.0));

    app.update();

    let mut query = app.world_mut().query::<(&PulseRing, &Scale2D)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "expected one PulseRing with Scale2D");

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

// ── Behavior 15: sync_pulse_visual updates Scale2D to match PulseRadius ──

#[test]
fn sync_pulse_visual_updates_scale_to_match_radius() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, sync_pulse_visual);

    app.world_mut().spawn((
        PulseRing,
        PulseRadius(25.0),
        Spatial::builder().at_position(Vec2::ZERO).build(),
        Scale2D { x: 0.0, y: 0.0 },
    ));

    app.update();

    let mut query = app.world_mut().query::<(&PulseRing, &Scale2D)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1);

    let (_, scale) = results[0];
    assert!(
        (scale.x - 25.0).abs() < f32::EPSILON,
        "expected Scale2D.x=25.0 matching PulseRadius, got {}",
        scale.x
    );
    assert!(
        (scale.y - 25.0).abs() < f32::EPSILON,
        "expected Scale2D.y=25.0 matching PulseRadius, got {}",
        scale.y
    );
}

#[test]
fn sync_pulse_visual_handles_zero_radius() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, sync_pulse_visual);

    app.world_mut().spawn((
        PulseRing,
        PulseRadius(0.0),
        Spatial::builder().at_position(Vec2::ZERO).build(),
        Scale2D { x: 10.0, y: 10.0 },
    ));

    app.update();

    let mut query = app.world_mut().query::<(&PulseRing, &Scale2D)>();
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
