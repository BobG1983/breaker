//! Group C: Mixed-cap same-tick semantics (Behaviors 10-11).

use bevy::prelude::*;

use super::super::helpers::{PendingHeal, TestEntity, build_apply_heal_app, heal_msg};
use crate::{prelude::*, shared::death_pipeline::heal_dealt::HealCap};

/// Behavior 10: Starting then Max — second message's higher ceiling extends.
#[test]
fn mixed_cap_starting_then_max_second_ceiling_extends() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    // Step-by-step reasoning:
    // msg 1 (Starting, amount=100): current = (5 + 100).min(10) = 10
    // msg 2 (Max, amount=100):      current = (10 + 100).min(20) = 20
    app.insert_resource(PendingHeal(vec![
        heal_msg(entity, 100.0, HealCap::Starting),
        heal_msg(entity, 100.0, HealCap::Max),
    ]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "Mixed Starting->Max should reach max=20.0, got {}",
        hp.current
    );
}

/// Behavior 10 edge: reversed order (Max then Starting) — Starting is no-op.
#[test]
fn mixed_cap_max_then_starting_second_message_is_noop() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    // msg 1 (Max, amount=100):      current = (5 + 100).min(20) = 20
    // msg 2 (Starting, amount=100): current=20 >= starting=10, no-op (Behavior 9)
    app.insert_resource(PendingHeal(vec![
        heal_msg(entity, 100.0, HealCap::Max),
        heal_msg(entity, 100.0, HealCap::Starting),
    ]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "Mixed Max->Starting should stay at 20.0, got {}",
        hp.current
    );
}

/// Behavior 10 edge: small amounts, neither ceiling hit — simple accumulation.
#[test]
fn mixed_cap_small_additive_both_under_ceiling() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    // msg 1 (Starting, 3.0): current = (5 + 3).min(10) = 8
    // msg 2 (Max, 3.0):      current = (8 + 3).min(20) = 11
    app.insert_resource(PendingHeal(vec![
        heal_msg(entity, 3.0, HealCap::Starting),
        heal_msg(entity, 3.0, HealCap::Max),
    ]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 11.0).abs() < f32::EPSILON,
        "Mixed small-amount additive should give 11.0, got {}",
        hp.current
    );
}

/// Behavior 10b: per-message ceiling re-read — four-message sequence lands at 13.0.
#[test]
fn per_message_ceiling_re_read_not_cached() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  2.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    // Step-by-step:
    // msg 1 (Starting, 3): current = (2 + 3).min(10) = 5
    // msg 2 (Max, 3):      current = (5 + 3).min(20) = 8
    // msg 3 (Starting, 3): current = (8 + 3).min(10) = 10
    // msg 4 (Max, 3):      current = (10 + 3).min(20) = 13
    app.insert_resource(PendingHeal(vec![
        heal_msg(entity, 3.0, HealCap::Starting),
        heal_msg(entity, 3.0, HealCap::Max),
        heal_msg(entity, 3.0, HealCap::Starting),
        heal_msg(entity, 3.0, HealCap::Max),
    ]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 13.0).abs() < f32::EPSILON,
        "Per-message ceiling re-read should produce 13.0; cached would give 10.0, got {}",
        hp.current
    );
}

/// Behavior 10b edge: swap messages 3 and 4 — different result (11.0).
#[test]
fn per_message_ceiling_re_read_order_matters() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  2.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    // Step-by-step:
    // msg 1 (Starting, 3): current = (2 + 3).min(10) = 5
    // msg 2 (Max, 3):      current = (5 + 3).min(20) = 8
    // msg 3 (Max, 3):      current = (8 + 3).min(20) = 11
    // msg 4 (Starting, 3): current=11 >= starting=10, no-op
    app.insert_resource(PendingHeal(vec![
        heal_msg(entity, 3.0, HealCap::Starting),
        heal_msg(entity, 3.0, HealCap::Max),
        heal_msg(entity, 3.0, HealCap::Max),
        heal_msg(entity, 3.0, HealCap::Starting),
    ]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 11.0).abs() < f32::EPSILON,
        "Swapped order should produce 11.0 (different from 13.0), got {}",
        hp.current
    );
}

/// Behavior 11: both mixed heals fit under lower ceiling — simple accumulation.
#[test]
fn mixed_cap_both_under_lower_ceiling_accumulate() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  2.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    // msg 1 (Starting, 3): current = (2 + 3).min(10) = 5
    // msg 2 (Max, 2):      current = (5 + 2).min(20) = 7
    app.insert_resource(PendingHeal(vec![
        heal_msg(entity, 3.0, HealCap::Starting),
        heal_msg(entity, 2.0, HealCap::Max),
    ]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Mixed non-binding clamps should accumulate to 7.0, got {}",
        hp.current
    );
}

/// Behavior 11 edge: reversed order — still 7.0.
#[test]
fn mixed_cap_both_under_lower_ceiling_reversed_order() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  2.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    // msg 1 (Max, 2):      current = (2 + 2).min(20) = 4
    // msg 2 (Starting, 3): current = (4 + 3).min(10) = 7
    app.insert_resource(PendingHeal(vec![
        heal_msg(entity, 2.0, HealCap::Max),
        heal_msg(entity, 3.0, HealCap::Starting),
    ]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Reversed mixed additive should still give 7.0, got {}",
        hp.current
    );
}
