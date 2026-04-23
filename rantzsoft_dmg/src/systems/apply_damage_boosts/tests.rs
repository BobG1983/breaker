use std::marker::PhantomData;

use bevy::prelude::*;

use super::system::*;
use crate::{SourceId, components::DamageBoostStack, messages::DamageDealt, traits::Dmgable};

#[derive(Component)]
struct TestT;
impl Dmgable for TestT {}

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

/// Build a bare App with `MinimalPlugins`, `Messages<DamageDealt<TestT>>`
/// registered, and `apply_damage_boosts::<TestT>` scheduled in
/// `FixedUpdate`. No `RantzDmgPlugin` — we exercise this system in
/// isolation.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<DamageDealt<TestT>>();
    app.add_systems(FixedUpdate, apply_damage_boosts::<TestT>);
    app
}

fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

fn enqueue(app: &mut App, msg: DamageDealt<TestT>) {
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(msg);
}

fn drain_messages(app: &mut App) -> Vec<DamageDealt<TestT>> {
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .drain()
        .collect()
}

fn mk_msg(dealer: Option<Entity>, amount: f32) -> DamageDealt<TestT> {
    DamageDealt::<TestT> {
        dealer,
        target: Entity::PLACEHOLDER,
        amount,
        source: None,
        _marker: PhantomData,
    }
}

// ── Behavior 82: persistent-only `DamageBoostStack` multiplies by
//     persistent aggregate, lane is NOT drained ──

