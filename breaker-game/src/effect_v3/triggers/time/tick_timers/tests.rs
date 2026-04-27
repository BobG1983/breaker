use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::system::tick_effect_timers;
use crate::{
    effect_v3::triggers::time::{components::EffectTimers, messages::EffectTimerExpired},
    prelude::*,
};

// -- Helpers ----------------------------------------------------------

fn timer_test_app() -> App {
    TestAppBuilder::new()
        .with_message_capture::<EffectTimerExpired>()
        .with_system(FixedUpdate, tick_effect_timers)
        .build()
}

fn test_source(name: &str) -> SourceId {
    SourceId::from(name.to_owned())
}

// -- Behavior 1: single timer decrements by delta time each tick ------

#[test]
fn single_timer_decrements_by_delta_time() {
    let mut app = timer_test_app();

    let entity = app
        .world_mut()
        .spawn(EffectTimers {
            timers: vec![(
                OrderedFloat(1.0),
                OrderedFloat(1.0),
                test_source("test_source"),
            )],
        })
        .id();

    tick(&mut app);

    let timers = app
        .world()
        .get::<EffectTimers>(entity)
        .expect("EffectTimers should still be present");
    assert_eq!(timers.timers.len(), 1);
    // TestAppBuilder uses a 64 Hz fixed timestep (dt = 1/64 s).
    // 1.0 - 1/64 = 0.984_375 — exact in f32 since 1/64 = 2^-6.
    assert_eq!(
        timers.timers[0].0,
        OrderedFloat(0.984_375),
        "remaining should be 1.0 - 1/64 = 0.984_375"
    );
}

#[test]
fn timer_at_zero_immediately_expires_and_sends_message() {
    let mut app = timer_test_app();

    let entity = app
        .world_mut()
        .spawn(EffectTimers {
            timers: vec![(
                OrderedFloat(0.0),
                OrderedFloat(1.0),
                test_source("test_source"),
            )],
        })
        .id();

    tick(&mut app);

    assert!(
        app.world().get::<EffectTimers>(entity).is_none(),
        "EffectTimers component should be removed after timer expires"
    );

    let collector = app
        .world()
        .resource::<MessageCollector<EffectTimerExpired>>();
    assert_eq!(collector.0.len(), 1, "should send one EffectTimerExpired");
    assert_eq!(collector.0[0].entity, entity);
    assert_eq!(collector.0[0].original_duration, OrderedFloat(1.0));
    assert_eq!(collector.0[0].source, test_source("test_source"));
}

// -- Behavior 2: timer reaching zero sends EffectTimerExpired ---------

#[test]
fn timer_reaching_zero_sends_expired_message_with_correct_original_duration() {
    let mut app = timer_test_app();

    let entity = app
        .world_mut()
        .spawn(EffectTimers {
            timers: vec![(
                OrderedFloat(0.01),
                OrderedFloat(5.0),
                test_source("test_source"),
            )],
        })
        .id();

    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<EffectTimerExpired>>();
    assert_eq!(collector.0.len(), 1, "should send one EffectTimerExpired");
    assert_eq!(collector.0[0].entity, entity);
    assert_eq!(
        collector.0[0].original_duration,
        OrderedFloat(5.0),
        "original_duration should be 5.0"
    );
    assert_eq!(collector.0[0].source, test_source("test_source"));
}

#[test]
fn timer_going_negative_still_triggers_expired() {
    let mut app = timer_test_app();

    // remaining 0.005 < dt 0.015625 — will go negative
    let entity = app
        .world_mut()
        .spawn(EffectTimers {
            timers: vec![(
                OrderedFloat(0.005),
                OrderedFloat(3.0),
                test_source("test_source"),
            )],
        })
        .id();

    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<EffectTimerExpired>>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(collector.0[0].entity, entity);
    assert_eq!(collector.0[0].original_duration, OrderedFloat(3.0));
    assert_eq!(collector.0[0].source, test_source("test_source"));
}

