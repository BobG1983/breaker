use super::helpers::{assert_f32_eq, drain_messages, enqueue, mk_msg, test_app, tick};
use crate::{SourceId, components::DamageBoostStack};

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
