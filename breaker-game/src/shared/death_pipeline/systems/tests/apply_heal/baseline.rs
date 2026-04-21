//! Group A: Baseline heal semantics (Behaviors 1-3).

use bevy::prelude::*;

use super::super::helpers::{PendingHeal, TestEntity, build_apply_heal_app, heal_msg};
use crate::{prelude::*, shared::death_pipeline::heal_dealt::HealCap};

/// Behavior 1: heal increases Hp.current by amount.
#[test]
fn heal_increases_hp_current_by_amount() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 8.0).abs() < f32::EPSILON,
        "Hp should be 8.0 after 3.0 heal on 5.0-HP entity, got {}",
        hp.current
    );
    assert!(
        (hp.starting - 10.0).abs() < f32::EPSILON,
        "starting should remain 10.0, got {}",
        hp.starting
    );
    assert!(hp.max.is_none(), "max should remain None, got {:?}", hp.max);
}

/// Behavior 1 edge: heal that lands exactly on the ceiling.
#[test]
fn heal_lands_exactly_on_ceiling_starting_fallback() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  7.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Hp should be 10.0 (exactly at ceiling) after 7.0+3.0, got {}",
        hp.current
    );
}

/// Behavior 2: multiple heals in a single tick accumulate (same cap variant).
#[test]
fn multiple_heals_same_tick_accumulate_same_cap() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 30.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![
        heal_msg(entity, 4.0, HealCap::Max),
        heal_msg(entity, 6.0, HealCap::Max),
    ]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "Hp should be 15.0 after 5 + 4 + 6, got {}",
        hp.current
    );
}

/// Behavior 2 edge: three heals reach ceiling (clamp engages across messages).
#[test]
fn multiple_heals_clamp_engages_across_messages() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  27.0,
                starting: 30.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![
        heal_msg(entity, 2.0, HealCap::Max),
        heal_msg(entity, 3.0, HealCap::Max),
        heal_msg(entity, 4.0, HealCap::Max),
    ]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 30.0).abs() < f32::EPSILON,
        "Hp should clamp at 30.0 (ceiling), got {}",
        hp.current
    );
}

/// Behavior 3: max = None + `HealCap::Max` falls back to starting.
#[test]
fn heal_max_cap_falls_back_to_starting_when_max_is_none() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  9.5,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 5.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Hp should clamp at starting (10.0) when max is None, got {}",
        hp.current
    );
}

/// Behavior 3 edge: already at starting with max=None — no overflow past.
#[test]
fn heal_at_starting_no_overflow_when_max_none() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  10.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 1.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Hp should remain 10.0 (no overflow), got {}",
        hp.current
    );
}
