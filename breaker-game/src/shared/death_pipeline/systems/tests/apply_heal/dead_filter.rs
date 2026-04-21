//! Group D: Without<Dead> filter (Behaviors 12-14).

use bevy::prelude::*;

use super::super::helpers::{
    PendingHeal, TestEntity, build_apply_heal_app, heal_msg, spawn_test_entity_dead,
};
use crate::{prelude::*, shared::death_pipeline::heal_dealt::HealCap};

/// Behavior 12: heal on Dead entity is a no-op (Max cap).
#[test]
fn heal_on_dead_entity_is_noop_max_cap() {
    let mut app = build_apply_heal_app();
    let entity = spawn_test_entity_dead(&mut app, 10.0);

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 7.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Dead entity Hp should remain 10.0, got {}",
        hp.current
    );
}

/// Behavior 12 edge: Dead entity with negative Hp — still not healed.
#[test]
fn heal_on_dead_negative_hp_entity_is_noop() {
    let mut app = build_apply_heal_app();
    let entity = spawn_test_entity_dead(&mut app, -5.0);

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 100.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - (-5.0)).abs() < f32::EPSILON,
        "Dead entity with negative Hp should remain -5.0, got {}",
        hp.current
    );
}

/// Behavior 12 edge: Starting cap also blocked on Dead entity.
#[test]
fn heal_on_dead_entity_is_noop_starting_cap() {
    let mut app = build_apply_heal_app();
    let entity = spawn_test_entity_dead(&mut app, -5.0);

    app.insert_resource(PendingHeal(vec![heal_msg(
        entity,
        100.0,
        HealCap::Starting,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - (-5.0)).abs() < f32::EPSILON,
        "Dead entity with Starting cap should remain -5.0, got {}",
        hp.current
    );
}

/// Behavior 12 edge: Dead marker still present after filtered tick.
#[test]
fn heal_on_dead_entity_does_not_remove_dead_marker() {
    let mut app = build_apply_heal_app();
    let entity = spawn_test_entity_dead(&mut app, 10.0);

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 7.0, HealCap::Max)]));
    tick(&mut app);

    assert!(
        app.world().get::<Dead>(entity).is_some(),
        "Dead marker should still be present after the filtered tick"
    );
}

/// Behavior 13: non-Dead entity with Hp.current == 0.0 IS healed (Dead is the gate).
#[test]
fn heal_applies_to_non_dead_entity_with_zero_hp() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  0.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 4.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 4.0).abs() < f32::EPSILON,
        "non-Dead entity with 0.0 Hp should be healed to 4.0, got {}",
        hp.current
    );
}

/// Behavior 14: heal on Dead entity does not set `KilledBy`.
#[test]
fn heal_on_dead_entity_does_not_set_killed_by() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  0.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Dead,
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 4.0, HealCap::Max)]));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(entity).unwrap();
    assert_eq!(
        killed_by.dealer, None,
        "KilledBy.dealer should remain None — heal must not touch it"
    );
}
