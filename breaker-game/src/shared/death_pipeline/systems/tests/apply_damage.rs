//! Tests for `apply_damage<T>`.

use bevy::prelude::*;

use super::helpers::{
    PendingDamage, TestEntity, build_apply_damage_app, damage_msg, spawn_test_entity,
    spawn_test_entity_invulnerable,
};
use crate::shared::death_pipeline::{
    dead::Dead, hp::Hp, invulnerable::Invulnerable, killed_by::KilledBy,
};

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

// ── Group I: Without<Invulnerable> filter ───────────────────────────────────

/// I1: `apply_damage::<TestEntity>` skips entities that carry `Invulnerable`.
#[test]
fn apply_damage_skips_invulnerable_entity() {
    let mut app = build_apply_damage_app();
    let entity = spawn_test_entity_invulnerable(&mut app, 3.0);

    app.insert_resource(PendingDamage(vec![damage_msg(entity, 1.0, None)]));
    crate::shared::test_utils::tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 3.0).abs() < f32::EPSILON,
        "Invulnerable entity's Hp should remain 3.0, got {}",
        hp.current
    );

    let killed_by = app.world().get::<KilledBy>(entity).unwrap();
    assert_eq!(
        killed_by.dealer, None,
        "KilledBy.dealer should remain None because damage was filtered out"
    );

    // Marker still present — filter is non-destructive.
    assert!(
        app.world().get::<Invulnerable>(entity).is_some(),
        "Invulnerable marker should still be present after the filtered tick"
    );
}

/// I1 edge: overkill damage (1000.0) against an invulnerable entity still
/// leaves HP unchanged at 3.0.
#[test]
fn apply_damage_skips_invulnerable_entity_against_overkill() {
    let mut app = build_apply_damage_app();
    let entity = spawn_test_entity_invulnerable(&mut app, 3.0);

    app.insert_resource(PendingDamage(vec![damage_msg(entity, 1000.0, None)]));
    crate::shared::test_utils::tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 3.0).abs() < f32::EPSILON,
        "Invulnerable entity should absorb overkill damage, Hp should remain 3.0, got {}",
        hp.current
    );
}

/// I1 edge: a dealer is NOT recorded in `KilledBy` because the victim is
/// invulnerable.
#[test]
fn apply_damage_invulnerable_entity_does_not_record_dealer() {
    let mut app = build_apply_damage_app();
    let entity = spawn_test_entity_invulnerable(&mut app, 3.0);
    let dealer = app.world_mut().spawn_empty().id();

    app.insert_resource(PendingDamage(vec![damage_msg(
        entity,
        1000.0,
        Some(dealer),
    )]));
    crate::shared::test_utils::tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(entity).unwrap();
    assert_eq!(
        killed_by.dealer, None,
        "dealer should NOT be recorded because the victim is invulnerable"
    );
}

/// I2: `apply_damage::<TestEntity>` applies damage to entities WITHOUT
/// `Invulnerable` — positive assertion that the filter does not exclude
/// non-invulnerable entities.
#[test]
fn apply_damage_applies_to_non_invulnerable_entity() {
    let mut app = build_apply_damage_app();
    let entity = spawn_test_entity(&mut app, 3.0);

    app.insert_resource(PendingDamage(vec![damage_msg(entity, 1.0, None)]));
    crate::shared::test_utils::tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 2.0).abs() < f32::EPSILON,
        "non-invulnerable entity should take 1.0 damage (3.0 -> 2.0), got {}",
        hp.current
    );
}

/// I2 edge: dealer IS recorded in `KilledBy.dealer` when a non-invulnerable
/// entity takes the killing blow.
#[test]
fn apply_damage_non_invulnerable_entity_records_dealer_on_killing_blow() {
    let mut app = build_apply_damage_app();
    let entity = spawn_test_entity(&mut app, 3.0);
    let dealer = app.world_mut().spawn_empty().id();

    app.insert_resource(PendingDamage(vec![damage_msg(entity, 3.0, Some(dealer))]));
    crate::shared::test_utils::tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(entity).unwrap();
    assert_eq!(
        killed_by.dealer,
        Some(dealer),
        "dealer should be recorded on the killing blow for a non-invulnerable entity"
    );
}

/// I3: a mixed batch of invulnerable and vulnerable targets — only the
/// vulnerable entity takes damage, filter is applied per-entity.
#[test]
fn apply_damage_filter_splits_invulnerable_from_vulnerable() {
    let mut app = build_apply_damage_app();
    let invulnerable_entity = spawn_test_entity_invulnerable(&mut app, 3.0);
    let vulnerable_entity = spawn_test_entity(&mut app, 3.0);

    app.insert_resource(PendingDamage(vec![
        damage_msg(invulnerable_entity, 1.0, None),
        damage_msg(vulnerable_entity, 1.0, None),
    ]));
    crate::shared::test_utils::tick(&mut app);

    let inv_hp = app.world().get::<Hp>(invulnerable_entity).unwrap();
    assert!(
        (inv_hp.current - 3.0).abs() < f32::EPSILON,
        "invulnerable entity Hp should remain 3.0, got {}",
        inv_hp.current
    );

    let vul_hp = app.world().get::<Hp>(vulnerable_entity).unwrap();
    assert!(
        (vul_hp.current - 2.0).abs() < f32::EPSILON,
        "vulnerable entity Hp should be 2.0 after 1.0 damage, got {}",
        vul_hp.current
    );
}

/// I3 edge: order reversed in the `PendingDamage` vec — same outcome, filter
/// is order-independent.
#[test]
fn apply_damage_filter_order_independent_for_mixed_batch() {
    let mut app = build_apply_damage_app();
    let invulnerable_entity = spawn_test_entity_invulnerable(&mut app, 3.0);
    let vulnerable_entity = spawn_test_entity(&mut app, 3.0);

    app.insert_resource(PendingDamage(vec![
        damage_msg(vulnerable_entity, 1.0, None),
        damage_msg(invulnerable_entity, 1.0, None),
    ]));
    crate::shared::test_utils::tick(&mut app);

    assert!(
        (app.world().get::<Hp>(invulnerable_entity).unwrap().current - 3.0).abs() < f32::EPSILON,
        "invulnerable entity Hp should remain 3.0 regardless of message order"
    );
    assert!(
        (app.world().get::<Hp>(vulnerable_entity).unwrap().current - 2.0).abs() < f32::EPSILON,
        "vulnerable entity Hp should be 2.0 regardless of message order"
    );
}

/// I3 edge: multiple damage messages against the same invulnerable entity in
/// the same tick — still unchanged because the filter drops the entity from
/// the query entirely.
#[test]
fn apply_damage_multiple_messages_against_invulnerable_still_skipped() {
    let mut app = build_apply_damage_app();
    let entity = spawn_test_entity_invulnerable(&mut app, 3.0);

    app.insert_resource(PendingDamage(vec![
        damage_msg(entity, 1.0, None),
        damage_msg(entity, 1.0, None),
        damage_msg(entity, 1.0, None),
    ]));
    crate::shared::test_utils::tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 3.0).abs() < f32::EPSILON,
        "invulnerable entity should absorb ALL damage messages in the same tick, got {}",
        hp.current
    );
}
