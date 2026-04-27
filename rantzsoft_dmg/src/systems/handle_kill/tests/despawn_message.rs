use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::{TestT, drain_despawn, enqueue_kill, mk_kill, test_app, tick};

// ── Behavior 116: one DespawnEntity per unique victim ──

#[test]
fn writes_one_despawn_per_victim() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, None));
    tick(&mut app);

    let drained = drain_despawn(&mut app);
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].entity, victim);
}

#[test]
fn writes_one_despawn_per_each_of_three_victims() {
    // Edge case 116a.
    let mut app = test_app();
    let v1 = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(1.0, 1.0))))
        .id();
    let v2 = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(2.0, 2.0))))
        .id();
    let v3 = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(3.0, 3.0))))
        .id();

    enqueue_kill(&mut app, mk_kill(v1, None));
    enqueue_kill(&mut app, mk_kill(v2, None));
    enqueue_kill(&mut app, mk_kill(v3, None));
    tick(&mut app);

    let drained = drain_despawn(&mut app);
    assert_eq!(drained.len(), 3);
    let entities: Vec<Entity> = drained.iter().map(|d| d.entity).collect();
    assert!(entities.contains(&v1));
    assert!(entities.contains(&v2));
    assert!(entities.contains(&v3));
}
