use super::helpers::{assert_f32_eq, drain_messages, enqueue, mk_msg, test_app, tick};
use crate::{SourceId, components::VulnerableStack};

// ── Behavior 87: persistent-only VulnerableStack multiplies by aggregate ──

#[test]
fn persistent_only_multiplies_and_retains() {
    let mut app = test_app();
    let target = app
        .world_mut()
        .spawn({
            let mut s = VulnerableStack::default();
            s.add(SourceId::from("src:a"), 2.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(None, target, 4.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 8.0);

    let stack = app.world().get::<VulnerableStack>(target).unwrap();
    assert_f32_eq(stack.aggregate_persistent(), 2.0);
}

#[test]
fn persistent_multi_entry_multiplies_by_product() {
    // Edge case 87a.
    let mut app = test_app();
    let target = app
        .world_mut()
        .spawn({
            let mut s = VulnerableStack::default();
            s.add(SourceId::from("src:a"), 2.0);
            s.add(SourceId::from("src:b"), 3.0);
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
fn persistent_applies_to_each_of_two_messages_same_tick() {
    // Edge case 87b.
    let mut app = test_app();
    let target = app
        .world_mut()
        .spawn({
            let mut s = VulnerableStack::default();
            s.add(SourceId::from("src:a"), 2.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(None, target, 4.0));
    enqueue(&mut app, mk_msg(None, target, 4.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 2);
    assert_f32_eq(drained[0].amount, 8.0);
    assert_f32_eq(drained[1].amount, 8.0);
}
