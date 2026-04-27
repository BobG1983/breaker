use super::helpers::{assert_f32_eq, drain_messages, enqueue, mk_msg_full, test_app, tick};
use crate::{SourceId, components::DamageBoostStack};

// ── W2 Behavior 12: apply_damage_boosts reads msg.dealer, NOT attributed_to ──

#[test]
fn boost_lookup_ignores_attributed_to_when_dealer_none() {
    // attributed_to: Some(D2) with boost ×2.0, dealer: None → amount unchanged.
    let mut app = test_app();
    let d2 = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add(SourceId::from("src:d2"), 2.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg_full(None, Some(d2), 10.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    // apply_damage_boosts must read msg.dealer, not attributed_to — D2's
    // boost stack should NOT be consulted when dealer is None.
    assert_f32_eq(drained[0].amount, 10.0);
}

#[test]
fn boost_still_applied_via_dealer_when_attributed_to_none() {
    // dealer: Some(D2), attributed_to: None → amount × 2.0 (baseline sanity).
    let mut app = test_app();
    let d2 = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add(SourceId::from("src:d2"), 2.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg_full(Some(d2), None, 10.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 20.0);
}

#[test]
fn boost_from_attributed_to_invisible_when_dealer_has_no_stack() {
    // dealer: Some(D1) with NO stack, attributed_to: Some(D2) with ×2.0 boost
    // → amount unchanged (D2's stack is invisible to apply_damage_boosts).
    let mut app = test_app();
    let d1 = app.world_mut().spawn_empty().id();
    let d2 = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add(SourceId::from("src:d2"), 2.0);
            s
        })
        .id();

    enqueue(&mut app, mk_msg_full(Some(d1), Some(d2), 10.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 10.0);
}
