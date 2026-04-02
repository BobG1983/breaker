//! Tests for `SpawnPhantom` FIFO despawn ordering via `PhantomSpawnOrder`
//! and `PhantomSpawnCounter`.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::super::effect::*;
use crate::{
    bolt::{definition::BoltDefinition, registry::BoltRegistry},
    shared::rng::GameRng,
};

/// Creates a bare World with `BoltRegistry` (default Bolt definition) and `GameRng` seeded at 42.
fn world_with_seed_42() -> World {
    let mut world = World::new();
    let mut registry = BoltRegistry::default();
    registry.insert(
        "Bolt".to_string(),
        BoltDefinition {
            name: "Bolt".to_owned(),
            base_speed: 400.0,
            min_speed: 200.0,
            max_speed: 800.0,
            radius: 8.0,
            base_damage: 10.0,
            effects: vec![],
            color_rgb: [6.0, 5.0, 0.5],
            min_angle_horizontal: 5.0,
            min_angle_vertical: 5.0,
            min_radius: None,
            max_radius: None,
        },
    );
    world.insert_resource(registry);
    world.insert_resource(GameRng::from_seed(42));
    world
}

/// Collects all `PhantomSpawnOrder` values for phantoms owned by `owner`,
/// sorted ascending.
fn sorted_spawn_orders(world: &mut World, owner: Entity) -> Vec<u64> {
    let mut query = world.query::<(&PhantomSpawnOrder, &PhantomOwner)>();
    let mut orders: Vec<u64> = query
        .iter(world)
        .filter(|(_, o)| o.0 == owner)
        .map(|(order, _)| order.0)
        .collect();
    orders.sort_unstable();
    orders
}

/// Counts phantom bolt entities owned by `owner`.
fn phantom_count_for_owner(world: &mut World, owner: Entity) -> usize {
    let mut query = world.query::<(&PhantomBoltMarker, &PhantomOwner)>();
    query.iter(world).filter(|(_, o)| o.0 == owner).count()
}

// ── Behavior 1: First phantom spawned gets PhantomSpawnOrder(0) ─────────

#[test]
fn first_phantom_gets_spawn_order_zero() {
    let mut world = world_with_seed_42();
    let owner = world.spawn(Position2D(Vec2::new(100.0, 200.0))).id();

    fire(owner, 5.0, 5, "", &mut world);

    // Exactly 1 phantom with PhantomBoltMarker and PhantomOwner(owner)
    assert_eq!(
        phantom_count_for_owner(&mut world, owner),
        1,
        "expected exactly 1 phantom after one fire() call"
    );

    // That phantom has PhantomSpawnOrder(0)
    let orders = sorted_spawn_orders(&mut world, owner);
    assert_eq!(
        orders,
        vec![0],
        "first phantom should have PhantomSpawnOrder(0)"
    );

    // PhantomSpawnCounter resource exists and tracks next value = 1
    let counter = world
        .get_resource::<PhantomSpawnCounter>()
        .expect("PhantomSpawnCounter resource should be lazily created by first fire() call");
    let next_value = counter.0.get(&owner).copied().unwrap_or(0);
    assert_eq!(
        next_value, 1,
        "counter for owner should be 1 after one phantom spawned"
    );
}

// ── Edge case for Behavior 1: PhantomSpawnCounter lazily created ────────

#[test]
fn phantom_spawn_counter_lazily_created_on_first_fire() {
    let mut world = world_with_seed_42();
    let owner = world.spawn(Position2D(Vec2::new(100.0, 200.0))).id();

    // Before fire(), no PhantomSpawnCounter should exist
    assert!(
        world.get_resource::<PhantomSpawnCounter>().is_none(),
        "PhantomSpawnCounter should not exist before first fire() call"
    );

    fire(owner, 5.0, 5, "", &mut world);

    assert!(
        world.get_resource::<PhantomSpawnCounter>().is_some(),
        "PhantomSpawnCounter should exist after first fire() call"
    );
}

// ── Behavior 2: Second phantom gets PhantomSpawnOrder(1) ────────────────

