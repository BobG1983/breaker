//! Behavior 10: Multiple breakers with different behaviors process only their
//! own messages.

use bevy::prelude::*;

use super::helpers::*;
use crate::{breaker::components::BoltLossBehavior, prelude::*};

#[test]
fn each_breaker_processes_only_its_own_message() {
    let mut app = handle_bolt_lost_test_app();

    let breaker_a = spawn_test_breaker(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        Some(Hp {
            current:  3.0,
            starting: 3.0,
            max:      None,
        }),
    );
    let breaker_b = spawn_test_breaker(&mut app, BoltLossBehavior::TimeLoss(2.0), None);

    // Write one BoltLost per breaker.
    let mut messages = app.world_mut().resource_mut::<Messages<BoltLost>>();
    messages.write(BoltLost {
        bolt:    Entity::PLACEHOLDER,
        breaker: breaker_a,
    });
    messages.write(BoltLost {
        bolt:    Entity::PLACEHOLDER,
        breaker: breaker_b,
    });

    app.update();

    // A's Hp decremented by 1.
    let hp_a = app.world().get::<Hp>(breaker_a).unwrap();
    assert!(
        (hp_a.current - 2.0).abs() < f32::EPSILON,
        "breaker A Hp.current should be 2.0, got {}",
        hp_a.current,
    );

    // Exactly one ReduceNodeTimer with delta 2.0 from breaker B.
    let collected = collect_reduce_messages(&app);
    assert_eq!(
        collected.len(),
        1,
        "expected exactly 1 ReduceNodeTimer, got {}",
        collected.len(),
    );
    assert!(
        (collected[0].delta - 2.0).abs() < f32::EPSILON,
        "ReduceNodeTimer.delta should be 2.0, got {}",
        collected[0].delta,
    );

    // B has no Hp.
    assert!(
        app.world().get::<Hp>(breaker_b).is_none(),
        "breaker B should not have an Hp component",
    );
}
