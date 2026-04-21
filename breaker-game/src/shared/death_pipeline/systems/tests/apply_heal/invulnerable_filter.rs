//! Group E: Without<Invulnerable> filter (Behaviors 15-16).

use bevy::prelude::*;

use super::super::helpers::{
    PendingHeal, TestEntity, build_apply_heal_app, heal_msg, spawn_test_entity,
};
use crate::{prelude::*, shared::death_pipeline::heal_dealt::HealCap};

/// Behavior 15: heal on Invulnerable entity is a no-op (Max cap).
#[test]
fn heal_on_invulnerable_entity_is_noop_max_cap() {
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
            Invulnerable,
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Invulnerable entity Hp should remain 5.0, got {}",
        hp.current
    );
}

/// Behavior 15 edge: Starting cap also blocked on Invulnerable.
#[test]
fn heal_on_invulnerable_entity_is_noop_starting_cap() {
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
            Invulnerable,
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Starting)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Invulnerable entity Hp should remain 5.0 (Starting cap), got {}",
        hp.current
    );
}

/// Behavior 15 edge: Invulnerable marker still present after filtered tick.
#[test]
fn heal_on_invulnerable_entity_preserves_marker() {
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
            Invulnerable,
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app);

    assert!(
        app.world().get::<Invulnerable>(entity).is_some(),
        "Invulnerable marker should still be present after filtered tick"
    );
}

/// Behavior 15b: Dead + Invulnerable — both filters compose (still skipped).
#[test]
fn heal_on_dead_and_invulnerable_entity_is_noop_max_cap() {
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
            Dead,
            Invulnerable,
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Dead+Invulnerable entity should remain 5.0, got {}",
        hp.current
    );
}

/// Behavior 15b edge: Starting cap also blocked on Dead+Invulnerable.
#[test]
fn heal_on_dead_and_invulnerable_entity_is_noop_starting_cap() {
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
            Dead,
            Invulnerable,
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Starting)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Dead+Invulnerable entity with Starting cap should remain 5.0, got {}",
        hp.current
    );
}

/// Behavior 15b edge: both markers still present after filtered tick.
#[test]
fn heal_on_dead_and_invulnerable_entity_preserves_both_markers() {
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
            Dead,
            Invulnerable,
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app);

    assert!(
        app.world().get::<Dead>(entity).is_some(),
        "Dead marker should still be present after filtered tick"
    );
    assert!(
        app.world().get::<Invulnerable>(entity).is_some(),
        "Invulnerable marker should still be present after filtered tick"
    );
}

/// Behavior 16: mixed batch — Invulnerable A and vulnerable B.
#[test]
fn mixed_batch_invulnerable_vs_vulnerable() {
    let mut app = build_apply_heal_app();
    let entity_a = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Invulnerable,
        ))
        .id();
    let entity_b = spawn_test_entity(&mut app, 10.0);
    // Set B's current to 5.0 (spawn_test_entity creates current=starting=10.0)
    app.world_mut().get_mut::<Hp>(entity_b).unwrap().current = 5.0;

    app.insert_resource(PendingHeal(vec![
        heal_msg(entity_a, 3.0, HealCap::Max),
        heal_msg(entity_b, 3.0, HealCap::Max),
    ]));
    tick(&mut app);

    let hp_a = app.world().get::<Hp>(entity_a).unwrap();
    let hp_b = app.world().get::<Hp>(entity_b).unwrap();
    assert!(
        (hp_a.current - 5.0).abs() < f32::EPSILON,
        "Invulnerable A should remain 5.0, got {}",
        hp_a.current
    );
    assert!(
        (hp_b.current - 8.0).abs() < f32::EPSILON,
        "Vulnerable B should be 8.0 after 3.0 heal, got {}",
        hp_b.current
    );
}

/// Behavior 16 edge: reversed message order — same outcome.
#[test]
fn mixed_batch_invulnerable_vs_vulnerable_reversed_order() {
    let mut app = build_apply_heal_app();
    let entity_a = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Invulnerable,
        ))
        .id();
    let entity_b = spawn_test_entity(&mut app, 10.0);
    app.world_mut().get_mut::<Hp>(entity_b).unwrap().current = 5.0;

    app.insert_resource(PendingHeal(vec![
        heal_msg(entity_b, 3.0, HealCap::Max),
        heal_msg(entity_a, 3.0, HealCap::Max),
    ]));
    tick(&mut app);

    let hp_a = app.world().get::<Hp>(entity_a).unwrap();
    let hp_b = app.world().get::<Hp>(entity_b).unwrap();
    assert!(
        (hp_a.current - 5.0).abs() < f32::EPSILON,
        "Invulnerable A should remain 5.0 (order-independent), got {}",
        hp_a.current
    );
    assert!(
        (hp_b.current - 8.0).abs() < f32::EPSILON,
        "Vulnerable B should be 8.0 (order-independent), got {}",
        hp_b.current
    );
}

/// Behavior 16 edge: different caps per entity.
#[test]
fn mixed_batch_invulnerable_starting_vulnerable_max() {
    let mut app = build_apply_heal_app();
    let entity_a = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Invulnerable,
        ))
        .id();
    let entity_b = spawn_test_entity(&mut app, 10.0);
    app.world_mut().get_mut::<Hp>(entity_b).unwrap().current = 5.0;

    app.insert_resource(PendingHeal(vec![
        heal_msg(entity_a, 3.0, HealCap::Starting),
        heal_msg(entity_b, 3.0, HealCap::Max),
    ]));
    tick(&mut app);

    let hp_a = app.world().get::<Hp>(entity_a).unwrap();
    let hp_b = app.world().get::<Hp>(entity_b).unwrap();
    assert!(
        (hp_a.current - 5.0).abs() < f32::EPSILON,
        "Invulnerable A filtered before cap considered, got {}",
        hp_a.current
    );
    assert!(
        (hp_b.current - 8.0).abs() < f32::EPSILON,
        "Vulnerable B should be 8.0, got {}",
        hp_b.current
    );
}
