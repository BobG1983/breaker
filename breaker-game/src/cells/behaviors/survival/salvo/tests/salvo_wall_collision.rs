//! Tests for `salvo_wall_collision` — behaviors 39-42.
//!
//! Wall collision uses `PlayfieldConfig` boundaries (not wall entities).
//! Default `PlayfieldConfig`: width=800, height=600.
//! Boundaries: left=-400, right=400, bottom=-300, top=300.

use bevy::prelude::*;

use super::helpers::*;
use crate::prelude::*;

// ── Behavior 39: Salvo below bottom boundary is despawned ──

#[test]
fn salvo_below_bottom_boundary_is_despawned() {
    let mut app = build_salvo_wall_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    // Default playfield: bottom = -300.0. Place salvo below.
    let salvo = spawn_salvo(
        &mut app,
        Vec2::new(0.0, -305.0), // below bottom boundary
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    assert!(
        app.world().get_entity(salvo).is_err(),
        "salvo below bottom boundary should be despawned"
    );
}

// ── Behavior 40: Salvo inside boundaries survives ──

#[test]
fn salvo_inside_boundaries_survives() {
    let mut app = build_salvo_wall_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    let salvo = spawn_salvo(
        &mut app,
        Vec2::new(0.0, 0.0), // center of playfield
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    assert!(
        app.world().get_entity(salvo).is_ok(),
        "salvo inside boundaries should survive"
    );
}

// ── Additional boundary checks ──

#[test]
fn salvo_beyond_left_boundary_is_despawned() {
    let mut app = build_salvo_wall_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    // Default playfield: left = -400.0. Place salvo beyond left edge.
    let salvo = spawn_salvo(
        &mut app,
        Vec2::new(-405.0, 0.0),
        Vec2::new(-50.0, -300.0),
        5.0,
        turret,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    assert!(
        app.world().get_entity(salvo).is_err(),
        "salvo beyond left boundary should be despawned"
    );
}

#[test]
fn salvo_beyond_right_boundary_is_despawned() {
    let mut app = build_salvo_wall_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    // Default playfield: right = 400.0.
    let salvo = spawn_salvo(
        &mut app,
        Vec2::new(405.0, 0.0),
        Vec2::new(50.0, -300.0),
        5.0,
        turret,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    assert!(
        app.world().get_entity(salvo).is_err(),
        "salvo beyond right boundary should be despawned"
    );
}

#[test]
fn salvo_beyond_top_boundary_is_despawned() {
    let mut app = build_salvo_wall_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    // Default playfield: top = 300.0.
    let salvo = spawn_salvo(
        &mut app,
        Vec2::new(0.0, 305.0),
        Vec2::new(0.0, 300.0),
        5.0,
        turret,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    assert!(
        app.world().get_entity(salvo).is_err(),
        "salvo beyond top boundary should be despawned"
    );
}

// ── Behavior 41: Multiple salvos hitting walls: all despawned ──

#[test]
fn multiple_salvos_beyond_boundaries_all_despawned() {
    let mut app = build_salvo_wall_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    let salvo1 = spawn_salvo(
        &mut app,
        Vec2::new(-405.0, 0.0), // beyond left
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );
    let salvo2 = spawn_salvo(
        &mut app,
        Vec2::new(-410.0, 0.0), // beyond left
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );
    let salvo3 = spawn_salvo(
        &mut app,
        Vec2::new(405.0, 0.0), // beyond right
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    assert!(
        app.world().get_entity(salvo1).is_err(),
        "salvo1 beyond left should be despawned"
    );
    assert!(
        app.world().get_entity(salvo2).is_err(),
        "salvo2 beyond left should be despawned"
    );
    assert!(
        app.world().get_entity(salvo3).is_err(),
        "salvo3 beyond right should be despawned"
    );
}

// ── Behavior 42: No salvos: system is a no-op ──

#[test]
fn no_salvos_no_crash() {
    let mut app = build_salvo_wall_collision_app();

    advance_to_playing(&mut app);
    tick(&mut app);

    // No crash is the assertion
}
