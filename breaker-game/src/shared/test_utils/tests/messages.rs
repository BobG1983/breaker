use bevy::prelude::*;

use super::{
    super::*,
    helpers::{
        ShouldSend, conditional_damage_sender, damage_and_bolt_lost_sender, damage_sender_system,
        triple_damage_sender,
    },
};
use crate::{bolt::messages::BoltLost, cells::messages::DamageCell};

// ════════════════════════════════════════════════════════════════════
// Section H: with_message()
// ════════════════════════════════════════════════════════════════════

// ── Behavior 14: with_message() registers a message type ──

/// Helper: system that reads `DamageCell` messages and counts them in a resource.
#[derive(Resource, Default)]
struct DamageCount(usize);

fn count_damage_messages(mut reader: MessageReader<DamageCell>, mut count: ResMut<DamageCount>) {
    for _msg in reader.read() {
        count.0 += 1;
    }
}

fn damage_sender_10(mut writer: MessageWriter<DamageCell>) {
    writer.write(DamageCell {
        cell:        Entity::PLACEHOLDER,
        damage:      10.0,
        source_chip: None,
    });
}

#[test]
fn with_message_enables_message_send_and_read() {
    let mut app = TestAppBuilder::new()
        .with_message::<DamageCell>()
        .with_resource::<DamageCount>()
        .with_system(FixedUpdate, damage_sender_10)
        .with_system(FixedUpdate, count_damage_messages.after(damage_sender_10))
        .build();
    tick(&mut app);
    assert_eq!(
        app.world().resource::<DamageCount>().0,
        1,
        "with_message should enable sending and reading DamageCell messages"
    );
}

#[test]
fn with_message_does_not_add_collector() {
    let app = TestAppBuilder::new().with_message::<DamageCell>().build();
    assert!(
        app.world()
            .get_resource::<MessageCollector<DamageCell>>()
            .is_none(),
        "with_message should NOT add a MessageCollector"
    );
}

// ════════════════════════════════════════════════════════════════════
// Section I: MessageCollector and with_message_capture()
// ════════════════════════════════════════════════════════════════════

// ── Behavior 15: with_message_capture() registers collector ──

#[test]
fn with_message_capture_registers_collector() {
    let app = TestAppBuilder::new()
        .with_message_capture::<DamageCell>()
        .build();
    let collector = app.world().get_resource::<MessageCollector<DamageCell>>();
    assert!(
        collector.is_some(),
        "with_message_capture must register MessageCollector<DamageCell>"
    );
    assert_eq!(
        collector.unwrap().0.len(),
        0,
        "MessageCollector should start empty"
    );
}

// ── Behavior 16: MessageCollector captures messages ──

#[test]
fn message_collector_captures_messages_during_tick() {
    let mut app = TestAppBuilder::new()
        .with_message_capture::<DamageCell>()
        .with_system(FixedUpdate, damage_sender_system)
        .build();
    tick(&mut app);
    let collector = app.world().resource::<MessageCollector<DamageCell>>();
    assert_eq!(
        collector.0.len(),
        1,
        "MessageCollector should capture 1 message after tick"
    );
    assert!(
        (collector.0[0].damage - 25.0).abs() < f32::EPSILON,
        "Captured message damage should be 25.0, got {}",
        collector.0[0].damage
    );
}

#[test]
fn message_collector_captures_multiple_messages_per_tick() {
    let mut app = TestAppBuilder::new()
        .with_message_capture::<DamageCell>()
        .with_system(FixedUpdate, triple_damage_sender)
        .build();
    tick(&mut app);
    assert_eq!(
        app.world()
            .resource::<MessageCollector<DamageCell>>()
            .0
            .len(),
        3,
        "MessageCollector should capture all 3 messages sent in one tick"
    );
}

// ── Behavior 17: MessageCollector auto-clears at start of each tick ──

#[test]
fn message_collector_auto_clears_between_ticks() {
    let mut app = TestAppBuilder::new()
        .with_message_capture::<DamageCell>()
        .insert_resource(ShouldSend(true))
        .with_system(FixedUpdate, conditional_damage_sender)
        .build();

    // First tick — flag is true, 1 message
    tick(&mut app);
    assert_eq!(
        app.world()
            .resource::<MessageCollector<DamageCell>>()
            .0
            .len(),
        1,
        "First tick: should have 1 message"
    );

    // Second tick — flag is false, no messages sent
    app.world_mut().resource_mut::<ShouldSend>().0 = false;
    tick(&mut app);
    assert_eq!(
        app.world()
            .resource::<MessageCollector<DamageCell>>()
            .0
            .len(),
        0,
        "Second tick: auto-clear should empty collector when no messages sent"
    );

    // Third tick — flag back to true
    app.world_mut().resource_mut::<ShouldSend>().0 = true;
    tick(&mut app);
    assert_eq!(
        app.world()
            .resource::<MessageCollector<DamageCell>>()
            .0
            .len(),
        1,
        "Third tick: collector should have 1 message again (clear-then-collect repeatable)"
    );
}

