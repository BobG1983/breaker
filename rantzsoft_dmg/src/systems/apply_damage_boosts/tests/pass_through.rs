use super::helpers::{assert_f32_eq, drain_messages, enqueue, mk_msg, test_app, tick};
use crate::{SourceId, components::DamageBoostStack};

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
