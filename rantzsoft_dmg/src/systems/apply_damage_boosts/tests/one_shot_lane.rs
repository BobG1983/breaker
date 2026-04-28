use super::helpers::{assert_f32_eq, drain_messages, enqueue, mk_msg, test_app, tick};
use crate::components::DamageBoostStack;

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
    assert_f32_eq(stack.aggregate_and_consume_one_shots(None), 1.0);
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