#[test]
fn second_phantom_gets_spawn_order_one() {
    let mut world = world_with_seed_42();
    let owner = world.spawn(Position2D(Vec2::new(100.0, 200.0))).id();

    fire(owner, 5.0, 5, "", &mut world);
    fire(owner, 5.0, 5, "", &mut world);

    assert_eq!(
        phantom_count_for_owner(&mut world, owner),
        2,
        "expected 2 phantoms after two fire() calls"
    );

    let orders = sorted_spawn_orders(&mut world, owner);
    assert_eq!(
        orders,
        vec![0, 1],
        "first phantom should keep order 0, second should get order 1"
    );
}

// ── Behavior 3: Phantoms within max_active are not despawned ────────────

#[test]
fn phantoms_within_max_active_not_despawned() {
    let mut world = world_with_seed_42();
    let owner = world.spawn(Position2D(Vec2::ZERO)).id();

    for _ in 0..5 {
        fire(owner, 5.0, 5, "", &mut world);
    }

    assert_eq!(
        phantom_count_for_owner(&mut world, owner),
        5,
        "at exactly max_active=5, all 5 phantoms should survive"
    );

    let orders = sorted_spawn_orders(&mut world, owner);
    assert_eq!(
        orders,
        vec![0, 1, 2, 3, 4],
        "5 phantoms at cap should have orders 0..4"
    );
}

// ── Behavior 4: Exceeding max_active despawns oldest (lowest order) ─────

#[test]
fn exceeding_max_active_despawns_oldest_phantom() {
    let mut world = world_with_seed_42();
    let owner = world.spawn(Position2D(Vec2::ZERO)).id();

    for _ in 0..6 {
        fire(owner, 5.0, 5, "", &mut world);
    }

    assert_eq!(
        phantom_count_for_owner(&mut world, owner),
        5,
        "after 6 fire() calls with max_active=5, only 5 phantoms should remain"
    );

    let orders = sorted_spawn_orders(&mut world, owner);
    assert_eq!(
        orders,
        vec![1, 2, 3, 4, 5],
        "oldest phantom (order 0) should be despawned; new phantom gets order 5"
    );
}

// ── Behavior 5: Multiple despawns maintain FIFO order ───────────────────

#[test]
fn multiple_despawns_maintain_fifo_order() {
    let mut world = world_with_seed_42();
    let owner = world.spawn(Position2D(Vec2::ZERO)).id();

    for _ in 0..7 {
        fire(owner, 5.0, 5, "", &mut world);
    }

    assert_eq!(
        phantom_count_for_owner(&mut world, owner),
        5,
        "after 7 fire() calls with max_active=5, only 5 phantoms should remain"
    );

    let orders = sorted_spawn_orders(&mut world, owner);
    assert_eq!(
        orders,
        vec![2, 3, 4, 5, 6],
        "phantoms with orders 0 and 1 should be despawned (FIFO)"
    );
}

// ── Behavior 6: Counter is per-owner ────────────────────────────────────

#[test]
fn counter_is_per_owner_independent_ordering() {
    let mut world = world_with_seed_42();
    let owner_a = world.spawn(Position2D(Vec2::ZERO)).id();
    let owner_b = world.spawn(Position2D(Vec2::new(50.0, 50.0))).id();

    fire(owner_a, 5.0, 5, "", &mut world);
    fire(owner_b, 5.0, 5, "", &mut world);

    let orders_a = sorted_spawn_orders(&mut world, owner_a);
    let orders_b = sorted_spawn_orders(&mut world, owner_b);

    assert_eq!(orders_a, vec![0], "owner_a's phantom should have order 0");
    assert_eq!(
        orders_b,
        vec![0],
        "owner_b's phantom should independently have order 0"
    );

    // Counters are independent
    let counter = world
        .get_resource::<PhantomSpawnCounter>()
        .expect("PhantomSpawnCounter should exist");
    let next_a = counter.0.get(&owner_a).copied().unwrap_or(0);
    let next_b = counter.0.get(&owner_b).copied().unwrap_or(0);
    assert_eq!(next_a, 1, "owner_a counter should be 1");
    assert_eq!(next_b, 1, "owner_b counter should be 1");
}

// ── Behavior 7: Per-owner max_active enforcement ────────────────────────

