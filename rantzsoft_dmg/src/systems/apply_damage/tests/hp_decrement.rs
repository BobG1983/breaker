use super::helpers::{TestT, assert_f32_eq, enqueue, mk_msg, test_app, tick};
use crate::components::{Dead, Hp, KilledBy};

// ── Behavior 96: Hp decrements by msg.amount ──

#[test]
fn hp_decrements_by_msg_amount() {
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(30.0))).id();

    enqueue(&mut app, mk_msg(None, None, e, 10.0));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 20.0);
    assert!(app.world().get::<Dead>(e).is_none());
    assert!(app.world().get::<KilledBy>(e).is_none());
}

#[test]
fn zero_amount_leaves_hp_unchanged_and_no_killed_by() {
    // Edge case 96a.
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(30.0))).id();

    enqueue(&mut app, mk_msg(None, None, e, 0.0));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 30.0);
    assert!(app.world().get::<KilledBy>(e).is_none());
}
