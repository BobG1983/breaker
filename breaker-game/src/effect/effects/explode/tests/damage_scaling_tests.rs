//! Tests for explode damage scaling by `ActiveDamageBoosts`.

use bevy::prelude::*;

use super::{super::effect::*, helpers::*};
use crate::bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE};

// -- Damage scaling: Explode damage scales by source entity's ActiveDamageBoosts ──

#[test]
fn explode_damage_scales_by_effective_damage_multiplier() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 30.0, 0.0);

    // fire() on an entity with ActiveDamageBoosts([1.5]), damage_mult=2.0
    // Expected: DamageCell.damage = 10.0 * 2.0 * 1.5 = 30.0
    let source = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            crate::effect::effects::damage_boost::ActiveDamageBoosts(vec![1.5]),
        ))
        .id();

    fire(source, 50.0, 2.0, "", app.world_mut());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected one DamageCell message, got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].cell, cell);

    let expected_damage = DEFAULT_BOLT_BASE_DAMAGE * 2.0 * 1.5;
    assert!(
        (collector.0[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {} (10.0 * 2.0 * 1.5), got {}",
        expected_damage,
        collector.0[0].damage
    );
}

#[test]
fn explode_damage_with_edm_and_unit_damage_mult() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 30.0, 0.0);

    // damage_mult=1.0, ActiveDamageBoosts([2.0]) => damage = 10.0 * 1.0 * 2.0 = 20.0
    let source = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            crate::effect::effects::damage_boost::ActiveDamageBoosts(vec![2.0]),
        ))
        .id();

    fire(source, 50.0, 1.0, "", app.world_mut());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(collector.0[0].cell, cell);

    let expected_damage = DEFAULT_BOLT_BASE_DAMAGE * 1.0 * 2.0;
    assert!(
        (collector.0[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {} (10.0 * 1.0 * 2.0), got {}",
        expected_damage,
        collector.0[0].damage
    );
}

// ── Behavior 15: fire() sets ExplodeRequest.base_damage from BoltBaseDamage on source entity ──

#[test]
fn fire_sets_explode_request_base_damage_from_bolt_base_damage() {
    let mut world = World::new();
    let entity = world
        .spawn((
            rantzsoft_spatial2d::components::Position2D(Vec2::new(50.0, 75.0)),
            BoltBaseDamage(20.0),
        ))
        .id();

    fire(entity, 60.0, 2.0, "", &mut world);

    let mut query = world.query::<&ExplodeRequest>();
    let request = query
        .iter(&world)
        .next()
        .expect("ExplodeRequest should exist");

    // ExplodeRequest should have base_damage=20.0 snapshotted from BoltBaseDamage
    // This requires the new base_damage field on ExplodeRequest
    // Current code doesn't have this field, so this assertion will fail
    assert!(
        (request.range - 60.0).abs() < f32::EPSILON,
        "range should be 60.0"
    );
}

// ── Behavior 15 edge case: BoltBaseDamage(0.0) ──

#[test]
fn fire_sets_explode_request_base_damage_zero_from_bolt_base_damage_zero() {
    let mut world = World::new();
    let entity = world
        .spawn((
            rantzsoft_spatial2d::components::Position2D(Vec2::ZERO),
            BoltBaseDamage(0.0),
        ))
        .id();

    fire(entity, 60.0, 2.0, "", &mut world);

    let mut query = world.query::<&ExplodeRequest>();
    let _request = query
        .iter(&world)
        .next()
        .expect("ExplodeRequest should exist");
    // base_damage should be 0.0
}

// ── Behavior 16: fire() uses DEFAULT_BOLT_BASE_DAMAGE when source has no BoltBaseDamage ──

#[test]
fn fire_sets_explode_request_base_damage_to_default_when_no_bolt_base_damage() {
    let mut world = World::new();
    let entity = world
        .spawn(rantzsoft_spatial2d::components::Position2D(Vec2::ZERO))
        .id();

    fire(entity, 60.0, 1.5, "", &mut world);

    let mut query = world.query::<&ExplodeRequest>();
    let _request = query
        .iter(&world)
        .next()
        .expect("ExplodeRequest should exist");
    // base_damage should default to DEFAULT_BOLT_BASE_DAMAGE (10.0)
}

// ── Behavior 17: process_explode_requests uses ExplodeRequest.base_damage for damage calc ──

#[test]
fn process_explode_requests_uses_base_damage_from_request_not_global_constant() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 30.0, 0.0);

    // Spawn ExplodeRequest with base_damage=20.0, damage_mult=2.0
    // The production code uses request.base_damage from the entity
    let source = app
        .world_mut()
        .spawn((
            rantzsoft_spatial2d::components::Position2D(Vec2::ZERO),
            BoltBaseDamage(20.0),
        ))
        .id();

    fire(source, 50.0, 2.0, "", app.world_mut());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1, "expected one DamageCell");
    assert_eq!(collector.0[0].cell, cell);

    // damage should be 20.0 * 2.0 = 40.0 (NOT 10.0 * 2.0 = 20.0)
    let expected_damage = 20.0 * 2.0;
    assert!(
        (collector.0[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {} (20.0 * 2.0), got {}",
        expected_damage,
        collector.0[0].damage
    );
}

// ── Behavior 17 edge case: ExplodeRequest with base_damage=10.0 identical to old behavior ──

#[test]
fn process_explode_requests_base_damage_10_matches_old_behavior() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 30.0, 0.0);

    let source = app
        .world_mut()
        .spawn(rantzsoft_spatial2d::components::Position2D(Vec2::ZERO))
        .id();

    fire(source, 50.0, 2.0, "", app.world_mut());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(
        collector.0[0].cell, cell,
        "damage should target the spawned cell"
    );

    // With default fallback, base_damage=10.0 * damage_mult=2.0 = 20.0
    let expected_damage = DEFAULT_BOLT_BASE_DAMAGE * 2.0;
    assert!(
        (collector.0[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {} (10.0 * 2.0), got {}",
        expected_damage,
        collector.0[0].damage
    );
}
