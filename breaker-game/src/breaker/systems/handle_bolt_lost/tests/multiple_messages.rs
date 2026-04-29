//! Behavior 8: Multiple `BoltLost` messages for the same breaker process all
//! of them.

use bevy::prelude::*;

use super::helpers::*;
use crate::{breaker::components::BoltLossBehavior, prelude::*};

#[test]
fn three_bolt_lost_messages_decrement_hp_three_times() {
    let mut app = handle_bolt_lost_test_app();
    let breaker = spawn_test_breaker(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        Some(Hp {
            current:  5.0,
            starting: 5.0,
            max:      None,
        }),
    );

    // Write three BoltLost messages targeting the same breaker.
    let mut messages = app.world_mut().resource_mut::<Messages<BoltLost>>();
    messages.write(BoltLost {
        bolt: Entity::PLACEHOLDER,
        breaker,
    });
    messages.write(BoltLost {
        bolt: Entity::PLACEHOLDER,
        breaker,
    });
    messages.write(BoltLost {
        bolt: Entity::PLACEHOLDER,
        breaker,
    });

    app.update();

    let hp = app.world().get::<Hp>(breaker).unwrap();
    assert!(
        (hp.current - 2.0).abs() < f32::EPSILON,
        "Hp.current should be 2.0 after 3x LifeLoss(1) on 5.0, got {}",
        hp.current,
    );
}

#[test]
fn three_bolt_lost_messages_emit_three_reduce_node_timer_messages_for_time_loss() {
    let mut app = handle_bolt_lost_test_app();
    let breaker = spawn_test_breaker(&mut app, BoltLossBehavior::TimeLoss(2.0), None);

    let mut messages = app.world_mut().resource_mut::<Messages<BoltLost>>();
    messages.write(BoltLost {
        bolt: Entity::PLACEHOLDER,
        breaker,
    });
    messages.write(BoltLost {
        bolt: Entity::PLACEHOLDER,
        breaker,
    });
    messages.write(BoltLost {
        bolt: Entity::PLACEHOLDER,
        breaker,
    });

    app.update();

    let collected = collect_reduce_messages(&app);
    assert_eq!(
        collected.len(),
        3,
        "TimeLoss should emit one ReduceNodeTimer per BoltLost; got {}",
        collected.len(),
    );
    for (i, m) in collected.iter().enumerate() {
        assert!(
            (m.delta - 2.0).abs() < f32::EPSILON,
            "ReduceNodeTimer[{i}].delta should be 2.0, got {}",
            m.delta,
        );
    }
}