// -- Behavior 3: EffectTimers component removed when all entries expire

#[test]
fn component_removed_when_all_entries_expire() {
    let mut app = timer_test_app();

    let entity = app
        .world_mut()
        .spawn(EffectTimers {
            timers: vec![(
                OrderedFloat(0.001),
                OrderedFloat(3.0),
                test_source("test_source"),
            )],
        })
        .id();

    tick(&mut app);

    assert!(
        app.world().get::<EffectTimers>(entity).is_none(),
        "EffectTimers component should be removed when all timers expire"
    );
}

#[test]
fn empty_timers_vec_removes_component() {
    let mut app = timer_test_app();

    let entity = app.world_mut().spawn(EffectTimers { timers: vec![] }).id();

    tick(&mut app);

    assert!(
        app.world().get::<EffectTimers>(entity).is_none(),
        "empty EffectTimers should be removed immediately"
    );
}

// -- Behavior 4: multiple timers, only expired ones are removed -------

#[test]
fn multiple_timers_only_expired_ones_removed() {
    let mut app = timer_test_app();

    let entity = app
        .world_mut()
        .spawn(EffectTimers {
            timers: vec![
                (
                    OrderedFloat(0.001),
                    OrderedFloat(2.0),
                    test_source("src_short"),
                ),
                (
                    OrderedFloat(10.0),
                    OrderedFloat(10.0),
                    test_source("src_long"),
                ),
            ],
        })
        .id();

    tick(&mut app);

    let timers = app
        .world()
        .get::<EffectTimers>(entity)
        .expect("EffectTimers should still be present (second timer alive)");
    assert_eq!(
        timers.timers.len(),
        1,
        "only the second timer should remain"
    );
    assert_eq!(
        timers.timers[0].2,
        test_source("src_long"),
        "surviving entry's source should be src_long"
    );

    let collector = app
        .world()
        .resource::<MessageCollector<EffectTimerExpired>>();
    assert_eq!(collector.0.len(), 1, "only one timer should have expired");
    assert_eq!(collector.0[0].original_duration, OrderedFloat(2.0));
    assert_eq!(
        collector.0[0].source,
        test_source("src_short"),
        "emitted message source should be src_short"
    );
}

#[test]
fn both_timers_expire_in_same_frame_sends_two_messages_and_removes_component() {
    let mut app = timer_test_app();

    let entity = app
        .world_mut()
        .spawn(EffectTimers {
            timers: vec![
                (OrderedFloat(0.001), OrderedFloat(2.0), test_source("src_a")),
                (OrderedFloat(0.001), OrderedFloat(4.0), test_source("src_b")),
            ],
        })
        .id();

    tick(&mut app);

    assert!(
        app.world().get::<EffectTimers>(entity).is_none(),
        "component should be removed when both timers expire"
    );

    let collector = app
        .world()
        .resource::<MessageCollector<EffectTimerExpired>>();
    assert_eq!(
        collector.0.len(),
        2,
        "two EffectTimerExpired messages should be sent"
    );

    let durations: Vec<OrderedFloat<f32>> =
        collector.0.iter().map(|m| m.original_duration).collect();
    assert!(durations.contains(&OrderedFloat(2.0)));
    assert!(durations.contains(&OrderedFloat(4.0)));

    let sources: Vec<SourceId> = collector.0.iter().map(|m| m.source.clone()).collect();
    assert!(
        sources.contains(&test_source("src_a")),
        "collector should contain a message with source src_a"
    );
    assert!(
        sources.contains(&test_source("src_b")),
        "collector should contain a message with source src_b"
    );
}

// -- Behavior 5: multiple entities with independent timers ------------

