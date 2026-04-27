use super::helpers::{TestT, assert_f32_eq, enqueue, mk_heal, test_app, tick};
use crate::components::{HealCap, Hp};

// ── Behavior 125: heal increases Hp.current (below ceiling) ──

#[test]
fn heal_increases_hp_below_ceiling() {
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
        ))
        .id();

    enqueue(&mut app, mk_heal(e, 3.0, HealCap::Max));
    tick(&mut app);

    let hp = app.world().get::<Hp>(e).unwrap();
    assert_f32_eq(hp.current, 8.0);
    assert_f32_eq(hp.starting, 10.0);
    assert!(hp.max.is_none());
}

#[test]
fn heal_clamps_exactly_at_ceiling() {
    // Edge case 125a.
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  7.0,
                starting: 10.0,
                max:      None,
            },
        ))
        .id();

    enqueue(&mut app, mk_heal(e, 3.0, HealCap::Starting));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
}

// ── Behavior 126: multiple heals same tick accumulate ──

#[test]
fn two_heals_same_tick_accumulate() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  5.0,
                starting: 30.0,
                max:      None,
            },
        ))
        .id();

    enqueue(&mut app, mk_heal(e, 4.0, HealCap::Max));
    enqueue(&mut app, mk_heal(e, 6.0, HealCap::Max));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 15.0);
}

#[test]
fn three_heals_same_tick_clamp_at_ceiling() {
    // Edge case 126a.
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  27.0,
                starting: 30.0,
                max:      None,
            },
        ))
        .id();

    enqueue(&mut app, mk_heal(e, 2.0, HealCap::Max));
    enqueue(&mut app, mk_heal(e, 3.0, HealCap::Max));
    enqueue(&mut app, mk_heal(e, 4.0, HealCap::Max));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 30.0);
}
