use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::system::on_time_expires;
use crate::{
    effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        storage::BoundEffects,
        triggers::time::{
            components::EffectTimers, messages::EffectTimerExpired, tick_timers::tick_effect_timers,
        },
        types::{BumpTarget, EffectType, ParticipantTarget, Terminal, Tree, Trigger},
    },
    shared::test_utils::TestAppBuilder,
};

// -- Helpers ----------------------------------------------------------

/// Resource to inject `EffectTimerExpired` messages into the test app.
#[derive(Resource, Default)]
struct TestTimerExpiredMessages(Vec<EffectTimerExpired>);

/// System that writes `EffectTimerExpired` messages from the test resource.
fn inject_timer_expired(
    messages: Res<TestTimerExpiredMessages>,
    mut writer: MessageWriter<EffectTimerExpired>,
) {
    for msg in &messages.0 {
        writer.write(msg.clone());
    }
}

fn bridge_test_app() -> App {
    TestAppBuilder::new()
        .with_message::<EffectTimerExpired>()
        .with_resource::<TestTimerExpiredMessages>()
        .with_system(
            FixedUpdate,
            (
                inject_timer_expired.before(on_time_expires),
                on_time_expires,
            ),
        )
        .build()
}

fn tick(app: &mut App) {
    crate::shared::test_utils::tick(app);
}

/// Helper to build a When(TimeExpires(duration), Fire(SpeedBoost)) tree.
fn time_expires_speed_tree(name: &str, duration: f32, multiplier: f32) -> (String, Tree) {
    (
        name.to_string(),
        Tree::When(
            Trigger::TimeExpires(OrderedFloat(duration)),
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(multiplier),
            }))),
        ),
    )
}

/// Helper to build a When(TimeExpires(duration), On(Bump(target), Fire(SpeedBoost))) tree.
fn time_expires_on_bump_tree(
    name: &str,
    duration: f32,
    target: BumpTarget,
    multiplier: f32,
) -> (String, Tree) {
    (
        name.to_string(),
        Tree::When(
            Trigger::TimeExpires(OrderedFloat(duration)),
            Box::new(Tree::On(
                ParticipantTarget::Bump(target),
                Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(multiplier),
                })),
            )),
        ),
    )
}

// -- Behavior 7: bridge dispatches TimeExpires trigger -----------------

#[test]
fn on_time_expires_dispatches_trigger_on_entity_with_bound_effects() {
    let mut app = bridge_test_app();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![time_expires_speed_tree(
            "chip_a", 5.0, 1.5,
        )]))
        .id();

    app.insert_resource(TestTimerExpiredMessages(vec![EffectTimerExpired {
        entity,
        original_duration: OrderedFloat(5.0),
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist on entity after TimeExpires trigger");
    assert_eq!(stack.len(), 1);
}

#[test]
fn on_time_expires_no_effect_when_tree_does_not_match_duration() {
    let mut app = bridge_test_app();

    // Tree expects TimeExpires(5.0) but message has duration 3.0
    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![time_expires_speed_tree(
            "chip_a", 5.0, 1.5,
        )]))
        .id();

    app.insert_resource(TestTimerExpiredMessages(vec![EffectTimerExpired {
        entity,
        original_duration: OrderedFloat(3.0),
    }]));

    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "no effect should fire when TimeExpires duration does not match"
    );
}

// -- Behavior 8: bridge does not walk entity without BoundEffects -----

#[test]
fn on_time_expires_skips_entity_without_bound_effects() {
    let mut app = bridge_test_app();

    let entity_a = app
        .world_mut()
        .spawn(BoundEffects(vec![time_expires_speed_tree(
            "chip_a", 3.0, 1.5,
        )]))
        .id();

    let entity_b = app.world_mut().spawn_empty().id();

    app.insert_resource(TestTimerExpiredMessages(vec![
        EffectTimerExpired {
            entity:            entity_a,
            original_duration: OrderedFloat(3.0),
        },
        EffectTimerExpired {
            entity:            entity_b,
            original_duration: OrderedFloat(3.0),
        },
    ]));

    tick(&mut app);

    let stack_a = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity_a)
        .expect("entity_a should have EffectStack");
    assert_eq!(stack_a.len(), 1);

    let stack_b = app.world().get::<EffectStack<SpeedBoostConfig>>(entity_b);
    assert!(
        stack_b.is_none(),
        "entity_b without BoundEffects should not have EffectStack"
    );
}

#[test]
fn on_time_expires_silently_skips_despawned_entity() {
    let mut app = bridge_test_app();

    let entity = app.world_mut().spawn_empty().id();
    app.world_mut().despawn(entity);

    app.insert_resource(TestTimerExpiredMessages(vec![EffectTimerExpired {
        entity,
        original_duration: OrderedFloat(3.0),
    }]));

    // Should not panic
    tick(&mut app);
}

// -- Behavior 9: bridge uses TriggerContext::None ---------------------

