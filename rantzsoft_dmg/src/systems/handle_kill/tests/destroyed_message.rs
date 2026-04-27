use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::{
    TestT, assert_f32_eq, drain_destroyed, enqueue_kill, mk_kill, test_app, tick,
};

// ── Behavior 112: writes Destroyed with victim_pos ──

#[test]
fn writes_destroyed_with_victim_pos() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, None));
    tick(&mut app);

    let drained = drain_destroyed(&mut app);
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].victim, victim);
    assert_f32_eq(drained[0].victim_pos.x, 10.0);
    assert_f32_eq(drained[0].victim_pos.y, 20.0);
    assert!(drained[0].killer.is_none());
    assert!(drained[0].killer_pos.is_none());
}

#[test]
fn writes_destroyed_with_zero_victim_pos() {
    // Edge case 112a: Vec2::ZERO is recorded verbatim, not treated as missing.
    let mut app = test_app();
    let victim = app.world_mut().spawn((TestT, Position2D(Vec2::ZERO))).id();

    enqueue_kill(&mut app, mk_kill(victim, None));
    tick(&mut app);

    let drained = drain_destroyed(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].victim_pos.x, 0.0);
    assert_f32_eq(drained[0].victim_pos.y, 0.0);
}

// ── Behavior 113: killer_pos from killer's Position2D ──

#[test]
fn writes_destroyed_with_killer_pos() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();
    let killer = app
        .world_mut()
        .spawn(Position2D(Vec2::new(50.0, 60.0)))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, Some(killer)));
    tick(&mut app);

    let drained = drain_destroyed(&mut app);
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].killer, Some(killer));
    let killer_pos = drained[0].killer_pos.expect("killer_pos should be Some");
    assert_f32_eq(killer_pos.x, 50.0);
    assert_f32_eq(killer_pos.y, 60.0);
}

#[test]
fn writes_destroyed_with_negative_killer_pos() {
    // Edge case 113a.
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();
    let killer = app
        .world_mut()
        .spawn(Position2D(Vec2::new(-100.0, -200.0)))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, Some(killer)));
    tick(&mut app);

    let drained = drain_destroyed(&mut app);
    assert_eq!(drained.len(), 1);
    let kp = drained[0].killer_pos.unwrap();
    assert_f32_eq(kp.x, -100.0);
    assert_f32_eq(kp.y, -200.0);
}

// ── Behavior 114: killer without Position2D → killer_pos: None ──

#[test]
fn killer_without_position_emits_none_killer_pos() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();
    let killer = app.world_mut().spawn_empty().id();

    enqueue_kill(&mut app, mk_kill(victim, Some(killer)));
    tick(&mut app);

    let drained = drain_destroyed(&mut app);
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].killer, Some(killer));
    assert!(drained[0].killer_pos.is_none());
}

// ── Behavior 115: killer: None → killer_pos: None ──

#[test]
fn none_killer_yields_none_killer_pos() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, None));
    tick(&mut app);

    let drained = drain_destroyed(&mut app);
    assert_eq!(drained.len(), 1);
    assert!(drained[0].killer.is_none());
    assert!(drained[0].killer_pos.is_none());
}
