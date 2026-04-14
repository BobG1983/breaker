//! Tests for `apply_damage<T>`.

use bevy::prelude::*;

use super::helpers::{
    PendingDamage, TestEntity, build_apply_damage_app, damage_msg, spawn_test_entity,
};
use crate::shared::death_pipeline::{dead::Dead, hp::Hp, killed_by::KilledBy};

#[test]
fn apply_damage_reduces_hp() {
    let mut app = build_apply_damage_app();
    let entity = spawn_test_entity(&mut app, 30.0);

    app.insert_resource(PendingDamage(vec![damage_msg(entity, 10.0, None)]));
    crate::shared::test_utils::tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "Hp should be 20.0 after 10 damage to 30-HP entity, got {}",
        hp.current
    );
}

#[test]
fn apply_damage_inserts_dead_marker_is_not_its_job() {
    // apply_damage does NOT insert Dead — that's the domain kill handler's job.
    // This test confirms that after apply_damage reduces Hp to 0, Dead is NOT present.
    let mut app = build_apply_damage_app();
    let entity = spawn_test_entity(&mut app, 10.0);

    app.insert_resource(PendingDamage(vec![damage_msg(entity, 10.0, None)]));
    crate::shared::test_utils::tick(&mut app);

    assert!(
        app.world().get::<Dead>(entity).is_none(),
        "apply_damage should NOT insert Dead — that is the domain kill handler's job"
    );
}

#[test]
fn apply_damage_sets_killed_by_on_killing_blow() {
    let mut app = build_apply_damage_app();
    let entity = spawn_test_entity(&mut app, 10.0);
    let dealer = app.world_mut().spawn_empty().id();

    app.insert_resource(PendingDamage(vec![damage_msg(entity, 10.0, Some(dealer))]));
    crate::shared::test_utils::tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(entity).unwrap();
    assert_eq!(
        killed_by.dealer,
        Some(dealer),
        "KilledBy should record the dealer on the killing blow"
    );
}

#[test]
fn apply_damage_sets_killed_by_when_dealer_is_none() {
    // Environmental death: dealer is None, but it's still the killing blow.
    let mut app = build_apply_damage_app();
    let entity = spawn_test_entity(&mut app, 10.0);

    app.insert_resource(PendingDamage(vec![damage_msg(entity, 10.0, None)]));
    crate::shared::test_utils::tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(entity).unwrap();
    assert_eq!(
        killed_by.dealer, None,
        "KilledBy.dealer should remain None for environmental kills"
    );
}

#[test]
fn apply_damage_does_not_set_killed_by_when_hp_stays_positive() {
    let mut app = build_apply_damage_app();
    let entity = spawn_test_entity(&mut app, 30.0);
    let dealer = app.world_mut().spawn_empty().id();

    app.insert_resource(PendingDamage(vec![damage_msg(entity, 10.0, Some(dealer))]));
    crate::shared::test_utils::tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(entity).unwrap();
    assert_eq!(
        killed_by.dealer, None,
        "KilledBy should not be set when Hp is still positive"
    );
}

#[test]
fn apply_damage_skips_entity_already_dead() {
    let mut app = build_apply_damage_app();
    let entity = app
        .world_mut()
        .spawn((TestEntity, Hp::new(10.0), KilledBy::default(), Dead))
        .id();

    app.insert_resource(PendingDamage(vec![damage_msg(entity, 5.0, None)]));
    crate::shared::test_utils::tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Dead entity's Hp should remain unchanged at 10.0, got {}",
        hp.current
    );
}

#[test]
fn apply_damage_skips_entity_without_hp() {
    // Entity with TestEntity but no Hp — system should silently skip.
    let mut app = build_apply_damage_app();
    let entity = app.world_mut().spawn(TestEntity).id();

    app.insert_resource(PendingDamage(vec![damage_msg(entity, 5.0, None)]));
    // Should not panic
    crate::shared::test_utils::tick(&mut app);

    assert!(
        app.world().get::<Hp>(entity).is_none(),
        "Entity without Hp should remain without Hp"
    );
}

#[test]
fn apply_damage_first_kill_wins() {
    // Two damage messages in the same frame both cross Hp to <= 0.
    // First kill wins — KilledBy records the first dealer.
    let mut app = build_apply_damage_app();
    let entity = spawn_test_entity(&mut app, 10.0);
    let dealer_a = app.world_mut().spawn_empty().id();
    let dealer_b = app.world_mut().spawn_empty().id();

    app.insert_resource(PendingDamage(vec![
        damage_msg(entity, 10.0, Some(dealer_a)),
        damage_msg(entity, 5.0, Some(dealer_b)),
    ]));
    crate::shared::test_utils::tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(entity).unwrap();
    assert_eq!(
        killed_by.dealer,
        Some(dealer_a),
        "First kill should win — dealer_a dealt the killing blow"
    );
}

#[test]
fn apply_damage_multiple_messages_accumulate() {
    let mut app = build_apply_damage_app();
    let entity = spawn_test_entity(&mut app, 30.0);

    app.insert_resource(PendingDamage(vec![
        damage_msg(entity, 10.0, None),
        damage_msg(entity, 8.0, None),
    ]));
    crate::shared::test_utils::tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 12.0).abs() < f32::EPSILON,
        "Hp should be 12.0 after 10+8 damage to 30-HP entity, got {}",
        hp.current
    );
}

#[test]
fn apply_damage_overkill_sets_negative_hp() {
    let mut app = build_apply_damage_app();
    let entity = spawn_test_entity(&mut app, 10.0);

    app.insert_resource(PendingDamage(vec![damage_msg(entity, 25.0, None)]));
    crate::shared::test_utils::tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        hp.current < 0.0,
        "Hp should go negative on overkill, got {}",
        hp.current
    );
    assert!(
        (hp.current - (-15.0)).abs() < f32::EPSILON,
        "Hp should be -15.0 after 25 damage to 10-HP entity, got {}",
        hp.current
    );
}
