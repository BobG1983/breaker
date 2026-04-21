//! Group G: Numeric edge cases on amount (Behaviors 21-23).

use bevy::prelude::*;

use super::super::helpers::{PendingHeal, TestEntity, build_apply_heal_app, heal_msg};
use crate::{prelude::*, shared::death_pipeline::heal_dealt::HealCap};

/// Behavior 21: heal with amount == 0.0 is a no-op (Max cap).
#[test]
fn heal_with_zero_amount_is_noop_max_cap() {
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

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 0.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Zero-amount heal should leave Hp unchanged at 7.0, got {}",
        hp.current
    );
}

/// Behavior 21 edge: zero amount, Starting cap.
#[test]
fn heal_with_zero_amount_is_noop_starting_cap() {
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

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 0.0, HealCap::Starting)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Zero-amount heal with Starting cap should leave Hp at 7.0, got {}",
        hp.current
    );
}

/// Behavior 22: heal with negative amount is a no-op (never damages).
#[test]
fn heal_with_negative_amount_is_noop_max_cap() {
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

    app.insert_resource(PendingHeal(vec![heal_msg(entity, -3.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Negative-amount heal must NOT damage — Hp should remain 7.0, got {}",
        hp.current
    );
}

/// Behavior 22 edge: large negative amount — still no-op.
#[test]
fn heal_with_large_negative_amount_is_noop() {
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

    app.insert_resource(PendingHeal(vec![heal_msg(entity, -1000.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Large-negative amount guard must be absolute, got {}",
        hp.current
    );
}

/// Behavior 22 edge: negative zero.
#[test]
fn heal_with_negative_zero_amount_is_noop() {
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

    app.insert_resource(PendingHeal(vec![heal_msg(entity, -0.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Negative-zero heal should be no-op, got {}",
        hp.current
    );
}

/// Behavior 22 edge: negative amount with Starting cap (guard is cap-agnostic).
#[test]
fn heal_with_negative_amount_is_noop_starting_cap() {
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

    app.insert_resource(PendingHeal(vec![heal_msg(entity, -3.0, HealCap::Starting)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Negative-amount heal with Starting cap must not damage, got {}",
        hp.current
    );
}

/// Behavior 23: NaN amount does not corrupt Hp.current (Max cap).
#[test]
fn heal_with_nan_amount_does_not_corrupt_hp_max_cap() {
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

    app.insert_resource(PendingHeal(vec![heal_msg(entity, f32::NAN, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        hp.current.is_finite(),
        "Hp should remain finite after NaN heal, got {}",
        hp.current
    );
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Hp should remain 7.0 after NaN heal, got {}",
        hp.current
    );
}

/// Behavior 23 edge: NaN with Starting cap.
#[test]
fn heal_with_nan_amount_does_not_corrupt_hp_starting_cap() {
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

    app.insert_resource(PendingHeal(vec![heal_msg(
        entity,
        f32::NAN,
        HealCap::Starting,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        hp.current.is_finite() && (hp.current - 7.0).abs() < f32::EPSILON,
        "Hp should remain finite and 7.0 after NaN heal (Starting cap), got {}",
        hp.current
    );
}

/// Behavior 23 edge: INFINITY + Max cap, max=None → clamps at starting.
#[test]
fn heal_with_infinity_max_cap_no_max_clamps_at_starting() {
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

    app.insert_resource(PendingHeal(vec![heal_msg(
        entity,
        f32::INFINITY,
        HealCap::Max,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "INFINITY + Max cap + max=None should clamp at starting=10.0, got {}",
        hp.current
    );
}

/// Behavior 23 edge: INFINITY + Starting cap with max=Some → clamps at starting.
#[test]
fn heal_with_infinity_starting_cap_ignores_max() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  7.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(
        entity,
        f32::INFINITY,
        HealCap::Starting,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "INFINITY + Starting cap should clamp at starting=10.0 ignoring max=20.0, got {}",
        hp.current
    );
}

/// Behavior 23 edge: INFINITY + Max cap with max=Some → clamps at max.
#[test]
fn heal_with_infinity_max_cap_with_max_clamps_at_max() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  7.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(
        entity,
        f32::INFINITY,
        HealCap::Max,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "INFINITY + Max cap with max=20.0 should clamp at 20.0, got {}",
        hp.current
    );
}

/// Behavior 23 edge: `NEG_INFINITY` is blocked by negative-amount guard.
#[test]
fn heal_with_neg_infinity_is_noop() {
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

    app.insert_resource(PendingHeal(vec![heal_msg(
        entity,
        f32::NEG_INFINITY,
        HealCap::Max,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "NEG_INFINITY should be blocked by <= 0.0 guard, got {}",
        hp.current
    );
}
