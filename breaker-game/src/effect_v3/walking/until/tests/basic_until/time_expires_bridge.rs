//! Group A — A4: Source-keyed disambiguation for two same-duration `Until`
//! entries on the same entity. This file uses the `TestAppBuilder` + bridge
//! helper pattern (incompatible with the bare-`World` pattern in
//! `time_expires.rs`).

// Mirrored from triggers/time/bridges/tests.rs (until shared test_support exists).

use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use crate::{
    effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        storage::BoundEffects,
        triggers::time::{
            bridges::system::on_time_expires, components::EffectTimers,
            messages::EffectTimerExpired,
        },
        types::{ReversibleEffectType, ScopedTree, Tree, Trigger, TriggerContext},
        walking::{UntilApplied, walk_effects::walk_bound_effects},
    },
    prelude::*,
};

// -- Locally-duplicated bridge helpers --------------------------------

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

fn test_source(name: &str) -> SourceId {
    SourceId::from(name.to_owned())
}

fn until_time_expires_speed_tree(duration: f32, multiplier: f32) -> Tree {
    Tree::Until(
        Trigger::TimeExpires(OrderedFloat(duration)),
        Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
            SpeedBoostConfig {
                multiplier: OrderedFloat(multiplier),
            },
        ))),
    )
}

/// Synchronously walks `BoundEffects` with the given trigger inside the
/// test `App`. Used to perform the arming step before driving the bridge.
fn walk_bound_in_app(app: &mut App, entity: Entity, trigger: Trigger) {
    let trees = app
        .world()
        .get::<BoundEffects>(entity)
        .map(|b| b.0.clone())
        .unwrap_or_default();
    let world = app.world_mut();
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, world);
        walk_bound_effects(
            entity,
            &trigger,
            &TriggerContext::None,
            &trees,
            &mut commands,
        );
    }
    queue.apply(world);
}

// ----------------------------------------------------------------
// A4: Two Untils on the same entity, same duration, distinct sources —
//     only the source-matched Until reverses on its bridge dispatch.
// ----------------------------------------------------------------

#[test]
fn until_time_expires_source_keyed_dispatch_only_matching_source_reverses() {
    let mut app = bridge_test_app();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![
            (
                "chip_a".to_string(),
                until_time_expires_speed_tree(2.0, 1.5),
            ),
            (
                "chip_b".to_string(),
                until_time_expires_speed_tree(2.0, 1.7),
            ),
        ]))
        .id();

    // Arm both via a NodeStartOccurred walk
    walk_bound_in_app(&mut app, entity, Trigger::NodeStartOccurred);

    // Sanity: stack has 2, EffectTimers has 2, UntilApplied has both
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist after both Untils arm");
    assert_eq!(stack.len(), 2);
    let timers = app
        .world()
        .get::<EffectTimers>(entity)
        .expect("EffectTimers should be armed for both Untils");
    assert_eq!(timers.timers.len(), 2);
    {
        let third_fields: Vec<SourceId> = timers.timers.iter().map(|(_, _, s)| s.clone()).collect();
        assert!(third_fields.contains(&test_source("chip_a")));
        assert!(third_fields.contains(&test_source("chip_b")));
    }
    let until_applied = app
        .world()
        .get::<UntilApplied>(entity)
        .expect("UntilApplied should be present");
    assert!(until_applied.0.contains("chip_a"));
    assert!(until_applied.0.contains("chip_b"));

    // Inject the chip_a expiry message and tick once
    app.insert_resource(TestTimerExpiredMessages(vec![EffectTimerExpired {
        entity,
        original_duration: OrderedFloat(2.0),
        source: test_source("chip_a"),
    }]));
    tick(&mut app);

    // Stack: only chip_b remains, multiplier 1.7
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should still exist (chip_b active)");
    assert_eq!(
        stack.len(),
        1,
        "only chip_b should remain after chip_a reverses"
    );
    let entries: Vec<&(SourceId, SpeedBoostConfig)> = stack.iter().collect();
    assert_eq!(entries[0].0, test_source("chip_b"));
    assert_eq!(entries[0].1.multiplier, OrderedFloat(1.7));

    // EffectTimers: only chip_b's entry remains
    let timers_after = app.world().get::<EffectTimers>(entity);
    let only_chip_b = match timers_after {
        Some(t) => t.timers.len() == 1 && t.timers[0].2 == test_source("chip_b"),
        None => false,
    };
    assert!(
        only_chip_b,
        "only chip_b's EffectTimers entry should remain after chip_a reverses"
    );

    // UntilApplied: chip_b yes, chip_a no
    let until_applied_after = app
        .world()
        .get::<UntilApplied>(entity)
        .expect("UntilApplied should still be present");
    assert!(until_applied_after.0.contains("chip_b"));
    assert!(!until_applied_after.0.contains("chip_a"));

    // BoundEffects: exactly one entry named chip_b
    let bound = app.world().get::<BoundEffects>(entity).unwrap();
    let names: Vec<&String> = bound.0.iter().map(|(n, _)| n).collect();
    assert_eq!(
        names.iter().filter(|&&n| n == "chip_b").count(),
        1,
        "BoundEffects should have exactly one chip_b entry"
    );
    assert_eq!(
        names.iter().filter(|&&n| n == "chip_a").count(),
        0,
        "BoundEffects should not have any chip_a entry"
    );
}

#[test]
fn until_time_expires_source_keyed_dispatch_second_message_clears_remaining_until() {
    let mut app = bridge_test_app();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![
            (
                "chip_a".to_string(),
                until_time_expires_speed_tree(2.0, 1.5),
            ),
            (
                "chip_b".to_string(),
                until_time_expires_speed_tree(2.0, 1.7),
            ),
        ]))
        .id();

    walk_bound_in_app(&mut app, entity, Trigger::NodeStartOccurred);

    // Reverse chip_a
    app.insert_resource(TestTimerExpiredMessages(vec![EffectTimerExpired {
        entity,
        original_duration: OrderedFloat(2.0),
        source: test_source("chip_a"),
    }]));
    tick(&mut app);

    // Reverse chip_b
    app.insert_resource(TestTimerExpiredMessages(vec![EffectTimerExpired {
        entity,
        original_duration: OrderedFloat(2.0),
        source: test_source("chip_b"),
    }]));
    tick(&mut app);

    let stack_empty = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .is_none_or(EffectStack::is_empty);
    assert!(
        stack_empty,
        "EffectStack should be empty after both reverse"
    );

    assert!(
        app.world().get::<EffectTimers>(entity).is_none(),
        "EffectTimers component should be removed when both timer entries are gone"
    );

    let until_applied = app
        .world()
        .get::<UntilApplied>(entity)
        .expect("UntilApplied should still be present");
    assert!(until_applied.0.is_empty());

    let bound = app.world().get::<BoundEffects>(entity).unwrap();
    assert!(
        bound.0.is_empty(),
        "BoundEffects should be empty after both Untils reverse"
    );
}
