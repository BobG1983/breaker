use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::helpers::*;
use crate::{
    breaker::messages::NoBump,
    effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        storage::BoundEffects,
        types::{BumpTarget, EffectType, Tree, Trigger},
    },
};

// -- Behavior 11: walks effects with NoBumpOccurred on all entities ----

#[test]
fn on_no_bump_occurred_walks_all_entities_with_bound_effects() {
    let mut app = bridge_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let entity_a = app
        .world_mut()
        .spawn(BoundEffects(vec![no_bump_speed_tree("chip_a", 1.5)]))
        .id();

    let entity_b = app
        .world_mut()
        .spawn(BoundEffects(vec![no_bump_speed_tree("chip_b", 2.0)]))
        .id();

    app.insert_resource(TestNoBumpMessages(vec![NoBump {
        bolt:    bolt_entity,
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack_a = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity_a)
        .expect("EffectStack should exist on entity_a");
    assert_eq!(stack_a.len(), 1, "entity_a should have one effect fired");

    let stack_b = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity_b)
        .expect("EffectStack should exist on entity_b");
    assert_eq!(stack_b.len(), 1, "entity_b should have one effect fired");
}

// -- Behavior 12: does not fire on non-matching trigger gates ----------

#[test]
fn on_no_bump_occurred_does_not_fire_on_non_matching_trigger() {
    let mut app = bridge_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    // BumpOccurred gate, NOT NoBumpOccurred
    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::When(
                Trigger::BumpOccurred,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            ),
        )]))
        .id();

    app.insert_resource(TestNoBumpMessages(vec![NoBump {
        bolt:    bolt_entity,
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "BumpOccurred gate should not match NoBumpOccurred trigger"
    );
}

// -- Behavior 13: provides Bump TriggerContext, On(Bump(Bolt)) resolves -

#[test]
fn on_no_bump_occurred_provides_bump_context_with_bolt() {
    let mut app = bridge_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![no_bump_on_target_tree(
            "chip_a",
            BumpTarget::Bolt,
            1.5,
        )]))
        .id();

    app.insert_resource(TestNoBumpMessages(vec![NoBump {
        bolt:    bolt_entity,
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let bolt_stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
        .expect("EffectStack should exist on bolt_entity via On(Bump(Bolt))");
    assert_eq!(bolt_stack.len(), 1);

    let owner_stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        owner_stack.is_none(),
        "On node should redirect to bolt, not fire on owner"
    );
}

// -- Behavior 14: On(Bump(Breaker)) resolves from context ----

#[test]
fn on_no_bump_occurred_resolves_bump_breaker_participant() {
    let mut app = bridge_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![no_bump_on_target_tree(
            "chip_a",
            BumpTarget::Breaker,
            1.5,
        )]))
        .id();

    app.insert_resource(TestNoBumpMessages(vec![NoBump {
        bolt:    bolt_entity,
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let breaker_stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
        .expect("EffectStack should exist on breaker_entity via On(Bump(Breaker))");
    assert_eq!(breaker_stack.len(), 1);

    let owner_stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        owner_stack.is_none(),
        "On node should redirect to breaker, not fire on owner"
    );
}

// -- Behavior 15: no-op when no NoBump messages exist ------------------

#[test]
fn on_no_bump_occurred_is_no_op_without_messages() {
    let mut app = bridge_test_app();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![no_bump_speed_tree("chip_a", 1.5)]))
        .id();

    // No messages injected (default empty resource)
    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "no EffectStack should exist when no NoBump messages are sent"
    );
}

// -- Behavior 16: handles multiple NoBump messages in one frame --------

#[test]
fn on_no_bump_occurred_handles_multiple_messages_per_frame() {
    let mut app = bridge_test_app();

    let bolt_a = app.world_mut().spawn_empty().id();
    let breaker_a = app.world_mut().spawn_empty().id();
    let bolt_b = app.world_mut().spawn_empty().id();
    let breaker_b = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![no_bump_speed_tree("chip_a", 1.5)]))
        .id();

    app.insert_resource(TestNoBumpMessages(vec![
        NoBump {
            bolt:    bolt_a,
            breaker: breaker_a,
        },
        NoBump {
            bolt:    bolt_b,
            breaker: breaker_b,
        },
    ]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist after two NoBump messages");
    assert_eq!(
        stack.len(),
        2,
        "each NoBump message should trigger a separate walk"
    );
}

// -- Behavior 17: does not walk entities without BoundEffects ----------

#[test]
fn on_no_bump_occurred_does_not_walk_entities_without_bound_effects() {
    let mut app = bridge_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let entity_a = app
        .world_mut()
        .spawn(BoundEffects(vec![no_bump_speed_tree("chip_a", 1.5)]))
        .id();

    // entity_b has no BoundEffects
    let entity_b = app.world_mut().spawn_empty().id();

    app.insert_resource(TestNoBumpMessages(vec![NoBump {
        bolt:    bolt_entity,
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack_a = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity_a)
        .expect("EffectStack should exist on entity_a (has BoundEffects)");
    assert_eq!(stack_a.len(), 1);

    let stack_b = app.world().get::<EffectStack<SpeedBoostConfig>>(entity_b);
    assert!(
        stack_b.is_none(),
        "entity_b (no BoundEffects) should not have an EffectStack"
    );
}
