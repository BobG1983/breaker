use super::helpers::{TestT, assert_f32_eq, enqueue, mk_msg, test_app, tick};
use crate::components::{Hp, KilledBy};

// ── Behavior 97: killing blow inserts KilledBy { killer: Some(e) } ──

#[test]
fn killing_blow_inserts_killed_by_with_dealer() {
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(10.0))).id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, e, 10.0));
    tick(&mut app);

    let killed_by = app
        .world()
        .get::<KilledBy>(e)
        .expect("KilledBy should be inserted on killing blow");
    assert_eq!(killed_by.killer, Some(dealer));
    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 0.0);
}

#[test]
fn overkill_inserts_killed_by_and_leaves_negative_hp() {
    // Edge case 97a.
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(10.0))).id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, e, 25.0));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(e).unwrap();
    assert_eq!(killed_by.killer, Some(dealer));
    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, -15.0);
}

#[test]
fn environmental_kill_inserts_killed_by_with_none_dealer() {
    // Edge case 97b.
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(10.0))).id();

    enqueue(&mut app, mk_msg(None, None, e, 10.0));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(e).unwrap();
    assert!(killed_by.killer.is_none());
}

// ── Behavior 103: dealer copied verbatim ──

#[test]
fn killing_blow_copies_dealer_verbatim() {
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(1.0))).id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, e, 1.0));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(e).unwrap();
    assert_eq!(killed_by.killer, Some(dealer));
}

#[test]
fn killing_blow_preserves_none_dealer() {
    // Edge case 103a.
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(1.0))).id();

    enqueue(&mut app, mk_msg(None, None, e, 1.0));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(e).unwrap();
    assert!(killed_by.killer.is_none());
}
