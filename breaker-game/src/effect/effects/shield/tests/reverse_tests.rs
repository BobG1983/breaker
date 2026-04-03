use super::helpers::*;

// ── Section B: reverse() despawns ShieldWall ────────────────────────

// Behavior 8: reverse() despawns all ShieldWall entities

#[test]
fn reverse_despawns_all_shield_wall_entities() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    // Spawn one ShieldWall
    world.spawn((
        ShieldWall,
        ShieldWallTimer(Timer::from_seconds(5.0, TimerMode::Once)),
    ));

    reverse(entity, "parry", &mut world);

    let count = world
        .query_filtered::<Entity, With<ShieldWall>>()
        .iter(&world)
        .count();
    assert_eq!(
        count, 0,
        "reverse should despawn all ShieldWall entities, found {count}"
    );
}

#[test]
fn reverse_despawns_multiple_shield_walls() {
    // Edge case: two ShieldWall entities (defensive)
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.spawn((
        ShieldWall,
        ShieldWallTimer(Timer::from_seconds(5.0, TimerMode::Once)),
    ));
    world.spawn((
        ShieldWall,
        ShieldWallTimer(Timer::from_seconds(3.0, TimerMode::Once)),
    ));

    reverse(entity, "parry", &mut world);

    let count = world
        .query_filtered::<Entity, With<ShieldWall>>()
        .iter(&world)
        .count();
    assert_eq!(
        count, 0,
        "reverse should despawn all ShieldWall entities even if multiple exist"
    );
}

// Behavior 9: reverse() on world with no ShieldWall does not panic

#[test]
fn reverse_with_no_shield_wall_does_not_panic() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    reverse(entity, "parry", &mut world); // should not panic
}
