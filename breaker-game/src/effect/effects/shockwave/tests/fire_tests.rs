use bevy::prelude::*;

use super::*;
use crate::effect::core::EffectSourceChip;

// -- fire tests ──────────────────────────────────────────────────

#[test]
fn fire_spawns_shockwave_entity_at_source_position() {
    let mut world = World::new();
    let entity = world.spawn(Position2D(Vec2::new(100.0, 200.0))).id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", &mut world);

    let mut query = world.query::<(
        &ShockwaveSource,
        &ShockwaveRadius,
        &ShockwaveMaxRadius,
        &ShockwaveSpeed,
        &Position2D,
    )>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected exactly one shockwave entity");

    let (_source, radius, max_radius, speed, pos) = results[0];
    assert!(
        (radius.0 - 0.0).abs() < f32::EPSILON,
        "expected radius 0.0, got {}",
        radius.0
    );
    // stacks=1 -> effective = 24.0 + (1-1)*8.0 = 24.0
    assert!(
        (max_radius.0 - 24.0).abs() < f32::EPSILON,
        "expected max_radius 24.0, got {}",
        max_radius.0
    );
    assert!(
        (speed.0 - 50.0).abs() < f32::EPSILON,
        "expected speed 50.0, got {}",
        speed.0
    );
    assert!(
        (pos.0.x - 100.0).abs() < f32::EPSILON,
        "expected x 100.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - 200.0).abs() < f32::EPSILON,
        "expected y 200.0, got {}",
        pos.0.y
    );
}

#[test]
fn fire_effective_range_scales_with_stacks() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

    // stacks=3, base=24, per_level=8 -> effective = 24 + (3-1)*8 = 40
    fire(entity, 24.0, 8.0, 3, 50.0, "", &mut world);

    let mut query = world.query::<&ShockwaveMaxRadius>();
    let max_radius = query.iter(&world).next().unwrap();
    assert!(
        (max_radius.0 - 40.0).abs() < f32::EPSILON,
        "expected max_radius 40.0, got {}",
        max_radius.0
    );
}

// -- Behavior 7: fire() spawns ShockwaveDamaged on shockwave entity ──

#[test]
fn fire_spawns_shockwave_damaged_component_on_entity() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(50.0, 75.0, 0.0)).id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", &mut world);

    let mut query = world.query::<&ShockwaveDamaged>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "expected exactly one entity with ShockwaveDamaged"
    );
    assert!(
        results[0].0.is_empty(),
        "ShockwaveDamaged set should be empty on spawn"
    );
}

#[test]
fn fire_twice_spawns_two_independent_shockwave_damaged() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(50.0, 75.0, 0.0)).id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", &mut world);
    fire(entity, 24.0, 8.0, 1, 50.0, "", &mut world);

    let mut query = world.query::<&ShockwaveDamaged>();
    let count = query.iter(&world).count();
    assert_eq!(
        count, 2,
        "two fire() calls should produce two ShockwaveDamaged components"
    );

    for damaged in query.iter(&world) {
        assert!(
            damaged.0.is_empty(),
            "each ShockwaveDamaged should start with an empty HashSet"
        );
    }
}

// -- Section C: EffectSourceChip attribution tests ───────────────────

#[test]
fn fire_stores_effect_source_chip_with_non_empty_chip_name() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(100.0, 200.0, 0.0)).id();

    fire(entity, 24.0, 8.0, 1, 50.0, "seismic", &mut world);

    let mut query = world.query::<&EffectSourceChip>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one entity with EffectSourceChip"
    );
    assert_eq!(
        results[0].0,
        Some("seismic".to_string()),
        "spawned shockwave should have EffectSourceChip(Some(\"seismic\"))"
    );
}

#[test]
fn fire_stores_effect_source_chip_none_with_empty_chip_name() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", &mut world);

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

// ── Behavior: shockwave fire() reads Position2D for spawn position ──

#[test]
fn fire_reads_position2d_for_spawn_position() {
    let mut world = World::new();
    let entity = world.spawn(Position2D(Vec2::new(100.0, 200.0))).id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", &mut world);

    let mut query = world.query::<(&ShockwaveSource, &Position2D)>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected exactly one shockwave entity");

    let (_source, pos) = results[0];
    assert!(
        (pos.0.x - 100.0).abs() < f32::EPSILON,
        "expected shockwave Position2D x=100.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - 200.0).abs() < f32::EPSILON,
        "expected shockwave Position2D y=200.0, got {}",
        pos.0.y
    );
}

// ── Behavior: shockwave fire() uses Position2D not Transform when both present ──

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

    fire(entity, 24.0, 8.0, 1, 50.0, "", &mut world);

    let mut query = world.query::<(&ShockwaveSource, &Position2D)>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected exactly one shockwave entity");

    let (_source, pos) = results[0];
    assert!(
        (pos.0.x - 50.0).abs() < f32::EPSILON,
        "shockwave should use Position2D x=50.0, not Transform x=999.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - 75.0).abs() < f32::EPSILON,
        "shockwave should use Position2D y=75.0, not Transform y=888.0, got {}",
        pos.0.y
    );
}

// ── Behavior: shockwave fire() falls back to Vec2::ZERO when Position2D absent ──

#[test]
fn fire_falls_back_to_zero_when_position2d_absent() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", &mut world);

    let mut query = world.query::<(&ShockwaveSource, &Position2D)>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected exactly one shockwave entity");

    let (_source, pos) = results[0];
    assert!(
        (pos.0.x).abs() < f32::EPSILON,
        "shockwave Position2D x should default to 0.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y).abs() < f32::EPSILON,
        "shockwave Position2D y should default to 0.0, got {}",
        pos.0.y
    );
}
