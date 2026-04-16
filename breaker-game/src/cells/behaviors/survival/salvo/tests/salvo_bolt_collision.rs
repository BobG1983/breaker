//! Tests for `salvo_bolt_collision` — behaviors 29-33.

use bevy::prelude::*;

use super::helpers::*;
use crate::prelude::*;

// ── Behavior 29: Salvo overlapping a bolt is despawned ──

#[test]
fn salvo_overlapping_bolt_is_despawned() {
    let mut app = build_salvo_bolt_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    let salvo = spawn_salvo(
        &mut app,
        Vec2::new(100.0, 50.0),
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );
    let _bolt = spawn_collision_bolt(&mut app, Vec2::new(100.0, 50.0), Vec2::new(6.0, 6.0));

    advance_to_playing(&mut app);
    tick(&mut app);

    assert!(
        app.world().get_entity(salvo).is_err(),
        "salvo should be despawned after overlapping a bolt"
    );
}

// ── Behavior 30: Bolt is unaffected by salvo absorption ──

#[test]
fn bolt_unaffected_by_salvo_absorption() {
    let mut app = build_salvo_bolt_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    let _salvo = spawn_salvo(
        &mut app,
        Vec2::new(100.0, 50.0),
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );
    let bolt = spawn_collision_bolt(&mut app, Vec2::new(100.0, 50.0), Vec2::new(6.0, 6.0));

    advance_to_playing(&mut app);
    tick(&mut app);

    // Bolt should still exist with unchanged components
    let hp = app
        .world()
        .get::<Hp>(bolt)
        .expect("bolt should still have Hp");
    assert!(
        (hp.current - 1.0).abs() < f32::EPSILON,
        "bolt Hp should be unchanged at 1.0, got {}",
        hp.current
    );

    let vel = app
        .world()
        .get::<Velocity2D>(bolt)
        .expect("bolt should still have Velocity2D");
    assert!(
        (vel.0.x - 100.0).abs() < 1.0 && (vel.0.y - 200.0).abs() < 1.0,
        "bolt velocity should be approximately unchanged"
    );

    let pos = app
        .world()
        .get::<Position2D>(bolt)
        .expect("bolt should still have Position2D");
    assert!(
        (pos.0.x - 100.0).abs() < 1.0 && (pos.0.y - 50.0).abs() < 1.0,
        "bolt position should be approximately unchanged"
    );
}

// ── Behavior 31: Multiple salvos overlapping same bolt: all absorbed ──

#[test]
fn multiple_salvos_overlapping_bolt_all_despawned() {
    let mut app = build_salvo_bolt_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    let salvo1 = spawn_salvo(
        &mut app,
        Vec2::new(100.0, 50.0),
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );
    let salvo2 = spawn_salvo(
        &mut app,
        Vec2::new(101.0, 50.0),
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );
    let salvo3 = spawn_salvo(
        &mut app,
        Vec2::new(99.0, 50.0),
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );
    let _bolt = spawn_collision_bolt(&mut app, Vec2::new(100.0, 50.0), Vec2::new(6.0, 6.0));

    advance_to_playing(&mut app);
    tick(&mut app);

    assert!(
        app.world().get_entity(salvo1).is_err(),
        "salvo1 should be despawned"
    );
    assert!(
        app.world().get_entity(salvo2).is_err(),
        "salvo2 should be despawned"
    );
    assert!(
        app.world().get_entity(salvo3).is_err(),
        "salvo3 should be despawned"
    );
}

// ── Behavior 32: Salvo not overlapping bolt is not absorbed ──

#[test]
fn salvo_not_overlapping_bolt_survives() {
    let mut app = build_salvo_bolt_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    let salvo = spawn_salvo(
        &mut app,
        Vec2::new(100.0, 300.0), // far from bolt
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );
    let _bolt = spawn_collision_bolt(&mut app, Vec2::new(100.0, 50.0), Vec2::new(6.0, 6.0));

    advance_to_playing(&mut app);
    tick(&mut app);

    assert!(
        app.world().get_entity(salvo).is_ok(),
        "salvo far from bolt should not be despawned"
    );
}

// ── Behavior 33: No salvos or bolts: system is a no-op ──

#[test]
fn no_salvos_no_bolts_no_crash() {
    let mut app = build_salvo_bolt_collision_app();

    advance_to_playing(&mut app);
    tick(&mut app);

    // No crash is the assertion
}

#[test]
fn salvos_exist_but_no_bolts_no_despawn() {
    let mut app = build_salvo_bolt_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    let salvo = spawn_salvo(
        &mut app,
        Vec2::new(100.0, 50.0),
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    assert!(
        app.world().get_entity(salvo).is_ok(),
        "salvo should survive when no bolts exist"
    );
}
