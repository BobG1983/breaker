use super::helpers::{TestT, assert_f32_eq, enqueue, mk_heal, test_app, tick};
use crate::components::{HealCap, Hp};

// ── Behavior 133: heal amount guards ──

fn run_with_amount(amount: f32) -> f32 {
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
    enqueue(&mut app, mk_heal(e, amount, HealCap::Max));
    tick(&mut app);
    app.world().get::<Hp>(e).unwrap().current
}

#[test]
fn amount_zero_is_noop() {
    assert_f32_eq(run_with_amount(0.0), 7.0);
}

#[test]
fn amount_negative_is_noop() {
    assert_f32_eq(run_with_amount(-3.0), 7.0);
}

#[test]
fn amount_negative_zero_is_noop() {
    assert_f32_eq(run_with_amount(-0.0), 7.0);
}

#[test]
fn amount_nan_is_noop() {
    assert_f32_eq(run_with_amount(f32::NAN), 7.0);
}

#[test]
fn amount_neg_infinity_is_noop() {
    assert_f32_eq(run_with_amount(f32::NEG_INFINITY), 7.0);
}

#[test]
fn amount_infinity_max_cap_no_max_clamps_at_starting() {
    // Edge case 133a.
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
    enqueue(&mut app, mk_heal(e, f32::INFINITY, HealCap::Max));
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
}

#[test]
fn amount_infinity_starting_cap_with_max_clamps_at_starting() {
    // Edge case 133b.
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  7.0,
                starting: 10.0,
                max:      Some(20.0),
            },
        ))
        .id();
    enqueue(&mut app, mk_heal(e, f32::INFINITY, HealCap::Starting));
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
}

#[test]
fn amount_infinity_max_cap_with_max_clamps_at_max() {
    // Edge case 133c.
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  7.0,
                starting: 10.0,
                max:      Some(20.0),
            },
        ))
        .id();
    enqueue(&mut app, mk_heal(e, f32::INFINITY, HealCap::Max));
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 20.0);
}
