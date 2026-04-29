//! Behavior 3: `LifeLoss(n)` is a no-op when `Hp` is absent (Godmode pattern).

use super::helpers::*;
use crate::breaker::components::BoltLossBehavior;

#[test]
fn life_loss_without_hp_does_not_panic() {
    let mut app = handle_bolt_lost_test_app();
    let breaker = spawn_test_breaker(&mut app, BoltLossBehavior::LifeLoss(1), None);
    send_bolt_lost(&mut app, breaker);
    app.update();

    let messages = collect_reduce_messages(&app);
    assert!(
        messages.is_empty(),
        "LifeLoss without Hp must NOT emit ReduceNodeTimer; got {} messages",
        messages.len(),
    );
}

#[test]
fn life_loss_99_without_hp_still_no_panic_no_messages() {
    let mut app = handle_bolt_lost_test_app();
    let breaker = spawn_test_breaker(&mut app, BoltLossBehavior::LifeLoss(99), None);
    send_bolt_lost(&mut app, breaker);
    app.update();

    let messages = collect_reduce_messages(&app);
    assert!(
        messages.is_empty(),
        "LifeLoss(99) without Hp must NOT emit ReduceNodeTimer; got {} messages",
        messages.len(),
    );
}
