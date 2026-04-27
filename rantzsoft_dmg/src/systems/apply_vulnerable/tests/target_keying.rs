use super::helpers::{assert_f32_eq, drain_messages, enqueue, mk_msg, test_app, tick};
use crate::{
    SourceId,
    components::{DamageBoostStack, VulnerableStack},
};

// ── Behavior 91: dealer's boost stack does NOT apply here; target keys
//     this system ──

#[test]
fn dealer_damage_boost_stack_is_ignored() {
    let mut app = test_app();
    let dealer = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add(SourceId::from("src:a"), 10.0);
            s.add_one_shot(10.0);
            s
        })
        .id();
    let target = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), target, 3.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 3.0);

    // Dealer's stack untouched.
    let mut dealer_stack = app.world_mut().get_mut::<DamageBoostStack>(dealer).unwrap();
    assert_f32_eq(dealer_stack.aggregate_persistent(), 10.0);
    assert_f32_eq(dealer_stack.aggregate_and_consume_one_shots(), 10.0);
}

#[test]
fn target_persistent_only_multiplies_when_dealer_has_no_boost_stack() {
    // Edge case 91a.
    let mut app = test_app();
    let dealer = app.world_mut().spawn_empty().id();
    let target = app
        .world_mut()
        .spawn({
            let mut s = VulnerableStack::default();
            s.add(SourceId::from("src:a"), 10.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(Some(dealer), target, 3.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 30.0);
}

#[test]
fn target_one_shot_only_multiplies_when_dealer_has_no_boost_stack() {
    // Edge case 91b.
    let mut app = test_app();
    let dealer = app.world_mut().spawn_empty().id();
    let target = app
        .world_mut()
        .spawn({
            let mut s = VulnerableStack::default();
            s.add_one_shot(10.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg(Some(dealer), target, 3.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 30.0);
}
