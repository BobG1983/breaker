//! System-level coverage for the `msg.source.as_ref()` wiring in
//! `apply_damage_boosts`. These tests guard against regressions where the
//! emission source is no longer threaded into the aggregate calls — a defect
//! that would silently disable every filtered entry in production.
//!
//! Component-level filter logic is exercised exhaustively in
//! `damage_boost_stack/tests/`. These tests focus only on the path from
//! `DamageDealt.source` → `emission_source` → both aggregate methods.

use super::helpers::{assert_f32_eq, drain_messages, enqueue, mk_msg_with_source, test_app, tick};
use crate::{SourceId, components::DamageBoostStack};

#[test]
fn filtered_persistent_entry_applies_when_emission_source_matches() {
    let mut app = test_app();
    let dealer = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add_filtered(SourceId::from("src:a"), 2.0, SourceId::from("emit:burnout"));
            s
        })
        .id();

    enqueue(
        &mut app,
        mk_msg_with_source(Some(dealer), Some(SourceId::from("emit:burnout")), 10.0),
    );
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 20.0);
}

#[test]
fn filtered_persistent_entry_skipped_when_emission_source_mismatches() {
    let mut app = test_app();
    let dealer = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add_filtered(SourceId::from("src:a"), 2.0, SourceId::from("emit:burnout"));
            s
        })
        .id();

    enqueue(
        &mut app,
        mk_msg_with_source(Some(dealer), Some(SourceId::from("emit:lightning")), 10.0),
    );
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 10.0);
}

#[test]
fn filtered_persistent_entry_skipped_when_emission_source_is_none() {
    let mut app = test_app();
    let dealer = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add_filtered(SourceId::from("src:a"), 2.0, SourceId::from("emit:burnout"));
            s
        })
        .id();

    enqueue(&mut app, mk_msg_with_source(Some(dealer), None, 10.0));
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 10.0);
}

#[test]
fn filtered_one_shot_consumed_only_when_emission_source_matches() {
    let mut app = test_app();
    let dealer = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add_one_shot_filtered(3.0, SourceId::from("emit:burnout"));
            s
        })
        .id();

    // Mismatched emission: filtered one-shot is preserved on the lane.
    enqueue(
        &mut app,
        mk_msg_with_source(Some(dealer), Some(SourceId::from("emit:lightning")), 10.0),
    );
    tick(&mut app);
    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 10.0);

    let stack = app.world().get::<DamageBoostStack>(dealer).unwrap();
    assert_f32_eq(
        stack.aggregate_one_shots(Some(&SourceId::from("emit:burnout"))),
        3.0,
    );

    // Matching emission: one-shot fires and drains.
    enqueue(
        &mut app,
        mk_msg_with_source(Some(dealer), Some(SourceId::from("emit:burnout")), 10.0),
    );
    tick(&mut app);
    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 30.0);

    let stack = app.world().get::<DamageBoostStack>(dealer).unwrap();
    assert!(stack.is_empty());
}

#[test]
fn filterless_one_shot_drains_for_any_emission_source() {
    let mut app = test_app();
    let dealer = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add_one_shot(2.0);
            s
        })
        .id();

    enqueue(
        &mut app,
        mk_msg_with_source(Some(dealer), Some(SourceId::from("emit:any")), 5.0),
    );
    tick(&mut app);

    let drained = drain_messages(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].amount, 10.0);

    let stack = app.world().get::<DamageBoostStack>(dealer).unwrap();
    assert!(stack.is_empty());
}
