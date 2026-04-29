//! Behaviors 1, 2: `BoltLossBehavior::LifeLoss(n)` decrements `Hp.current`
//! by `n`, clamped at 0.0.

use super::helpers::*;
use crate::{breaker::components::BoltLossBehavior, prelude::*};

// ── Behavior 1: LifeLoss(1) decrements Hp.current by 1.0 ──

#[test]
fn life_loss_one_decrements_hp_by_one() {
    let mut app = handle_bolt_lost_test_app();
    let breaker = spawn_test_breaker(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        Some(Hp {
            current:  3.0,
            starting: 3.0,
            max:      None,
        }),
    );
    send_bolt_lost(&mut app, breaker);
    app.update();

    let hp = app
        .world()
        .get::<Hp>(breaker)
        .expect("breaker should still have Hp");
    assert!(
        (hp.current - 2.0).abs() < f32::EPSILON,
        "Hp.current should be 2.0 after LifeLoss(1) on 3.0, got {}",
        hp.current,
    );
    assert!(
        (hp.starting - 3.0).abs() < f32::EPSILON,
        "Hp.starting should remain 3.0, got {}",
        hp.starting,
    );
    let messages = collect_reduce_messages(&app);
    assert!(
        messages.is_empty(),
        "LifeLoss must not emit ReduceNodeTimer; got {} messages",
        messages.len(),
    );
}

#[test]
fn life_loss_two_decrements_hp_by_two() {
    let mut app = handle_bolt_lost_test_app();
    let breaker = spawn_test_breaker(
        &mut app,
        BoltLossBehavior::LifeLoss(2),
        Some(Hp {
            current:  3.0,
            starting: 3.0,
            max:      None,
        }),
    );
    send_bolt_lost(&mut app, breaker);
    app.update();

    let hp = app.world().get::<Hp>(breaker).unwrap();
    assert!(
        (hp.current - 1.0).abs() < f32::EPSILON,
        "Hp.current should be 1.0 after LifeLoss(2) on 3.0, got {}",
        hp.current,
    );
}

// ── Behavior 2: LifeLoss(n) clamps Hp.current at 0.0 ──

#[test]
fn life_loss_clamps_hp_at_zero_no_underflow() {
    let mut app = handle_bolt_lost_test_app();
    let breaker = spawn_test_breaker(
        &mut app,
        BoltLossBehavior::LifeLoss(5),
        Some(Hp {
            current:  1.0,
            starting: 3.0,
            max:      None,
        }),
    );
    send_bolt_lost(&mut app, breaker);
    app.update();

    let hp = app.world().get::<Hp>(breaker).unwrap();
    assert!(
        (hp.current - 0.0).abs() < f32::EPSILON,
        "Hp.current should be clamped to 0.0 after LifeLoss(5) on 1.0, got {}",
        hp.current,
    );
}

#[test]
fn life_loss_on_zero_hp_stays_at_zero() {
    let mut app = handle_bolt_lost_test_app();
    let breaker = spawn_test_breaker(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        Some(Hp {
            current:  0.0,
            starting: 3.0,
            max:      None,
        }),
    );
    send_bolt_lost(&mut app, breaker);
    app.update();

    let hp = app.world().get::<Hp>(breaker).unwrap();
    assert!(
        (hp.current - 0.0).abs() < f32::EPSILON,
        "Hp.current should stay 0.0 (no underflow), got {}",
        hp.current,
    );
}