#[test]
fn on_time_expires_uses_trigger_context_none_so_on_bump_cannot_resolve() {
    let mut app = bridge_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![time_expires_on_bump_tree(
            "chip_a",
            2.0,
            BumpTarget::Bolt,
            1.5,
        )]))
        .id();

    app.insert_resource(TestTimerExpiredMessages(vec![EffectTimerExpired {
        entity,
        original_duration: OrderedFloat(2.0),
    }]));

    tick(&mut app);

    // On(Bump(Bolt)) cannot resolve with TriggerContext::None
    let bolt_stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_entity);
    assert!(
        bolt_stack.is_none(),
        "On(Bump(Bolt)) should not resolve with TriggerContext::None"
    );

    let owner_stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        owner_stack.is_none(),
        "effect should not fire on owner either when On node cannot resolve"
    );
}

// -- Behavior 10: multiple messages in one frame ----------------------

#[test]
fn on_time_expires_handles_multiple_messages_in_one_frame() {
    let mut app = bridge_test_app();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![time_expires_speed_tree(
            "chip_a", 3.0, 1.5,
        )]))
        .id();

    app.insert_resource(TestTimerExpiredMessages(vec![
        EffectTimerExpired {
            entity,
            original_duration: OrderedFloat(3.0),
        },
        EffectTimerExpired {
            entity,
            original_duration: OrderedFloat(3.0),
        },
    ]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist after two messages");
    assert_eq!(
        stack.len(),
        2,
        "each message should trigger a separate walk"
    );
}

#[test]
fn on_time_expires_different_durations_fire_matching_trees_only() {
    let mut app = bridge_test_app();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![
            time_expires_speed_tree("chip_a", 3.0, 1.5),
            time_expires_speed_tree("chip_b", 5.0, 2.0),
        ]))
        .id();

    app.insert_resource(TestTimerExpiredMessages(vec![
        EffectTimerExpired {
            entity,
            original_duration: OrderedFloat(3.0),
        },
        EffectTimerExpired {
            entity,
            original_duration: OrderedFloat(5.0),
        },
    ]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist");
    assert_eq!(
        stack.len(),
        2,
        "each matching duration should fire its own tree"
    );
}

// -- Behavior 11: no-op when no messages exist ------------------------

#[test]
fn on_time_expires_is_noop_without_messages() {
    let mut app = bridge_test_app();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![time_expires_speed_tree(
            "chip_a", 5.0, 1.5,
        )]))
        .id();

    // No messages injected
    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "no EffectStack should exist when no messages are sent"
    );
}

// -- Behavior 12: end-to-end tick_timers + on_time_expires across two ticks

#[test]
fn end_to_end_timer_expires_and_bridge_fires_across_two_ticks() {
    // Production ordering: Bridge before Tick
    let mut app = TestAppBuilder::new()
        .with_message::<EffectTimerExpired>()
        .with_system(
            FixedUpdate,
            (
                on_time_expires.before(tick_effect_timers),
                tick_effect_timers,
            ),
        )
        .build();

    let entity = app
        .world_mut()
        .spawn((
            EffectTimers {
                timers: vec![(OrderedFloat(0.001), OrderedFloat(4.0))],
            },
            BoundEffects(vec![time_expires_speed_tree("chip_a", 4.0, 2.0)]),
        ))
        .id();

    // Tick 1: tick_effect_timers expires the timer, enqueues EffectTimerExpired.
    // Bridge ran first (before tick) so it reads nothing this frame.
    tick(&mut app);

    assert!(
        app.world().get::<EffectTimers>(entity).is_none(),
        "EffectTimers should be removed after tick 1"
    );
    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .is_none(),
        "effect should NOT fire yet after tick 1 (bridge ran before tick)"
    );

    // Tick 2: bridge reads the expired message and fires the effect.
    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist after tick 2");
    assert_eq!(
        stack.len(),
        1,
        "one effect entry should be added from the expired timer"
    );
}

#[test]
fn end_to_end_timer_with_long_remaining_does_not_fire() {
    // Production ordering: Bridge before Tick
    let mut app = TestAppBuilder::new()
        .with_message::<EffectTimerExpired>()
        .with_system(
            FixedUpdate,
            (
                on_time_expires.before(tick_effect_timers),
                tick_effect_timers,
            ),
        )
        .build();

    let entity = app
        .world_mut()
        .spawn((
            EffectTimers {
                timers: vec![(OrderedFloat(10.0), OrderedFloat(10.0))],
            },
            BoundEffects(vec![time_expires_speed_tree("chip_a", 10.0, 2.0)]),
        ))
        .id();

    tick(&mut app);
    tick(&mut app);

    assert!(
        app.world().get::<EffectTimers>(entity).is_some(),
        "EffectTimers should still be present with 10s remaining"
    );
    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .is_none(),
        "no effect should fire — timer has not expired"
    );
}

// -- Behavior 13: non-matching TimeExpires duration does not trigger ---

#[test]
fn non_matching_time_expires_duration_does_not_trigger() {
    let mut app = bridge_test_app();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![time_expires_speed_tree(
            "chip_a", 5.0, 1.5,
        )]))
        .id();

    // Send duration 3.0, but tree expects 5.0
    app.insert_resource(TestTimerExpiredMessages(vec![EffectTimerExpired {
        entity,
        original_duration: OrderedFloat(3.0),
    }]));

    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "tree gated on TimeExpires(5.0) should not fire for TimeExpires(3.0)"
    );
}
