//! Tests for explode flash visual entity spawning.

use bevy::prelude::*;
use rantzsoft_stateflow::CleanupOnExit;
use rantzsoft_spatial2d::components::{Position2D, Scale2D};

use super::helpers::*;
use crate::{
    effect::effects::explode::ExplodeRequest, fx::EffectFlashTimer, shared::GameDrawLayer,
    state::types::NodeState,
};

// ── Test app with asset support ────────────────────────────────────

fn flash_test_app() -> App {
    let mut app = damage_test_app();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<ColorMaterial>>();
    app
}

fn spawn_flash_explode_request(app: &mut App, x: f32, y: f32, range: f32, damage: f32) -> Entity {
    app.world_mut()
        .spawn((
            ExplodeRequest { range, damage },
            Position2D(Vec2::new(x, y)),
        ))
        .id()
}

// ── Behavior 14: process_explode_requests spawns flash visual entity ──

#[test]
fn process_explode_requests_spawns_flash_visual_entity_with_required_components() {
    let mut app = flash_test_app();

    let request = spawn_flash_explode_request(&mut app, 50.0, 75.0, 40.0, 15.0);

    tick(&mut app);

    // Request should be despawned
    assert!(
        app.world().get_entity(request).is_err(),
        "ExplodeRequest should be despawned after processing"
    );

    // Query for flash entities by EffectFlashTimer
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<EffectFlashTimer>>();
    let flash_entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(
        flash_entities.len(),
        1,
        "expected exactly 1 flash entity, got {}",
        flash_entities.len()
    );

    let flash = flash_entities[0];

    // Check all required components
    assert!(
        app.world().get::<Mesh2d>(flash).is_some(),
        "flash entity should have Mesh2d"
    );
    assert!(
        app.world()
            .get::<MeshMaterial2d<ColorMaterial>>(flash)
            .is_some(),
        "flash entity should have MeshMaterial2d<ColorMaterial>"
    );
    assert!(
        app.world().get::<CleanupOnExit<NodeState>>(flash).is_some(),
        "flash entity should have CleanupOnExit<NodeState>"
    );
    assert!(
        matches!(
            app.world().get::<GameDrawLayer>(flash),
            Some(GameDrawLayer::Fx)
        ),
        "flash entity should have GameDrawLayer::Fx"
    );
}

#[test]
fn explode_flash_spawns_with_no_cells_in_range() {
    let mut app = flash_test_app();

    let request = spawn_flash_explode_request(&mut app, 0.0, 0.0, 50.0, 10.0);

    tick(&mut app);

    assert!(
        app.world().get_entity(request).is_err(),
        "request should be despawned"
    );

    let mut flash_query = app
        .world_mut()
        .query_filtered::<Entity, With<EffectFlashTimer>>();
    let flash_count = flash_query.iter(app.world()).count();
    assert_eq!(
        flash_count, 1,
        "flash entity should be spawned even with no cells, got {flash_count}"
    );
}

// ── Behavior 15: Flash entity has EffectFlashTimer(0.2) ──

#[test]
fn explode_flash_has_timer_value_0_2() {
    let mut app = flash_test_app();

    spawn_flash_explode_request(&mut app, 0.0, 0.0, 50.0, 10.0);

    tick(&mut app);

    let mut query = app.world_mut().query::<&EffectFlashTimer>();
    let timer = query
        .iter(app.world())
        .next()
        .expect("flash entity should exist");
    assert!(
        (timer.0 - 0.2).abs() < f32::EPSILON,
        "EffectFlashTimer should be 0.2, got {}",
        timer.0
    );
}

// ── Behavior 16: Flash positioned at explosion origin ──

#[test]
fn explode_flash_positioned_at_explosion_origin() {
    let mut app = flash_test_app();

    spawn_flash_explode_request(&mut app, 100.0, -50.0, 30.0, 10.0);

    tick(&mut app);

    let mut query = app
        .world_mut()
        .query_filtered::<&Position2D, With<EffectFlashTimer>>();
    let pos = query
        .iter(app.world())
        .next()
        .expect("flash entity should have Position2D");
    assert!(
        (pos.0.x - 100.0).abs() < f32::EPSILON,
        "flash x should be 100.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - (-50.0)).abs() < f32::EPSILON,
        "flash y should be -50.0, got {}",
        pos.0.y
    );
}

// ── Behavior 17: Flash scaled to explosion range ──

#[test]
fn explode_flash_scale_matches_explosion_range() {
    let mut app = flash_test_app();

    spawn_flash_explode_request(&mut app, 0.0, 0.0, 60.0, 10.0);

    tick(&mut app);

    let mut query = app
        .world_mut()
        .query_filtered::<&Scale2D, With<EffectFlashTimer>>();
    let scale = query
        .iter(app.world())
        .next()
        .expect("flash entity should have Scale2D");
    assert!(
        (scale.x - 60.0).abs() < f32::EPSILON,
        "Scale2D.x should be range (60.0), got {}",
        scale.x
    );
    assert!(
        (scale.y - 60.0).abs() < f32::EPSILON,
        "Scale2D.y should be range (60.0), got {}",
        scale.y
    );
}

// ── Behavior 18: Multiple requests spawn independent flash entities ──

#[test]
fn multiple_explode_requests_spawn_independent_flash_entities() {
    let mut app = flash_test_app();

    let req_a = spawn_flash_explode_request(&mut app, 10.0, 0.0, 20.0, 10.0);
    let req_b = spawn_flash_explode_request(&mut app, 50.0, 0.0, 40.0, 10.0);

    tick(&mut app);

    // Both requests despawned
    assert!(
        app.world().get_entity(req_a).is_err(),
        "request A should be despawned"
    );
    assert!(
        app.world().get_entity(req_b).is_err(),
        "request B should be despawned"
    );

    // Two flash entities spawned
    let mut query = app.world_mut().query::<&EffectFlashTimer>();
    let timers: Vec<&EffectFlashTimer> = query.iter(app.world()).collect();
    assert_eq!(
        timers.len(),
        2,
        "expected 2 flash entities (one per request), got {}",
        timers.len()
    );

    for timer in &timers {
        assert!(
            (timer.0 - 0.2).abs() < f32::EPSILON,
            "each flash should have EffectFlashTimer(0.2), got {}",
            timer.0
        );
    }
}

// ── Behavior 19: Flash spawns even with no cells in range ──

#[test]
fn explode_flash_spawns_even_with_no_cells_in_range_and_zero_damage_messages() {
    let mut app = flash_test_app();

    let request = spawn_flash_explode_request(&mut app, 0.0, 0.0, 50.0, 10.0);

    tick(&mut app);

    assert!(
        app.world().get_entity(request).is_err(),
        "request should be despawned"
    );

    let mut flash_query = app
        .world_mut()
        .query_filtered::<Entity, With<EffectFlashTimer>>();
    let flash_count = flash_query.iter(app.world()).count();
    assert_eq!(
        flash_count, 1,
        "flash entity should be spawned even with no cells, got {flash_count}"
    );

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "no damage should be dealt with no cells in range"
    );
}
