//! Regression tests for `SecondWind` `fire()` duplicate wall guard.

use bevy::prelude::*;

use crate::{effect::effects::second_wind::system::*, shared::PlayfieldConfig};

#[test]
fn fire_with_existing_wall_does_not_spawn_second() {
    // Regression: fire() spawned a wall unconditionally, allowing wall count > 1.
    // Given: A SecondWindWall entity already exists.
    // When: fire() is called again.
    // Then: Wall count remains 1 (no additional wall spawned).
    let mut world = World::new();
    world.insert_resource(PlayfieldConfig::default());
    let entity = world.spawn_empty().id();

    // First fire -- should spawn the wall
    fire(entity, "", &mut world);
    let count_after_first: usize = world
        .query_filtered::<Entity, With<SecondWindWall>>()
        .iter(&world)
        .count();
    assert_eq!(
        count_after_first, 1,
        "first fire should spawn exactly one SecondWindWall"
    );

    // Second fire -- should NOT spawn another wall
    fire(entity, "", &mut world);
    let count_after_second: usize = world
        .query_filtered::<Entity, With<SecondWindWall>>()
        .iter(&world)
        .count();
    assert_eq!(
        count_after_second, 1,
        "second fire should not spawn another wall when one already exists, got {count_after_second}"
    );
}

#[test]
fn fire_without_existing_wall_spawns_wall() {
    // Positive companion: fire() spawns a wall when none exists.
    // Given: No SecondWindWall entities exist.
    // When: fire() is called.
    // Then: Exactly one SecondWindWall is spawned.
    let mut world = World::new();
    world.insert_resource(PlayfieldConfig::default());
    let entity = world.spawn_empty().id();

    let count_before: usize = world
        .query_filtered::<Entity, With<SecondWindWall>>()
        .iter(&world)
        .count();
    assert_eq!(count_before, 0, "precondition: no walls should exist");

    fire(entity, "", &mut world);

    let count_after: usize = world
        .query_filtered::<Entity, With<SecondWindWall>>()
        .iter(&world)
        .count();
    assert_eq!(
        count_after, 1,
        "fire should spawn exactly one SecondWindWall when none exists"
    );
}
