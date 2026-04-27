use super::helpers::{TestT, assert_f32_eq, enqueue, mk_msg, test_app, tick};
use crate::components::{Hp, KilledBy};

// ── Behavior 102: first-kill-wins ──

#[test]
fn first_kill_wins_with_two_messages_same_tick() {
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(10.0))).id();
    let dealer_a = app.world_mut().spawn_empty().id();
    let dealer_b = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer_a), None, e, 10.0));
    enqueue(&mut app, mk_msg(Some(dealer_b), None, e, 5.0));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(e).unwrap();
    assert_eq!(killed_by.killer, Some(dealer_a));
}

#[test]
fn first_kill_wins_with_three_messages_same_tick() {
    // Edge case 102a.
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(10.0))).id();
    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();
    let c = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(a), None, e, 10.0));
    enqueue(&mut app, mk_msg(Some(b), None, e, 5.0));
    enqueue(&mut app, mk_msg(Some(c), None, e, 2.0));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(e).unwrap();
    assert_eq!(killed_by.killer, Some(a));
    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, -7.0);
}

#[test]
fn first_kill_wins_when_first_is_non_killing() {
    // Edge case 102b.
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(10.0))).id();
    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();
    let c = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(a), None, e, 3.0));
    enqueue(&mut app, mk_msg(Some(b), None, e, 7.0));
    enqueue(&mut app, mk_msg(Some(c), None, e, 5.0));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(e).unwrap();
    assert_eq!(killed_by.killer, Some(b));
}
