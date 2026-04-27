use super::helpers::{TestT, assert_f32_eq, enqueue, mk_msg, test_app, tick};
use crate::components::{Hp, KilledBy};

// ── Behavior 98: non-killing hit does NOT insert KilledBy ──

#[test]
fn non_killing_hit_does_not_insert_killed_by() {
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(30.0))).id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, e, 10.0));
    tick(&mut app);

    assert!(app.world().get::<KilledBy>(e).is_none());
}

#[test]
fn second_non_killing_hit_still_no_killed_by() {
    // Edge case 98a.
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(30.0))).id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, e, 10.0));
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 20.0);

    enqueue(&mut app, mk_msg(Some(dealer), None, e, 5.0));
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 15.0);
    assert!(app.world().get::<KilledBy>(e).is_none());
}

// ── W2 Behavior 10: KilledBy only inserted on the killing blow ──

#[test]
fn non_killing_blow_with_positive_hp_skips_killed_by() {
    // 2.0 damage on 5.0 HP: was_positive && hp.current (3.0) > 0 → no insert.
    let mut app = test_app();
    let victim = app.world_mut().spawn((TestT, Hp::new(5.0))).id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, victim, 2.0));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 3.0);
    assert!(app.world().get::<KilledBy>(victim).is_none());
}

#[test]
fn zero_damage_on_positive_hp_skips_killed_by() {
    // Edge: amount 0.0 → was_positive true but post-HP still positive → no insert.
    let mut app = test_app();
    let victim = app.world_mut().spawn((TestT, Hp::new(5.0))).id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, victim, 0.0));
    tick(&mut app);

    assert!(app.world().get::<KilledBy>(victim).is_none());
}

#[test]
fn pre_dead_hp_does_not_insert_killed_by() {
    // Edge: victim already at Hp 0.0 before damage → was_positive false → no insert.
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  0.0,
                starting: 5.0,
                max:      None,
            },
        ))
        .id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, victim, 1.0));
    tick(&mut app);

    assert!(app.world().get::<KilledBy>(victim).is_none());
}

#[test]
fn exact_kill_inserts_killed_by() {
    // Edge: exact kill (amount == starting) inserts KilledBy (positive regression pin).
    let mut app = test_app();
    let victim = app.world_mut().spawn((TestT, Hp::new(5.0))).id();
    let dealer = app.world_mut().spawn_empty().id();
    let attributed = app.world_mut().spawn_empty().id();

    enqueue(
        &mut app,
        mk_msg(Some(dealer), Some(attributed), victim, 5.0),
    );
    tick(&mut app);

    let killed_by = app
        .world()
        .get::<KilledBy>(victim)
        .expect("KilledBy should be inserted on exact kill");
    assert_eq!(killed_by.killer, Some(attributed));
}
