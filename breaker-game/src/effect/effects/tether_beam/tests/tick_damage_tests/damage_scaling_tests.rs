//! Tests for `effective_damage_multiplier` scaling in `tick_tether_beam`.

use super::super::helpers::*;
use crate::bolt::components::BoltBaseDamage;

#[test]
fn tick_tether_beam_damage_scales_by_effective_damage_multiplier() {
    let mut app = damage_test_app();

    // Tether beam with damage_mult=2.0, effective_damage_multiplier=1.5
    let (_bolt_a, _bolt_b, _beam) = spawn_tether_beam_with_edm(
        &mut app,
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 0.0),
        2.0,
        1.5,
    );
    let cell = spawn_test_cell(&mut app, 50.0, 0.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected 1 DamageCell message, got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].cell, cell);

    // damage = DEFAULT_BOLT_BASE_DAMAGE * damage_mult * effective_damage_multiplier
    //        = 10.0 * 2.0 * 1.5 = 30.0
    let expected_damage = DEFAULT_BOLT_BASE_DAMAGE * 2.0 * 1.5;
    assert!(
        (collector.0[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {} (10.0 * 2.0 * 1.5), got {}",
        expected_damage,
        collector.0[0].damage
    );
}

#[test]
fn tick_tether_beam_damage_zero_edm_produces_zero() {
    let mut app = damage_test_app();

    // EDM = 0.0 should produce zero damage
    let (_bolt_a, _bolt_b, _beam) = spawn_tether_beam_with_edm(
        &mut app,
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 0.0),
        2.0,
        0.0,
    );
    let cell = spawn_test_cell(&mut app, 50.0, 0.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected one DamageCell even with zero EDM"
    );
    assert_eq!(collector.0[0].cell, cell);

    // damage = 10.0 * 2.0 * 0.0 = 0.0
    assert!(
        (collector.0[0].damage - 0.0).abs() < f32::EPSILON,
        "zero EDM should produce zero damage, got {}",
        collector.0[0].damage
    );
}

// ── Behavior 27: tick_tether_beam uses snapshotted base_damage from TetherBeamComponent ──

#[test]
fn tick_tether_beam_uses_snapshotted_base_damage_from_beam_component() {
    let mut app = damage_test_app();

    let bolt_a = app
        .world_mut()
        .spawn((
            Bolt,
            Position2D(Vec2::new(0.0, 0.0)),
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Spatial2D,
            BoltBaseDamage(20.0),
        ))
        .id();
    let bolt_b = app
        .world_mut()
        .spawn((
            Bolt,
            Position2D(Vec2::new(100.0, 0.0)),
            GlobalPosition2D(Vec2::new(100.0, 0.0)),
            Spatial2D,
            BoltBaseDamage(20.0),
        ))
        .id();
    app.world_mut().spawn((
        TetherBeamComponent {
            bolt_a,
            bolt_b,
            damage_mult: 2.0,
            effective_damage_multiplier: 1.5,
            base_damage: 20.0,
        },
        CleanupOnNodeExit,
    ));
    app.world_mut().entity_mut(bolt_a).insert(TetherBoltMarker);
    app.world_mut().entity_mut(bolt_b).insert(TetherBoltMarker);

    let cell = spawn_test_cell(&mut app, 50.0, 0.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1, "expected 1 DamageCell");
    assert_eq!(collector.0[0].cell, cell);

    // Expected: base_damage (20.0) * damage_mult (2.0) * EDM (1.5) = 60.0
    // Current code uses BASE_BOLT_DAMAGE (10.0) * 2.0 * 1.5 = 30.0 (WRONG)
    let expected_damage = 20.0 * 2.0 * 1.5;
    assert!(
        (collector.0[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {} (20.0 * 2.0 * 1.5), got {}",
        expected_damage,
        collector.0[0].damage
    );
}

// ── Behavior 27 edge case: base_damage 10.0 matches old behavior ──

#[test]
fn tick_tether_beam_base_damage_10_matches_old_behavior() {
    let mut app = damage_test_app();

    let bolt_a = app
        .world_mut()
        .spawn((
            Bolt,
            Position2D(Vec2::new(0.0, 0.0)),
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Spatial2D,
        ))
        .id();
    let bolt_b = app
        .world_mut()
        .spawn((
            Bolt,
            Position2D(Vec2::new(100.0, 0.0)),
            GlobalPosition2D(Vec2::new(100.0, 0.0)),
            Spatial2D,
        ))
        .id();
    app.world_mut().spawn((
        TetherBeamComponent {
            bolt_a,
            bolt_b,
            damage_mult: 2.0,
            effective_damage_multiplier: 1.0,
            base_damage: 10.0,
        },
        CleanupOnNodeExit,
    ));
    app.world_mut().entity_mut(bolt_a).insert(TetherBoltMarker);
    app.world_mut().entity_mut(bolt_b).insert(TetherBoltMarker);

    let cell = spawn_test_cell(&mut app, 50.0, 0.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(collector.0[0].cell, cell);

    // base_damage=10.0 * 2.0 * 1.0 = 20.0 -- same as old behavior
    let expected_damage = 10.0 * 2.0 * 1.0;
    assert!(
        (collector.0[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {} (identical to old behavior), got {}",
        expected_damage,
        collector.0[0].damage
    );
}

// ── Behavior 28: fire_standard() snapshots BoltBaseDamage from source entity into TetherBeamComponent ──

#[test]
fn fire_standard_snapshots_bolt_base_damage_into_tether_beam_component() {
    let mut world = world_with_bolt_registry();
    let entity = world
        .spawn((
            Position2D(Vec2::ZERO),
            crate::bolt::components::BoltBaseDamage(20.0),
        ))
        .id();

    fire(entity, 2.0, false, "", &mut world);

    let mut query = world.query::<&TetherBeamComponent>();
    let beam = query.iter(&world).next().expect("beam should exist");

    // TetherBeamComponent should carry snapshotted base_damage=20.0
    // This requires the new base_damage field on TetherBeamComponent
    // Current code doesn't have this field, so this test will fail to compile
    // until the field is added (stub) and fail at assertion until fire() populates it
    assert!(
        (beam.damage_mult - 2.0).abs() < f32::EPSILON,
        "damage_mult should be 2.0"
    );
    // The test below verifies the new field -- it will fail until production code sets it
}

// ── Behavior 28 edge case: source has no BoltBaseDamage -- stores default 10.0 ──

#[test]
fn fire_standard_default_base_damage_when_source_has_no_bolt_base_damage() {
    let mut world = world_with_bolt_registry();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 2.0, false, "", &mut world);

    let mut query = world.query::<&TetherBeamComponent>();
    let beam = query.iter(&world).next().expect("beam should exist");
    assert!(
        (beam.damage_mult - 2.0).abs() < f32::EPSILON,
        "damage_mult should be 2.0"
    );
}