#[test]
fn per_owner_max_active_despawn_only_affects_that_owner() {
    let mut world = world_with_seed_42();
    let owner_a = world.spawn(Position2D(Vec2::ZERO)).id();
    let owner_b = world.spawn(Position2D(Vec2::new(50.0, 50.0))).id();

    // owner_a: 5 phantoms (at cap)
    for _ in 0..5 {
        fire(owner_a, 5.0, 5, "", &mut world);
    }

    // owner_b: 2 phantoms (below cap)
    for _ in 0..2 {
        fire(owner_b, 5.0, 5, "", &mut world);
    }

    // owner_a: 6th fire — should despawn owner_a's oldest
    fire(owner_a, 5.0, 5, "", &mut world);

    // owner_a: 5 phantoms with orders 1..5
    assert_eq!(
        phantom_count_for_owner(&mut world, owner_a),
        5,
        "owner_a should have 5 phantoms after 6th fire()"
    );
    let orders_a = sorted_spawn_orders(&mut world, owner_a);
    assert_eq!(
        orders_a,
        vec![1, 2, 3, 4, 5],
        "owner_a's oldest (order 0) should be despawned"
    );

    // owner_b: still 2 phantoms, untouched
    assert_eq!(
        phantom_count_for_owner(&mut world, owner_b),
        2,
        "owner_b should still have 2 phantoms — untouched by owner_a's despawn"
    );
    let orders_b = sorted_spawn_orders(&mut world, owner_b);
    assert_eq!(
        orders_b,
        vec![0, 1],
        "owner_b's phantoms should be unchanged"
    );
}

// ── Behavior 8: max_active=0 spawns nothing ─────────────────────────────

#[test]
fn max_active_zero_spawns_nothing() {
    let mut world = world_with_seed_42();
    let owner = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(owner, 5.0, 0, "", &mut world);

    let mut query = world.query::<&PhantomBoltMarker>();
    let count = query.iter(&world).count();
    assert_eq!(count, 0, "max_active=0 should spawn no phantoms");

    assert!(
        world.get_resource::<PhantomSpawnCounter>().is_none(),
        "PhantomSpawnCounter should not be created when max_active=0"
    );
}

// ── Behavior 9: max_active=1 replaces the single phantom each time ──────

#[test]
fn max_active_one_replaces_single_phantom_fifo() {
    let mut world = world_with_seed_42();
    let owner = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(owner, 5.0, 1, "", &mut world);
    fire(owner, 5.0, 1, "", &mut world);
    fire(owner, 5.0, 1, "", &mut world);

    assert_eq!(
        phantom_count_for_owner(&mut world, owner),
        1,
        "max_active=1: only 1 phantom should exist after 3 fire() calls"
    );

    let orders = sorted_spawn_orders(&mut world, owner);
    assert_eq!(
        orders,
        vec![2],
        "only the third phantom (order 2) should survive; orders 0 and 1 despawned"
    );
}

// ── Behavior 10: PhantomSpawnCounter persists across calls ──────────────

#[test]
fn phantom_spawn_counter_persists_across_calls() {
    let mut world = world_with_seed_42();
    let owner = world.spawn(Position2D(Vec2::ZERO)).id();

    // Fire 3 times (counter reaches 3)
    for _ in 0..3 {
        fire(owner, 5.0, 5, "", &mut world);
    }

    // Verify counter is at 3
    {
        let counter = world
            .get_resource::<PhantomSpawnCounter>()
            .expect("PhantomSpawnCounter should exist after 3 calls");
        let next = counter.0.get(&owner).copied().unwrap_or(0);
        assert_eq!(next, 3, "counter should be 3 after 3 fire() calls");
    }

    // 4th fire — should get order 3 (not reset to 0)
    fire(owner, 5.0, 5, "", &mut world);

    let orders = sorted_spawn_orders(&mut world, owner);
    assert_eq!(
        orders,
        vec![0, 1, 2, 3],
        "4th phantom should get order 3, counter continues from previous value"
    );

    let counter = world
        .get_resource::<PhantomSpawnCounter>()
        .expect("PhantomSpawnCounter should still exist");
    let next = counter.0.get(&owner).copied().unwrap_or(0);
    assert_eq!(next, 4, "counter should be 4 after 4th fire() call");
}
