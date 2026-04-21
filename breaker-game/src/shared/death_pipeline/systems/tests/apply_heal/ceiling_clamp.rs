//! Group B: Ceiling clamping per `HealCap` variant (Behaviors 4-9).

use bevy::prelude::*;

use super::super::helpers::{PendingHeal, TestEntity, build_apply_heal_app, heal_msg};
use crate::{prelude::*, shared::death_pipeline::heal_dealt::HealCap};

/// Behavior 4: `HealCap::Max` clamps at Hp.max when Some.
#[test]
fn heal_max_cap_clamps_at_hp_max_when_some() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  18.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 5.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "Hp should clamp at max=20.0, got {}",
        hp.current
    );
}

/// Behavior 4 edge: already at max, heal is a no-op.
#[test]
fn heal_max_cap_at_max_is_noop() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  20.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "Hp should remain 20.0 (no-op at ceiling), got {}",
        hp.current
    );
}

/// Behavior 5: `HealCap::Max` falls back to starting with huge amount.
#[test]
fn heal_max_cap_huge_amount_falls_back_to_starting() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  8.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 500.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Hp should clamp at starting (10.0) — NOT unbounded, got {}",
        hp.current
    );
    assert!(
        hp.current.is_finite(),
        "Hp should not be infinite, got {}",
        hp.current
    );
}

/// Behavior 5 edge: 1e9 heal on tiny current, max=None.
#[test]
fn heal_max_cap_1e9_amount_clamps_at_starting() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  0.5,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 1.0e9, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Hp should clamp at starting (10.0), got {}",
        hp.current
    );
}

/// Behavior 6: `HealCap::Max` with max below starting — ceiling honoured verbatim.
#[test]
fn heal_max_cap_with_max_below_starting_honours_max() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  3.0,
                starting: 10.0,
                max:      Some(5.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 100.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Hp should clamp at max=5.0 (even though starting=10.0), got {}",
        hp.current
    );
}

/// Behavior 6 edge: at max=5.0 already — no-op.
#[test]
fn heal_max_cap_at_depressed_max_is_noop() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      Some(5.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 1.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Hp should remain 5.0 (already at max), got {}",
        hp.current
    );
}

/// Behavior 7: `HealCap::Starting` clamps at starting even when max > starting.
#[test]
fn heal_starting_cap_clamps_at_starting_ignoring_higher_max() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  8.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(
        entity,
        500.0,
        HealCap::Starting,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Starting cap must IGNORE max and clamp at starting=10.0, got {}",
        hp.current
    );
}

/// Behavior 7 edge: current at starting — Starting cap no-op.
#[test]
fn heal_starting_cap_at_starting_is_noop() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  10.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 5.0, HealCap::Starting)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Starting cap should no-op at starting, got {}",
        hp.current
    );
}

/// Behavior 7 edge: current 12.0 over-cap pre-state — Starting cap never lowers HP.
#[test]
fn heal_starting_cap_over_starting_never_lowers_hp() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  12.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 5.0, HealCap::Starting)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 12.0).abs() < f32::EPSILON,
        "Starting cap must NEVER lower HP, got {}",
        hp.current
    );
}

/// Behavior 8: Starting cap with max=None behaves identically to Max cap.
#[test]
fn heal_starting_cap_equals_max_cap_when_max_none() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  8.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(
        entity,
        500.0,
        HealCap::Starting,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Starting cap with max=None should clamp at starting=10.0, got {}",
        hp.current
    );
}

/// Behavior 8 edge: small additive heal with Starting cap (identical to Max).
#[test]
fn heal_starting_cap_additive_when_under_ceiling() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  2.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Starting)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Starting cap + 3.0 on current=2.0 = 5.0, got {}",
        hp.current
    );
}

/// Behavior 8 paired: Starting and Max caps give identical results when max=None.
#[test]
fn heal_cap_variants_identical_when_max_none() {
    // Run 1: HealCap::Starting
    let mut app_a = build_apply_heal_app();
    let entity_a = app_a
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  8.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();
    app_a.insert_resource(PendingHeal(vec![heal_msg(
        entity_a,
        500.0,
        HealCap::Starting,
    )]));
    tick(&mut app_a);
    let hp_a = app_a.world().get::<Hp>(entity_a).unwrap().current;

    // Run 2: HealCap::Max
    let mut app_b = build_apply_heal_app();
    let entity_b = app_b
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  8.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();
    app_b.insert_resource(PendingHeal(vec![heal_msg(entity_b, 500.0, HealCap::Max)]));
    tick(&mut app_b);
    let hp_b = app_b.world().get::<Hp>(entity_b).unwrap().current;

    assert!(
        (hp_a - 10.0).abs() < f32::EPSILON,
        "Starting cap should produce 10.0, got {hp_a}"
    );
    assert!(
        (hp_b - 10.0).abs() < f32::EPSILON,
        "Max cap should produce 10.0, got {hp_b}"
    );
    assert!(
        (hp_a - hp_b).abs() < f32::EPSILON,
        "Both caps should produce identical results when max=None, got {hp_a} vs {hp_b}"
    );
}

/// Behavior 9: over-cap pre-state — Max cap never lowers HP.
#[test]
fn heal_max_cap_over_cap_pre_state_never_lowers_hp() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  25.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 5.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 25.0).abs() < f32::EPSILON,
        "Max cap must never lower over-cap HP, got {}",
        hp.current
    );
}

/// Behavior 9 edge: same over-cap — Starting cap also never lowers HP.
#[test]
fn heal_starting_cap_over_cap_pre_state_never_lowers_hp() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  25.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 5.0, HealCap::Starting)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 25.0).abs() < f32::EPSILON,
        "Starting cap must never lower over-cap HP, got {}",
        hp.current
    );
}
