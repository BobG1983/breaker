use super::helpers::{assert_f32_eq, drain_messages, enqueue, mk_msg, test_app, tick};
use crate::components::VulnerableStack;

// ── Behavior 90: empty stack is identity; no stack pass-through; despawned
//     target pass-through; multiple messages on empty stack identity ──

#[test]
fn empty_stack_is_identity() {
    let mut app = test_app();
    let target = app.world_mut().spawn(VulnerableStack::default()).id();

    enqueue(&mut app, mk_msg(None, target, 7.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 7.0);
}

#[test]
fn target_without_stack_passes_through() {
    // Edge case 90a.
    let mut app = test_app();
    let target = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(None, target, 5.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 5.0);
}

#[test]
fn despawned_target_passes_through() {
    // Edge case 90b.
    let mut app = test_app();
    let target = app.world_mut().spawn_empty().id();
    app.world_mut().despawn(target);

    enqueue(&mut app, mk_msg(None, target, 9.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 9.0);
}

#[test]
fn multiple_messages_empty_stack_identity() {
    // Edge case 90c.
    let mut app = test_app();
    let target = app.world_mut().spawn(VulnerableStack::default()).id();

    enqueue(&mut app, mk_msg(None, target, 7.0));
    enqueue(&mut app, mk_msg(None, target, 3.5));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 2);
    assert_f32_eq(drained[0].amount, 7.0);
    assert_f32_eq(drained[1].amount, 3.5);
}