#[test]
fn persistent_only_stack_multiplies_and_retains() {
    let mut app = test_app();
    let dealer = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add(SourceId::from("src:a"), 2.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(Some(dealer), 10.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 20.0);

    // Persistent lane retained.
    let stack = app.world().get::<DamageBoostStack>(dealer).unwrap();
    assert_f32_eq(stack.aggregate_persistent(), 2.0);
}

#[test]
fn persistent_only_stack_applies_to_each_of_two_messages_same_tick() {
    // Edge case 82a.
    let mut app = test_app();
    let dealer = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add(SourceId::from("src:a"), 2.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(Some(dealer), 10.0));
    enqueue(&mut app, mk_msg(Some(dealer), 10.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 2);
    assert_f32_eq(drained[0].amount, 20.0);
    assert_f32_eq(drained[1].amount, 20.0);
}

// ── Behavior 83: one-shot-only stack multiplies and drains the lane ──

#[test]
fn one_shot_only_stack_multiplies_and_drains() {
    let mut app = test_app();
    let dealer = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add_one_shot(3.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(Some(dealer), 10.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 30.0);

    // One-shot lane drained.
    let mut stack = app.world_mut().get_mut::<DamageBoostStack>(dealer).unwrap();
    assert_f32_eq(stack.aggregate_and_consume_one_shots(), 1.0);
}

#[test]
fn one_shot_multi_entry_stack_multiplies_by_product() {
    // Edge case 83a.
    let mut app = test_app();
    let dealer = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add_one_shot(2.0);
            s.add_one_shot(3.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(Some(dealer), 10.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 60.0);
}

#[test]
fn one_shot_drained_by_first_message_is_identity_for_second_same_tick() {
    // Edge case 83b.
    let mut app = test_app();
    let dealer = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add_one_shot(3.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(Some(dealer), 10.0));
    enqueue(&mut app, mk_msg(Some(dealer), 10.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 2);
    assert_f32_eq(drained[0].amount, 30.0);
    assert_f32_eq(drained[1].amount, 10.0);
}

// ── Behavior 84: both lanes populated — product of both, one-shots drain,
//     persistent retained ──

#[test]
fn both_lanes_populated_multiplies_by_product() {
    let mut app = test_app();
    let dealer = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add(SourceId::from("src:x"), 2.0);
            s.add_one_shot(3.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(Some(dealer), 10.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 60.0);

    // Post-tick: persistent retained, one-shots drained.
    let mut stack = app.world_mut().get_mut::<DamageBoostStack>(dealer).unwrap();
    assert_f32_eq(stack.aggregate_persistent(), 2.0);
    assert_f32_eq(stack.aggregate_and_consume_one_shots(), 1.0);
}

#[test]
fn both_lanes_multi_entry_multiplies_by_product_of_products() {
    // Edge case 84a.
    let mut app = test_app();
    let dealer = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add(SourceId::from("src:a"), 2.0);
            s.add(SourceId::from("src:b"), 5.0);
            s.add_one_shot(3.0);
            s.add_one_shot(4.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(Some(dealer), 1.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 120.0);

    let stack = app.world().get::<DamageBoostStack>(dealer).unwrap();
    assert_f32_eq(stack.aggregate_persistent(), 10.0);
}

#[test]
fn both_lanes_two_messages_only_first_gets_one_shots() {
    // Edge case 84b.
    let mut app = test_app();
    let dealer = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add(SourceId::from("src:a"), 2.0);
            s.add_one_shot(3.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(Some(dealer), 10.0));
    enqueue(&mut app, mk_msg(Some(dealer), 10.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 2);
    assert_f32_eq(drained[0].amount, 60.0);
    assert_f32_eq(drained[1].amount, 20.0);
}

// ── Behavior 85: multi-entry persistent lane multiplies by PRODUCT (not
//     sum, not identity) ──

#[test]
fn multi_entry_persistent_multiplies_by_product() {
    let mut app = test_app();
    let dealer = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add(SourceId::from("src:a"), 2.0);
            s.add(SourceId::from("src:b"), 3.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(Some(dealer), 10.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 60.0);

    let stack = app.world().get::<DamageBoostStack>(dealer).unwrap();
    assert_f32_eq(stack.aggregate_persistent(), 6.0);
}

#[test]
fn three_entry_persistent_multiplies_by_product() {
    // Edge case 85a.
    let mut app = test_app();
    let dealer = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add(SourceId::from("src:a"), 2.0);
            s.add(SourceId::from("src:b"), 3.0);
            s.add(SourceId::from("src:c"), 5.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(Some(dealer), 1.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 30.0);
}

#[test]
fn empty_stack_is_identity() {
    // Edge case 85b — dealer has component, both lanes empty → identity.
    let mut app = test_app();
    let dealer = app.world_mut().spawn(DamageBoostStack::default()).id();

    enqueue(&mut app, mk_msg(Some(dealer), 4.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 4.0);
}

// ── Behavior 86: pass-through for dealer: None, missing stack, despawned
//     dealer; independent dealers isolated ──

#[test]
fn dealer_none_passes_through_unchanged() {
    let mut app = test_app();
    enqueue(&mut app, mk_msg(None, 10.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 10.0);
}

#[test]
fn dealer_none_does_not_consume_unrelated_stacks() {
    // Edge case 86a.
    let mut app = test_app();
    let unrelated = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add(SourceId::from("src:a"), 99.0);
            s.add_one_shot(99.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(None, 7.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 7.0);

    let mut stack = app
        .world_mut()
        .get_mut::<DamageBoostStack>(unrelated)
        .unwrap();
    assert_f32_eq(stack.aggregate_persistent(), 99.0);
    assert_f32_eq(stack.aggregate_and_consume_one_shots(), 99.0);
}

#[test]
fn dealer_without_stack_passes_through() {
    // Edge case 86b.
    let mut app = test_app();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), 5.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 5.0);
}

#[test]
fn despawned_dealer_handle_passes_through() {
    // Edge case 86c.
    let mut app = test_app();
    let dealer = app.world_mut().spawn_empty().id();
    app.world_mut().despawn(dealer);

    enqueue(&mut app, mk_msg(Some(dealer), 5.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 5.0);
}

#[test]
fn two_independent_dealers_apply_independently() {
    // Edge case 86d.
    let mut app = test_app();
    let d1 = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add(SourceId::from("src:a"), 2.0);
            s.add_one_shot(5.0);
            s
        })
        .id();
    let d2 = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add_one_shot(3.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(Some(d1), 10.0));
    enqueue(&mut app, mk_msg(Some(d2), 10.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 2);
    assert_f32_eq(drained[0].amount, 100.0);
    assert_f32_eq(drained[1].amount, 30.0);

    let d1_stack = app.world().get::<DamageBoostStack>(d1).unwrap();
    assert_f32_eq(d1_stack.aggregate_persistent(), 2.0);
}

#[test]
fn three_mixed_dealers_produce_expected_amounts() {
    // Edge case 86e.
    let mut app = test_app();
    let d1 = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add(SourceId::from("src:a"), 2.0);
            s
        })
        .id();
    let d2 = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add_one_shot(3.0);
            s
        })
        .id();
    let d3 = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(d1), 1.0));
    enqueue(&mut app, mk_msg(Some(d2), 1.0));
    enqueue(&mut app, mk_msg(Some(d3), 1.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 3);
    assert_f32_eq(drained[0].amount, 2.0);
    assert_f32_eq(drained[1].amount, 3.0);
    assert_f32_eq(drained[2].amount, 1.0);
}
