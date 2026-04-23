//! Per-T message-queue isolation tests for the generic message types.
//!
//! These tests cross multiple message types (`DamageDealt`, `HealDealt`,
//! `KillYourself`, `Destroyed`), so they live at the `messages/` level rather
//! than inside any single message file.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::{DamageDealt, Destroyed, HealDealt, KillYourself};
use crate::{HealCap, traits::Dmgable};

// Test-local dummy `Dmgable` types — distinct markers prove monomorphization
// produces independent `Messages<M<T>>` resources.
#[derive(Component)]
struct TestA;
impl Dmgable for TestA {}

#[derive(Component)]
struct TestB;
impl Dmgable for TestB {}

/// Compares two `f32` values for "equality" without tripping
/// `clippy::float_cmp`. Replicated per file per the test spec
/// copy-paste policy.
#[track_caller]
fn assert_f32_eq(actual: f32, expected: f32) {
    if expected.is_infinite() {
        assert!(
            actual.is_infinite() && actual.is_sign_positive() == expected.is_sign_positive(),
            "expected {expected}, got {actual}"
        );
    } else {
        assert!(
            (actual - expected).abs() < f32::EPSILON,
            "expected {expected}, got {actual}"
        );
    }
}

// ── Behavior 27: Per-T queue isolation — `DamageDealt<TestA>` and
//     `DamageDealt<TestB>` are independent Bevy message queues ──

#[test]
fn damage_dealt_per_t_queue_isolation_test_a_write_test_b_empty() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<DamageDealt<TestA>>();
    app.add_message::<DamageDealt<TestB>>();

    // Write a single message to the TestA queue.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestA>>>()
        .write(DamageDealt::<TestA> {
            dealer:  None,
            target:  Entity::PLACEHOLDER,
            amount:  9.0,
            source:  None,
            _marker: PhantomData,
        });

    app.update();

    // TestA queue must contain exactly one message with amount == 9.0.
    let drained_a: Vec<DamageDealt<TestA>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<TestA>>>()
        .drain()
        .collect();
    assert_eq!(drained_a.len(), 1);
    assert_f32_eq(drained_a[0].amount, 9.0);

    // TestB queue must be empty — proves per-T monomorphization isolates queues.
    let b_empty_count = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<TestB>>>()
        .drain()
        .count();
    assert_eq!(b_empty_count, 0);
}

#[test]
fn damage_dealt_per_t_queue_isolation_test_b_write_test_a_empty() {
    // Edge case (mirror): write to TestB, confirm TestA queue is empty.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<DamageDealt<TestA>>();
    app.add_message::<DamageDealt<TestB>>();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestB>>>()
        .write(DamageDealt::<TestB> {
            dealer:  None,
            target:  Entity::PLACEHOLDER,
            amount:  9.0,
            source:  None,
            _marker: PhantomData,
        });

    app.update();

    let drained_b: Vec<DamageDealt<TestB>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<TestB>>>()
        .drain()
        .collect();
    assert_eq!(drained_b.len(), 1);
    assert_f32_eq(drained_b[0].amount, 9.0);

    let a_empty_count = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<TestA>>>()
        .drain()
        .count();
    assert_eq!(a_empty_count, 0);
}

// ── Behavior 28: Per-T queue isolation extends to `HealDealt`,
//     `KillYourself`, and `Destroyed` ──

#[test]
fn heal_dealt_per_t_queue_isolation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<HealDealt<TestA>>();
    app.add_message::<HealDealt<TestB>>();

    app.world_mut()
        .resource_mut::<Messages<HealDealt<TestA>>>()
        .write(HealDealt::<TestA> {
            healer:  None,
            target:  Entity::PLACEHOLDER,
            amount:  3.0,
            source:  None,
            cap:     HealCap::Starting,
            _marker: PhantomData,
        });

    app.update();

    let b_empty_count = app
        .world_mut()
        .resource_mut::<Messages<HealDealt<TestB>>>()
        .drain()
        .count();
    assert_eq!(b_empty_count, 0);
}

#[test]
fn kill_yourself_per_t_queue_isolation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<KillYourself<TestA>>();
    app.add_message::<KillYourself<TestB>>();

    app.world_mut()
        .resource_mut::<Messages<KillYourself<TestA>>>()
        .write(KillYourself::<TestA> {
            victim:  Entity::PLACEHOLDER,
            killer:  None,
            _marker: PhantomData,
        });

    app.update();

    let b_empty_count = app
        .world_mut()
        .resource_mut::<Messages<KillYourself<TestB>>>()
        .drain()
        .count();
    assert_eq!(b_empty_count, 0);
}

#[test]
fn destroyed_per_t_queue_isolation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<Destroyed<TestA>>();
    app.add_message::<Destroyed<TestB>>();

    app.world_mut()
        .resource_mut::<Messages<Destroyed<TestA>>>()
        .write(Destroyed::<TestA> {
            victim:     Entity::PLACEHOLDER,
            killer:     None,
            victim_pos: Vec2::ZERO,
            killer_pos: None,
            _marker:    PhantomData,
        });

    app.update();

    let b_empty_count = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestB>>>()
        .drain()
        .count();
    assert_eq!(b_empty_count, 0);
}
