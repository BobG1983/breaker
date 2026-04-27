use super::helpers::{OtherT, TestT, assert_f32_eq, enqueue, mk_msg, test_app, tick};
use crate::components::{Dead, Hp, KilledBy};

// ── Behavior 99: entity with Dead marker is skipped ──

#[test]
fn dead_entity_is_skipped() {
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(10.0), Dead)).id();

    enqueue(&mut app, mk_msg(None, None, e, 5.0));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
    assert!(app.world().get::<KilledBy>(e).is_none());
}

#[test]
fn dead_entity_with_low_hp_is_still_skipped() {
    // Edge case 99a.
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(5.0), Dead)).id();

    enqueue(&mut app, mk_msg(None, None, e, 10.0));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 5.0);
}

// ── Behavior 100: entity without Hp is silently skipped ──

#[test]
fn entity_without_hp_is_skipped() {
    let mut app = test_app();
    let e = app.world_mut().spawn(TestT).id();

    enqueue(&mut app, mk_msg(None, None, e, 5.0));
    tick(&mut app);

    assert!(app.world().get::<Hp>(e).is_none());
    assert!(app.world().get::<KilledBy>(e).is_none());
}

// ── Behavior 101: entity without T marker is skipped ──

#[test]
fn entity_without_t_marker_is_skipped() {
    let mut app = test_app();
    let e = app.world_mut().spawn((OtherT, Hp::new(10.0))).id();

    enqueue(&mut app, mk_msg(None, None, e, 10.0));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
    assert!(app.world().get::<KilledBy>(e).is_none());
}
