use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::{
    TestT, drain_despawn, drain_destroyed, enqueue_kill, mk_kill, test_app, tick,
};
use crate::components::Dead;

// ── Behavior 117: same-frame duplicate for same victim dedupes ──

#[test]
fn same_frame_duplicate_dedupes() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, None));
    enqueue_kill(&mut app, mk_kill(victim, None));
    tick(&mut app);

    let destroyed = drain_destroyed(&mut app);
    let despawns = drain_despawn(&mut app);
    assert_eq!(destroyed.len(), 1);
    assert_eq!(despawns.len(), 1);
    assert!(app.world().get::<Dead>(victim).is_some());
}

// ── Behavior 118: cross-frame idempotency via Without<Dead> ──

#[test]
fn cross_frame_dead_victim_is_filtered() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0)), Dead))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, None));
    tick(&mut app);

    let destroyed = drain_destroyed(&mut app);
    let despawns = drain_despawn(&mut app);
    assert!(destroyed.is_empty());
    assert!(despawns.is_empty());
    assert!(app.world().get::<Dead>(victim).is_some());
}
