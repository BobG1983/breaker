use super::helpers::{TestT, assert_f32_eq, enqueue, mk_heal, test_app, tick};
use crate::components::{HealCap, Hp};

// ── Behavior 127: HealCap::Max with max: None → clamps at starting ──

#[test]
fn max_cap_with_no_max_falls_back_to_starting() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  9.5,
                starting: 10.0,
                max:      None,
            },
        ))
        .id();

    enqueue(&mut app, mk_heal(e, 5.0, HealCap::Max));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
}

#[test]
fn max_cap_at_starting_plus_small_heal_stays_at_starting() {
    // Edge case 127a.
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

    enqueue(&mut app, mk_heal(e, 1.0, HealCap::Max));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
}

// ── Behavior 128: HealCap::Starting ignores max ──

#[test]
fn starting_cap_ignores_max() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      Some(20.0),
            },
        ))
        .id();

    enqueue(&mut app, mk_heal(e, 50.0, HealCap::Starting));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
}

// ── Behavior 129: HealCap::Max with max: Some → clamps at max ──

#[test]
fn max_cap_with_some_max_clamps_at_max() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      Some(20.0),
            },
        ))
        .id();

    enqueue(&mut app, mk_heal(e, 50.0, HealCap::Max));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 20.0);
}
