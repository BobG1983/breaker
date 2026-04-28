use super::helpers::{assert_f32_eq, drain_messages, enqueue, mk_msg, test_app, tick};
use crate::components::VulnerableStack;

// ── Behavior 88: one-shot-only multiplies and drains ──

#[test]
fn one_shot_only_multiplies_and_drains() {
    let mut app = test_app();
    let target = app
        .world_mut()
        .spawn({
            let mut s = VulnerableStack::default();
            s.add_one_shot(2.5);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(None, target, 4.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 10.0);

    let mut stack = app.world_mut().get_mut::<VulnerableStack>(target).unwrap();
    assert_f32_eq(stack.aggregate_and_consume_one_shots(None), 1.0);
}

#[test]
fn one_shot_multi_entry_multiplies_by_product() {
    // Edge case 88a.
    let mut app = test_app();
    let target = app
        .world_mut()
        .spawn({
            let mut s = VulnerableStack::default();
            s.add_one_shot(2.0);
            s.add_one_shot(3.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(None, target, 1.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 6.0);
}

#[test]
fn one_shot_drained_by_first_message_is_identity_for_second_same_tick() {
    // Edge case 88b.
    let mut app = test_app();
    let target = app
        .world_mut()
        .spawn({
            let mut s = VulnerableStack::default();
            s.add_one_shot(2.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(None, target, 5.0));
    enqueue(&mut app, mk_msg(None, target, 5.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 2);
    assert_f32_eq(drained[0].amount, 10.0);
    assert_f32_eq(drained[1].amount, 5.0);
}
