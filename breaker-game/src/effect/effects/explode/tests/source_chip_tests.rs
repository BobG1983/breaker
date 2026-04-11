//! Tests for `EffectSourceChip` attribution on `ExplodeRequest` and
//! `Position2D`-based positioning for both `fire()` and `process_explode_requests`.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::{super::effect::*, helpers::*};
use crate::effect::core::EffectSourceChip;

// -- Section E: EffectSourceChip attribution tests ───────────────────

#[test]
fn fire_stores_effect_source_chip_with_non_empty_chip_name() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(50.0, 75.0, 0.0)).id();

    fire(entity, 60.0, 2.0, "blast", &mut world);

    let mut query = world.query::<&EffectSourceChip>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one entity with EffectSourceChip"
    );
    assert_eq!(
        results[0].0,
        Some("blast".to_string()),
        "spawned ExplodeRequest should have EffectSourceChip(Some(\"blast\"))"
    );
}

#[test]
fn fire_stores_effect_source_chip_none_with_empty_chip_name() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

    fire(entity, 60.0, 2.0, "", &mut world);

    let mut query = world.query::<&EffectSourceChip>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one entity with EffectSourceChip"
    );
    assert_eq!(
        results[0].0, None,
        "empty source_chip should produce EffectSourceChip(None)"
    );
}

#[test]
fn process_explode_requests_populates_source_chip_from_effect_source_chip() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 30.0, 0.0);

    app.world_mut().spawn((
        ExplodeRequest {
            range: 50.0,
            damage: 10.0,
        },
        EffectSourceChip(Some("blast".to_string())),
        Position2D(Vec2::new(0.0, 0.0)),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(collector.0[0].cell, cell);
    assert_eq!(
        collector.0[0].source_chip,
        Some("blast".to_string()),
        "DamageCell should have source_chip from EffectSourceChip"
    );
}

#[test]
fn process_explode_requests_source_chip_none_when_no_effect_source_chip() {
    let mut app = damage_test_app();

    spawn_test_cell(&mut app, 30.0, 0.0);

    // No EffectSourceChip on request
    app.world_mut().spawn((
        ExplodeRequest {
            range: 50.0,
            damage: 10.0,
        },
        Position2D(Vec2::new(0.0, 0.0)),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(
        collector.0[0].source_chip, None,
        "missing EffectSourceChip should default to source_chip None"
    );
}

#[test]
fn multiple_explode_requests_with_different_source_chips_produce_correctly_attributed_damage() {
    let mut app = damage_test_app();

    let cell_a = spawn_test_cell(&mut app, 10.0, 0.0);
    let cell_b = spawn_test_cell(&mut app, 210.0, 0.0);

    app.world_mut().spawn((
        ExplodeRequest {
            range: 50.0,
            damage: 10.0,
        },
        EffectSourceChip(Some("alpha".to_string())),
        Position2D(Vec2::new(0.0, 0.0)),
    ));

    app.world_mut().spawn((
        ExplodeRequest {
            range: 50.0,
            damage: 10.0,
        },
        EffectSourceChip(Some("beta".to_string())),
        Position2D(Vec2::new(200.0, 0.0)),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        2,
        "expected 2 DamageCell messages, got {}",
        collector.0.len()
    );

    let msg_a = collector.0.iter().find(|m| m.cell == cell_a).unwrap();
    assert_eq!(
        msg_a.source_chip,
        Some("alpha".to_string()),
        "cell near request A should have source_chip alpha"
    );

    let msg_b = collector.0.iter().find(|m| m.cell == cell_b).unwrap();
    assert_eq!(
        msg_b.source_chip,
        Some("beta".to_string()),
        "cell near request B should have source_chip beta"
    );
}

// ── Behavior: explode fire() reads Position2D for spawn position ──

#[test]
fn fire_reads_position2d_for_spawn_position() {
    let mut world = World::new();
    let entity = world.spawn(Position2D(Vec2::new(50.0, 75.0))).id();

    fire(entity, 60.0, 2.0, "", &mut world);

    let mut query = world.query::<(&ExplodeRequest, &Position2D)>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "expected exactly one ExplodeRequest entity"
    );

    let (request, pos) = results[0];
    assert!(
        (request.range - 60.0).abs() < f32::EPSILON,
        "expected range 60.0, got {}",
        request.range
    );
    assert!(
        (request.damage - 2.0).abs() < f32::EPSILON,
        "expected damage 2.0, got {}",
        request.damage
    );
    assert!(
        (pos.0.x - 50.0).abs() < f32::EPSILON,
        "expected Position2D x=50.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - 75.0).abs() < f32::EPSILON,
        "expected Position2D y=75.0, got {}",
        pos.0.y
    );
}

// ── Behavior: explode fire() uses Position2D not Transform when both present ──

#[test]
fn fire_uses_position2d_not_transform_when_both_present() {
    let mut world = World::new();
    // Position2D and Transform are intentionally divergent
    let entity = world
        .spawn((
            Position2D(Vec2::new(50.0, 75.0)),
            Transform::from_xyz(999.0, 888.0, 0.0),
        ))
        .id();

    fire(entity, 60.0, 2.0, "", &mut world);

    let mut query = world.query::<(&ExplodeRequest, &Position2D)>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "expected exactly one ExplodeRequest entity"
    );

    let (_request, pos) = results[0];
    assert!(
        (pos.0.x - 50.0).abs() < f32::EPSILON,
        "ExplodeRequest should use Position2D x=50.0, not Transform x=999.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - 75.0).abs() < f32::EPSILON,
        "ExplodeRequest should use Position2D y=75.0, not Transform y=888.0, got {}",
        pos.0.y
    );
}

// ── Behavior: explode fire() falls back to Vec2::ZERO when Position2D absent ──

#[test]
fn fire_falls_back_to_zero_when_position2d_absent() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    fire(entity, 60.0, 2.0, "", &mut world);

    let mut query = world.query::<(&ExplodeRequest, &Position2D)>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "request should still be spawned");

    let (_request, pos) = results[0];
    assert!(
        (pos.0.x).abs() < f32::EPSILON,
        "Position2D x should default to 0.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y).abs() < f32::EPSILON,
        "Position2D y should default to 0.0, got {}",
        pos.0.y
    );
}

// ── Behavior: process_explode_requests uses Position2D not Transform when both present ──

#[test]
fn process_explode_requests_uses_position2d_not_transform_when_both_present() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 30.0, 0.0);

    // ExplodeRequest at Position2D origin (0,0), but Transform at (500,500) — divergent.
    // If the system reads Position2D, cell at (30,0) is within range 50.
    // If the system incorrectly reads Transform, cell would be ~530 units away — outside range.
    app.world_mut().spawn((
        ExplodeRequest {
            range: 50.0,
            damage: 10.0,
        },
        Position2D(Vec2::new(0.0, 0.0)),
        Transform::from_xyz(500.0, 500.0, 0.0),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "cell at (30,0) should be within range 50 of Position2D (0,0), got {} messages",
        collector.0.len()
    );
    assert_eq!(
        collector.0[0].cell, cell,
        "DamageCell should target the cell within Position2D-based range"
    );
}
