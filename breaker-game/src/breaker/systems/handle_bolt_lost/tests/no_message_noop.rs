//! Behavior 7: No `BoltLost` messages → no-op.

use super::helpers::*;
use crate::{breaker::components::BoltLossBehavior, prelude::*};

#[test]
fn no_bolt_lost_messages_no_changes() {
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
    // Intentionally do NOT write any BoltLost message.
    app.update();

    let hp = app.world().get::<Hp>(breaker).unwrap();
    assert!(
        (hp.current - 3.0).abs() < f32::EPSILON,
        "Hp.current should be unchanged at 3.0, got {}",
        hp.current,
    );
    let messages = collect_reduce_messages(&app);
    assert!(
        messages.is_empty(),
        "no BoltLost should produce no ReduceNodeTimer; got {} messages",
        messages.len(),
    );
}
