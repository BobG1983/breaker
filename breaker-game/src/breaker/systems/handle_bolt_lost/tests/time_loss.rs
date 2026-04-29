//! Behaviors 4, 5: `TimeLoss(delta)` writes one `ReduceNodeTimer { delta }`
//! and does NOT touch `Hp` even when present.

use super::helpers::*;
use crate::{breaker::components::BoltLossBehavior, prelude::*};

// ── Behavior 4: TimeLoss(delta) writes exactly one ReduceNodeTimer ──

#[test]
fn time_loss_five_writes_one_reduce_node_timer_with_delta_five() {
    let mut app = handle_bolt_lost_test_app();
    let breaker = spawn_test_breaker(&mut app, BoltLossBehavior::TimeLoss(5.0), None);
    send_bolt_lost(&mut app, breaker);
    app.update();

    let messages = collect_reduce_messages(&app);
    assert_eq!(
        messages.len(),
        1,
        "TimeLoss(5.0) must emit exactly 1 ReduceNodeTimer message, got {}",
        messages.len(),
    );
    assert!(
        (messages[0].delta - 5.0).abs() < f32::EPSILON,
        "ReduceNodeTimer.delta should be 5.0, got {}",
        messages[0].delta,
    );
}

#[test]
fn time_loss_zero_point_five_writes_message_with_delta_zero_point_five() {
    let mut app = handle_bolt_lost_test_app();
    let breaker = spawn_test_breaker(&mut app, BoltLossBehavior::TimeLoss(0.5), None);
    send_bolt_lost(&mut app, breaker);
    app.update();

    let messages = collect_reduce_messages(&app);
    assert_eq!(messages.len(), 1, "expected exactly 1 message");
    assert!(
        (messages[0].delta - 0.5).abs() < f32::EPSILON,
        "delta should be 0.5, got {}",
        messages[0].delta,
    );
}

// ── Behavior 5: TimeLoss(delta) does NOT touch Hp even when present ──

#[test]
fn time_loss_does_not_touch_hp_when_present() {
    let mut app = handle_bolt_lost_test_app();
    let breaker = spawn_test_breaker(
        &mut app,
        BoltLossBehavior::TimeLoss(5.0),
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
        (hp.current - 3.0).abs() < f32::EPSILON,
        "Hp.current must be unchanged at 3.0; TimeLoss must NOT touch Hp; got {}",
        hp.current,
    );
    let messages = collect_reduce_messages(&app);
    assert_eq!(messages.len(), 1, "expected exactly 1 ReduceNodeTimer");
    assert!(
        (messages[0].delta - 5.0).abs() < f32::EPSILON,
        "ReduceNodeTimer.delta should be 5.0, got {}",
        messages[0].delta,
    );
}
