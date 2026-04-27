use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::{TestT, enqueue_kill, mk_kill, test_app, tick};
use crate::components::Dead;

// ── Behavior 111: handle_kill inserts Dead on the victim ──

#[test]
fn inserts_dead_on_victim() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, None));
    tick(&mut app);

    assert!(app.world().get::<Dead>(victim).is_some());
}

#[test]
fn dead_persists_across_idle_tick() {
    // Edge case 111a.
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, None));
    tick(&mut app);
    tick(&mut app);

    assert!(app.world().get::<Dead>(victim).is_some());
}