// ── Behavior 18: MessageCollector::clear() manual reset ──

#[test]
fn message_collector_manual_clear() {
    let mut app = TestAppBuilder::new()
        .with_message_capture::<DamageCell>()
        .with_system(FixedUpdate, damage_sender_system)
        .build();
    tick(&mut app);
    assert_eq!(
        app.world()
            .resource::<MessageCollector<DamageCell>>()
            .0
            .len(),
        1,
    );

    app.world_mut()
        .resource_mut::<MessageCollector<DamageCell>>()
        .clear();
    assert_eq!(
        app.world()
            .resource::<MessageCollector<DamageCell>>()
            .0
            .len(),
        0,
        "clear() should empty the collector"
    );
}

#[test]
fn message_collector_clear_on_empty_does_not_panic() {
    let mut app = TestAppBuilder::new()
        .with_message_capture::<DamageCell>()
        .build();
    app.world_mut()
        .resource_mut::<MessageCollector<DamageCell>>()
        .clear();
}

// ── Behavior 19: Multiple collectors coexist ──

#[test]
fn multiple_message_collectors_coexist() {
    let mut app = TestAppBuilder::new()
        .with_message_capture::<DamageCell>()
        .with_message_capture::<BoltLost>()
        .with_system(FixedUpdate, damage_and_bolt_lost_sender)
        .build();
    tick(&mut app);
    assert_eq!(
        app.world()
            .resource::<MessageCollector<DamageCell>>()
            .0
            .len(),
        1,
        "DamageCell collector should have 1 message"
    );
    assert_eq!(
        app.world().resource::<MessageCollector<BoltLost>>().0.len(),
        1,
        "BoltLost collector should have 1 message"
    );
}

#[test]
fn clearing_one_collector_does_not_affect_other() {
    let mut app = TestAppBuilder::new()
        .with_message_capture::<DamageCell>()
        .with_message_capture::<BoltLost>()
        .with_system(FixedUpdate, damage_and_bolt_lost_sender)
        .build();
    tick(&mut app);

    app.world_mut()
        .resource_mut::<MessageCollector<DamageCell>>()
        .clear();
    assert_eq!(
        app.world()
            .resource::<MessageCollector<DamageCell>>()
            .0
            .len(),
        0,
        "DamageCell collector should be empty after clear"
    );
    assert_eq!(
        app.world().resource::<MessageCollector<BoltLost>>().0.len(),
        1,
        "BoltLost collector should be unaffected by clearing DamageCell"
    );
}

// ── Behavior 20: accumulation pattern across ticks ──

#[test]
fn message_collector_per_tick_isolation() {
    let mut app = TestAppBuilder::new()
        .with_message_capture::<DamageCell>()
        .with_system(FixedUpdate, damage_sender_system)
        .build();

    let mut running_total = 0usize;
    for i in 1..=3 {
        tick(&mut app);
        let count = app
            .world()
            .resource::<MessageCollector<DamageCell>>()
            .0
            .len();
        running_total += count;
        assert_eq!(
            count, 1,
            "Tick {i}: collector should have exactly 1 message (auto-clear isolates ticks)"
        );
    }
    assert_eq!(running_total, 3, "Running total over 3 ticks should be 3");
}

// ── Behavior 20b: with_message_capture called twice is idempotent ──

#[test]
fn with_message_capture_twice_is_idempotent() {
    let mut app = TestAppBuilder::new()
        .with_message_capture::<DamageCell>()
        .with_message_capture::<DamageCell>()
        .with_system(FixedUpdate, damage_sender_system)
        .build();
    tick(&mut app);
    assert_eq!(
        app.world()
            .resource::<MessageCollector<DamageCell>>()
            .0
            .len(),
        1,
        "Double with_message_capture should not duplicate messages (len should be 1, not 2)"
    );
}

#[test]
fn with_message_then_message_capture_does_not_panic() {
    let mut app = TestAppBuilder::new()
        .with_message::<DamageCell>()
        .with_message_capture::<DamageCell>()
        .with_system(FixedUpdate, damage_sender_system)
        .build();
    tick(&mut app);
    assert_eq!(
        app.world()
            .resource::<MessageCollector<DamageCell>>()
            .0
            .len(),
        1,
        "with_message followed by with_message_capture should capture messages normally"
    );
}