#[test]
fn multiple_entities_independent_timers() {
    let mut app = timer_test_app();

    let entity_a = app
        .world_mut()
        .spawn(EffectTimers {
            timers: vec![(
                OrderedFloat(0.001),
                OrderedFloat(1.0),
                test_source("entity_a"),
            )],
        })
        .id();

    let entity_b = app
        .world_mut()
        .spawn(EffectTimers {
            timers: vec![(
                OrderedFloat(100.0),
                OrderedFloat(100.0),
                test_source("entity_b"),
            )],
        })
        .id();

    tick(&mut app);

    assert!(
        app.world().get::<EffectTimers>(entity_a).is_none(),
        "entity_a timer should have expired and component removed"
    );

    let timers_b = app
        .world()
        .get::<EffectTimers>(entity_b)
        .expect("entity_b should still have EffectTimers");
    assert_eq!(timers_b.timers.len(), 1);

    let collector = app
        .world()
        .resource::<MessageCollector<EffectTimerExpired>>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(collector.0[0].entity, entity_a);
    assert_eq!(collector.0[0].source, test_source("entity_a"));
}

#[test]
fn no_entities_with_timers_is_noop() {
    let mut app = timer_test_app();

    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<EffectTimerExpired>>();
    assert!(collector.0.is_empty(), "no messages should be sent");
}

// -- Behavior 6: entity field matches the entity that owned the timer -

#[test]
fn expired_message_entity_matches_owner() {
    let mut app = timer_test_app();

    let entity_a = app
        .world_mut()
        .spawn(EffectTimers {
            timers: vec![(
                OrderedFloat(0.001),
                OrderedFloat(7.5),
                test_source("test_source"),
            )],
        })
        .id();

    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<EffectTimerExpired>>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(
        collector.0[0].entity, entity_a,
        "EffectTimerExpired entity should match the entity that owned the timer"
    );
    assert_eq!(
        collector.0[0].original_duration,
        OrderedFloat(7.5),
        "original_duration should match"
    );
    assert_eq!(
        collector.0[0].source,
        test_source("test_source"),
        "source should match the entry's source"
    );
}

// -- C2 (NEW): tick_effect_timers propagates third tuple field --------

#[test]
fn tick_effect_timers_propagates_source_into_expired_message() {
    let mut app = timer_test_app();

    let entity = app
        .world_mut()
        .spawn(EffectTimers {
            timers: vec![(
                OrderedFloat(0.001),
                OrderedFloat(3.0),
                test_source("chip_a"),
            )],
        })
        .id();

    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<EffectTimerExpired>>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(collector.0[0].entity, entity);
    assert_eq!(collector.0[0].original_duration, OrderedFloat(3.0));
    assert_eq!(collector.0[0].source, test_source("chip_a"));
}

// -- C2 edge case: two entries with distinct sources both expire ------

#[test]
fn tick_effect_timers_propagates_distinct_sources_for_simultaneous_expiry() {
    let mut app = timer_test_app();

    let entity = app
        .world_mut()
        .spawn(EffectTimers {
            timers: vec![
                (
                    OrderedFloat(0.001),
                    OrderedFloat(2.0),
                    test_source("chip_a"),
                ),
                (
                    OrderedFloat(0.001),
                    OrderedFloat(4.0),
                    test_source("chip_b"),
                ),
            ],
        })
        .id();

    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<EffectTimerExpired>>();
    assert_eq!(collector.0.len(), 2);

    // Build (duration, source) pairs and assert exact multiset match
    let mut pairs: Vec<(OrderedFloat<f32>, SourceId)> = collector
        .0
        .iter()
        .map(|m| (m.original_duration, m.source.clone()))
        .collect();
    let mut expected: Vec<(OrderedFloat<f32>, SourceId)> = vec![
        (OrderedFloat(2.0), test_source("chip_a")),
        (OrderedFloat(4.0), test_source("chip_b")),
    ];
    pairs.sort_by_key(|a| a.0);
    expected.sort_by_key(|a| a.0);
    assert_eq!(
        pairs, expected,
        "the (duration, source) multiset should exactly match the inputs"
    );

    for msg in &collector.0 {
        assert_eq!(msg.entity, entity);
    }
}
