use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::{
    TestT, assert_f32_eq, drain_despawn, drain_destroyed, enqueue_kill, mk_kill, test_app, tick,
};
use crate::components::Dead;

// ── Behavior 122: multiple distinct victims in one tick ──

#[test]
fn multiple_victims_in_one_tick() {
    let mut app = test_app();
    let v1 = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(1.0, 2.0))))
        .id();
    let v2 = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(3.0, 4.0))))
        .id();
    let v3 = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(5.0, 6.0))))
        .id();

    enqueue_kill(&mut app, mk_kill(v1, None));
    enqueue_kill(&mut app, mk_kill(v2, None));
    enqueue_kill(&mut app, mk_kill(v3, None));
    tick(&mut app);

    let destroyed = drain_destroyed(&mut app);
    let despawns = drain_despawn(&mut app);
    assert_eq!(destroyed.len(), 3);
    assert_eq!(despawns.len(), 3);
    assert!(app.world().get::<Dead>(v1).is_some());
    assert!(app.world().get::<Dead>(v2).is_some());
    assert!(app.world().get::<Dead>(v3).is_some());

    // Per-victim victim_pos correctness.
    let pos_for = |victim: Entity| -> Vec2 {
        destroyed
            .iter()
            .find(|d| d.victim == victim)
            .map(|d| d.victim_pos)
            .expect("destroyed for victim")
    };
    let p1 = pos_for(v1);
    let p2 = pos_for(v2);
    let p3 = pos_for(v3);
    assert_f32_eq(p1.x, 1.0);
    assert_f32_eq(p1.y, 2.0);
    assert_f32_eq(p2.x, 3.0);
    assert_f32_eq(p2.y, 4.0);
    assert_f32_eq(p3.x, 5.0);
    assert_f32_eq(p3.y, 6.0);
}

#[test]
fn multiple_victims_with_mixed_killers_no_crossleak() {
    // Edge case 122a.
    let mut app = test_app();
    let v1 = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(1.0, 2.0))))
        .id();
    let v2 = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(3.0, 4.0))))
        .id();
    let v3 = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(5.0, 6.0))))
        .id();
    let killer = app
        .world_mut()
        .spawn(Position2D(Vec2::new(99.0, 99.0)))
        .id();

    enqueue_kill(&mut app, mk_kill(v1, None));
    enqueue_kill(&mut app, mk_kill(v2, Some(killer)));
    enqueue_kill(&mut app, mk_kill(v3, None));
    tick(&mut app);

    let destroyed = drain_destroyed(&mut app);
    assert_eq!(destroyed.len(), 3);

    let find = |v: Entity| destroyed.iter().find(|d| d.victim == v).unwrap();
    assert!(find(v1).killer_pos.is_none());
    let kp2 = find(v2).killer_pos.unwrap();
    assert_f32_eq(kp2.x, 99.0);
    assert_f32_eq(kp2.y, 99.0);
    assert!(find(v3).killer_pos.is_none());
}

// ── Behavior 123: self-kill (victim == killer) records both fields ──

#[test]
fn self_kill_records_both_fields() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, Some(victim)));
    tick(&mut app);

    let destroyed = drain_destroyed(&mut app);
    let despawns = drain_despawn(&mut app);
    assert_eq!(destroyed.len(), 1);
    assert_eq!(destroyed[0].victim, victim);
    assert_eq!(destroyed[0].killer, Some(victim));
    let kp = destroyed[0].killer_pos.unwrap();
    assert_f32_eq(kp.x, 10.0);
    assert_f32_eq(kp.y, 20.0);
    assert_f32_eq(destroyed[0].victim_pos.x, 10.0);
    assert_f32_eq(destroyed[0].victim_pos.y, 20.0);
    assert_eq!(despawns.len(), 1);
    assert!(app.world().get::<Dead>(victim).is_some());
}

// ── Behavior 124: Local<HashSet<Entity>> cleared at top of each tick ──

#[test]
fn survives_ten_ticks_with_fresh_victim_each_tick() {
    let mut app = test_app();

    for n in 0..10 {
        let v = app
            .world_mut()
            .spawn((TestT, Position2D(Vec2::new(n as f32, 0.0))))
            .id();
        enqueue_kill(&mut app, mk_kill(v, None));
        tick(&mut app);

        let destroyed = drain_destroyed(&mut app);
        assert_eq!(
            destroyed.len(),
            1,
            "tick {n}: expected exactly one Destroyed"
        );
        assert_eq!(destroyed[0].victim, v);
    }
}

#[test]
fn after_many_ticks_same_frame_dedupe_still_works() {
    // Edge case 124a.
    let mut app = test_app();

    for n in 0..5 {
        let v = app
            .world_mut()
            .spawn((TestT, Position2D(Vec2::new(n as f32, 0.0))))
            .id();
        enqueue_kill(&mut app, mk_kill(v, None));
        tick(&mut app);
        drop(drain_destroyed(&mut app));
        drop(drain_despawn(&mut app));
    }

    // Now a fresh victim with triplicate kills same tick.
    let v = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(99.0, 0.0))))
        .id();
    enqueue_kill(&mut app, mk_kill(v, None));
    enqueue_kill(&mut app, mk_kill(v, None));
    enqueue_kill(&mut app, mk_kill(v, None));
    tick(&mut app);

    let destroyed = drain_destroyed(&mut app);
    assert_eq!(destroyed.len(), 1);
}
