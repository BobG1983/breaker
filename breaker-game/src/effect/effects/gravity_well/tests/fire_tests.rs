use bevy::prelude::*;
use rantzsoft_spatial2d::prelude::*;

use super::super::effect::*;

// ── fire tests ──────────────────────────────────────────────────

#[test]
fn fire_with_max_zero_returns_immediately() {
    let mut world = World::new();
    let entity = world.spawn(Position2D(Vec2::new(50.0, 75.0))).id();

    fire(entity, 100.0, 5.0, 80.0, 0, "", &mut world);

    let mut query = world.query::<&GravityWellConfig>();
    let count = query.iter(&world).count();
    assert_eq!(count, 0, "no well entities should be spawned when max is 0");
}

#[test]
fn fire_spawns_well_entity_with_marker_and_config() {
    let mut world = World::new();
    let entity = world.spawn(Position2D(Vec2::new(50.0, 75.0))).id();

    fire(entity, 100.0, 5.0, 80.0, 3, "", &mut world);

    let mut query = world.query::<(&GravityWell, &GravityWellConfig, &Position2D)>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected exactly one gravity well");

    let (_marker, config, position) = results[0];
    assert!(
        (config.strength - 100.0).abs() < f32::EPSILON,
        "expected strength 100.0, got {}",
        config.strength
    );
    assert!(
        (config.radius - 80.0).abs() < f32::EPSILON,
        "expected radius 80.0, got {}",
        config.radius
    );
    assert!(
        (config.remaining - 5.0).abs() < f32::EPSILON,
        "expected remaining 5.0, got {}",
        config.remaining
    );
    assert_eq!(config.owner, entity);
    assert!(
        (position.0.x - 50.0).abs() < f32::EPSILON,
        "expected x 50.0, got {}",
        position.0.x
    );
    assert!(
        (position.0.y - 75.0).abs() < f32::EPSILON,
        "expected y 75.0, got {}",
        position.0.y
    );
}

#[test]
fn fire_enforces_max_cap_despawns_oldest() {
    let mut world = World::new();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    // Spawn 3 wells with max=2
    fire(entity, 100.0, 5.0, 80.0, 2, "", &mut world);
    fire(entity, 100.0, 5.0, 80.0, 2, "", &mut world);
    fire(entity, 100.0, 5.0, 80.0, 2, "", &mut world);

    let mut query = world.query::<&GravityWellConfig>();
    let count = query.iter(&world).count();
    assert_eq!(count, 2, "should enforce max of 2 wells, got {count}");
}

#[test]
fn reverse_is_noop() {
    let mut world = World::new();
    let owner = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(owner, 100.0, 5.0, 80.0, 10, "", &mut world);
    reverse(owner, "", &mut world);

    // Wells should still exist — reverse is a no-op
    let mut query = world.query::<&GravityWellConfig>();
    let count = query.iter(&world).count();
    assert_eq!(count, 1, "reverse should not despawn wells (no-op)");
}

// ── fire() reads Position2D, not Transform ────────────────────

#[test]
fn fire_reads_position2d_not_transform_for_well_spawn_position() {
    let mut world = World::new();
    // Position2D and Transform are deliberately different to catch the wrong read.
    let entity = world
        .spawn((
            Position2D(Vec2::new(100.0, 200.0)),
            Transform::from_xyz(999.0, 999.0, 0.0),
        ))
        .id();

    fire(entity, 100.0, 5.0, 80.0, 3, "", &mut world);

    let mut query = world.query::<(&GravityWell, &Position2D)>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected exactly one gravity well");

    let (_marker, pos) = results[0];
    assert_eq!(
        pos.0,
        Vec2::new(100.0, 200.0),
        "well should spawn at Position2D (100, 200), not Transform (999, 999)"
    );
}

#[test]
fn fire_reads_position2d_zero_not_transform_for_well_spawn_position() {
    let mut world = World::new();
    // Edge case: Position2D at origin, Transform at a non-zero position.
    let entity = world
        .spawn((Position2D(Vec2::ZERO), Transform::from_xyz(50.0, 50.0, 0.0)))
        .id();

    fire(entity, 100.0, 5.0, 80.0, 3, "", &mut world);

    let mut query = world.query::<(&GravityWell, &Position2D)>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected exactly one gravity well");

    let (_marker, pos) = results[0];
    assert_eq!(
        pos.0,
        Vec2::ZERO,
        "well should spawn at Position2D (0, 0), not Transform (50, 50)"
    );
}

#[test]
fn fire_falls_back_to_zero_when_entity_has_no_position2d() {
    let mut world = World::new();
    // Entity has only Transform, no Position2D. fire() should fall back to Vec2::ZERO.
    let entity = world.spawn(Transform::from_xyz(50.0, 75.0, 0.0)).id();

    fire(entity, 100.0, 5.0, 80.0, 3, "", &mut world);

    let mut query = world.query::<(&GravityWell, &Position2D)>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected exactly one gravity well");

    let (_marker, pos) = results[0];
    assert_eq!(
        pos.0,
        Vec2::ZERO,
        "well should default to Position2D(Vec2::ZERO) when owner has no Position2D"
    );
}

#[test]
fn fire_falls_back_to_zero_when_entity_is_empty() {
    let mut world = World::new();
    // Entity has neither Position2D nor Transform.
    let entity = world.spawn_empty().id();

    fire(entity, 100.0, 5.0, 80.0, 3, "", &mut world);

    let mut query = world.query::<(&GravityWell, &Position2D)>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected exactly one gravity well");

    let (_marker, pos) = results[0];
    assert_eq!(
        pos.0,
        Vec2::ZERO,
        "well should default to Position2D(Vec2::ZERO) when owner is empty"
    );
}

// ── Spawned well entity has CleanupOnNodeExit ───────────────

#[test]
fn fire_spawns_well_with_cleanup_on_node_exit() {
    use crate::shared::CleanupOnNodeExit;

    let mut world = World::new();
    let entity = world.spawn(Position2D(Vec2::new(50.0, 75.0))).id();

    fire(entity, 100.0, 5.0, 80.0, 3, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<GravityWell>>();
    let well = query.iter(&world).next().expect("well should exist");

    assert!(
        world.get::<CleanupOnNodeExit>(well).is_some(),
        "spawned gravity well should have CleanupOnNodeExit"
    );
}

#[test]
fn fire_multiple_wells_all_have_cleanup_on_node_exit() {
    use crate::shared::CleanupOnNodeExit;

    let mut world = World::new();
    let entity = world.spawn(Position2D(Vec2::new(10.0, 20.0))).id();

    // Spawn 3 wells with max=5 so none get despawned.
    fire(entity, 100.0, 5.0, 80.0, 5, "", &mut world);
    fire(entity, 200.0, 3.0, 60.0, 5, "", &mut world);
    fire(entity, 300.0, 4.0, 70.0, 5, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<GravityWell>>();
    let wells: Vec<Entity> = query.iter(&world).collect();
    assert_eq!(wells.len(), 3, "expected 3 gravity wells");

    for well in &wells {
        assert!(
            world.get::<CleanupOnNodeExit>(*well).is_some(),
            "ALL spawned gravity wells should have CleanupOnNodeExit"
        );
    }
}
