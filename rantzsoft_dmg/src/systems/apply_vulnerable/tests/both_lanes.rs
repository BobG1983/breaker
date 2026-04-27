use super::helpers::{assert_f32_eq, drain_messages, enqueue, mk_msg, test_app, tick};
use crate::{SourceId, components::VulnerableStack};

// ── Behavior 89: both lanes — product, one-shots drain, persistent stays ──

#[test]
fn both_lanes_populated_multiplies_by_product() {
    let mut app = test_app();
    let target = app
        .world_mut()
        .spawn({
            let mut s = VulnerableStack::default();
            s.add(SourceId::from("src:x"), 2.0);
            s.add_one_shot(3.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(None, target, 10.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 60.0);

    let mut stack = app.world_mut().get_mut::<VulnerableStack>(target).unwrap();
    assert_f32_eq(stack.aggregate_persistent(), 2.0);
    assert_f32_eq(stack.aggregate_and_consume_one_shots(), 1.0);
}

#[test]
fn both_lanes_multi_entry_multiplies_by_product_of_products() {
    // Edge case 89a.
    let mut app = test_app();
    let target = app
        .world_mut()
        .spawn({
            let mut s = VulnerableStack::default();
            s.add(SourceId::from("src:a"), 2.0);
            s.add(SourceId::from("src:b"), 5.0);
            s.add_one_shot(3.0);
            s.add_one_shot(4.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(None, target, 1.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 120.0);

    let stack = app.world().get::<VulnerableStack>(target).unwrap();
    assert_f32_eq(stack.aggregate_persistent(), 10.0);
}

#[test]
fn both_lanes_two_messages_only_first_gets_one_shots() {
    // Edge case 89b.
    let mut app = test_app();
    let target = app
        .world_mut()
        .spawn({
            let mut s = VulnerableStack::default();
            s.add(SourceId::from("src:a"), 2.0);
            s.add_one_shot(3.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(None, target, 10.0));
    enqueue(&mut app, mk_msg(None, target, 10.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 2);
    assert_f32_eq(drained[0].amount, 60.0);
    assert_f32_eq(drained[1].amount, 20.0);
}
