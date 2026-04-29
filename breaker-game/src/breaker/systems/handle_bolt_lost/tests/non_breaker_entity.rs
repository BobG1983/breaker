//! Behavior 9: `BoltLost` referring to an entity without a `Breaker`
//! component is silently ignored.

use bevy::prelude::*;

use super::helpers::*;
use crate::{breaker::components::BoltLossBehavior, prelude::*};

#[test]
fn non_breaker_entity_is_silently_ignored() {
    let mut app = handle_bolt_lost_test_app();

    // Spawn an entity with BoltLossBehavior + Hp but NO Breaker marker.
    let entity = app
        .world_mut()
        .spawn((
            BoltLossBehavior::LifeLoss(1),
            Hp {
                current:  3.0,
                starting: 3.0,
                max:      None,
            },
        ))
        .id();

    app.world_mut()
        .resource_mut::<Messages<BoltLost>>()
        .write(BoltLost {
            bolt:    Entity::PLACEHOLDER,
            breaker: entity,
        });

    app.update();

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 3.0).abs() < f32::EPSILON,
        "non-Breaker entity Hp.current should be unchanged at 3.0, got {}",
        hp.current,
    );
    let messages = collect_reduce_messages(&app);
    assert!(
        messages.is_empty(),
        "non-Breaker entity should produce no ReduceNodeTimer; got {} messages",
        messages.len(),
    );
}

#[test]
fn placeholder_breaker_entity_does_not_panic() {
    let mut app = handle_bolt_lost_test_app();

    // No entity exists; target Entity::PLACEHOLDER.
    app.world_mut()
        .resource_mut::<Messages<BoltLost>>()
        .write(BoltLost {
            bolt:    Entity::PLACEHOLDER,
            breaker: Entity::PLACEHOLDER,
        });

    app.update();

    let messages = collect_reduce_messages(&app);
    assert!(
        messages.is_empty(),
        "PLACEHOLDER target should produce no ReduceNodeTimer; got {} messages",
        messages.len(),
    );
}
