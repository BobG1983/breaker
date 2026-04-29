//! Behavior 6: `BoltLossBehavior::None` is a complete no-op.

use super::helpers::*;
use crate::{breaker::components::BoltLossBehavior, prelude::*};

#[test]
fn none_with_hp_does_not_decrement() {
    let mut app = handle_bolt_lost_test_app();
    let breaker = spawn_test_breaker(
        &mut app,
        BoltLossBehavior::None,
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
        (hp.current - 3.0).abs() < f32::EPSILON,
        "Hp.current should be unchanged at 3.0 for None, got {}",
        hp.current,
    );
    let messages = collect_reduce_messages(&app);
    assert!(
        messages.is_empty(),
        "None must NOT emit ReduceNodeTimer; got {} messages",
        messages.len(),
    );
}

#[test]
fn none_without_hp_does_not_panic_no_messages() {
    let mut app = handle_bolt_lost_test_app();
    let breaker = spawn_test_breaker(&mut app, BoltLossBehavior::None, None);
    send_bolt_lost(&mut app, breaker);
    app.update();

    let messages = collect_reduce_messages(&app);
    assert!(
        messages.is_empty(),
        "None without Hp must NOT emit ReduceNodeTimer; got {} messages",
        messages.len(),
    );
}
