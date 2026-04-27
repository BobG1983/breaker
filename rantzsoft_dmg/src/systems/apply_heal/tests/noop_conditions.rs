use super::helpers::{OtherT, TestT, assert_f32_eq, enqueue, mk_heal, test_app, tick};
use crate::components::{Dead, HealCap, Hp, Invulnerable};

// ── Behavior 130: Dead marker is no-op ──

#[test]
fn dead_entity_is_noop() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  10.0,
                starting: 10.0,
                max:      None,
            },
            Dead,
        ))
        .id();

    enqueue(&mut app, mk_heal(e, 7.0, HealCap::Max));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
}

#[test]
fn dead_entity_with_negative_hp_and_large_heal_is_noop() {
    // Edge case 130a.
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  -5.0,
                starting: 10.0,
                max:      None,
            },
            Dead,
        ))
        .id();

    enqueue(&mut app, mk_heal(e, 100.0, HealCap::Max));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, -5.0);
}

// ── Behavior 131: Invulnerable marker is no-op ──

#[test]
fn invulnerable_entity_is_noop() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            Invulnerable,
        ))
        .id();

    enqueue(&mut app, mk_heal(e, 3.0, HealCap::Max));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 5.0);
}

#[test]
fn invulnerable_with_starting_cap_is_noop() {
    // Edge case 131a.
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            Invulnerable,
        ))
        .id();

    enqueue(&mut app, mk_heal(e, 3.0, HealCap::Starting));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 5.0);
}

// ── Behavior 132: over-cap short-circuit (heal never lowers HP) ──

#[test]
fn over_cap_heal_does_not_lower_hp() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  15.0,
                starting: 10.0,
                max:      None,
            },
        ))
        .id();

    enqueue(&mut app, mk_heal(e, 3.0, HealCap::Max));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 15.0);
}

#[test]
fn exact_cap_heal_does_not_change_hp() {
    // Edge case 132a.
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  10.0,
                starting: 10.0,
                max:      None,
            },
        ))
        .id();

    enqueue(&mut app, mk_heal(e, 3.0, HealCap::Max));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
}

// ── Behavior 134: entity without T marker is excluded ──

#[test]
fn other_t_marker_is_excluded() {
    let mut app = test_app();
    let e = app.world_mut().spawn((OtherT, Hp::new(5.0))).id();

    enqueue(&mut app, mk_heal(e, 3.0, HealCap::Max));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 5.0);
}
