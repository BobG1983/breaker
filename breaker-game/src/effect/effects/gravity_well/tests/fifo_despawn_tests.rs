use bevy::prelude::*;
use rantzsoft_spatial2d::prelude::*;

use crate::effect::effects::gravity_well::effect::*;

/// Helper: call `fire()` with standard test values. Only `entity`, `max`, and `world` vary.
fn fire_well(entity: Entity, max: u32, world: &mut World) {
    fire(entity, 100.0, 10.0, 50.0, max, "", world);
}

/// Helper: collect sorted spawn-order values for all gravity wells owned by `owner`.
fn owned_spawn_orders(world: &mut World, owner: Entity) -> Vec<u64> {
    let mut query = world.query::<(&GravityWellSpawnOrder, &GravityWellConfig)>();
    let mut orders: Vec<u64> = query
        .iter(world)
        .filter(|(_, config)| config.owner == owner)
        .map(|(order, _)| order.0)
        .collect();
    orders.sort_unstable();
    orders
}

/// Helper: count gravity wells owned by `owner`.
fn owned_well_count(world: &mut World, owner: Entity) -> usize {
    let mut query = world.query::<&GravityWellConfig>();
    query
        .iter(world)
        .filter(|config| config.owner == owner)
        .count()
}

// ── Behavior 1: First well gets GravityWellSpawnOrder(0) ────────────

#[test]
fn first_gravity_well_gets_spawn_order_zero() {
    let mut world = World::new();
    let owner = world.spawn(Position2D(Vec2::new(50.0, 75.0))).id();

    fire_well(owner, 4, &mut world);

    let orders = owned_spawn_orders(&mut world, owner);
    assert_eq!(
        orders,
        vec![0],
        "first gravity well should have GravityWellSpawnOrder(0)"
    );
}

#[test]
fn first_fire_lazily_initializes_spawn_counter_resource() {
    let mut world = World::new();
    let owner = world.spawn(Position2D(Vec2::new(50.0, 75.0))).id();

    assert!(
        !world.contains_resource::<GravityWellSpawnCounter>(),
        "GravityWellSpawnCounter should not exist before first fire()"
    );

    fire_well(owner, 4, &mut world);

    assert!(
        world.contains_resource::<GravityWellSpawnCounter>(),
        "GravityWellSpawnCounter should be lazily initialized by fire()"
    );
}

// ── Behavior 2: Second well gets GravityWellSpawnOrder(1) ───────────

#[test]
fn second_gravity_well_gets_spawn_order_one() {
    let mut world = World::new();
    let owner = world.spawn(Position2D(Vec2::new(50.0, 75.0))).id();

    fire_well(owner, 4, &mut world);
    fire_well(owner, 4, &mut world);

    let orders = owned_spawn_orders(&mut world, owner);
    assert_eq!(
        orders,
        vec![0, 1],
        "two wells should have spawn orders 0 and 1"
    );
}

// ── Behavior 3: Wells within max are not despawned ──────────────────

#[test]
fn wells_within_max_count_are_not_despawned() {
    let mut world = World::new();
    let owner = world.spawn(Position2D(Vec2::ZERO)).id();

    for _ in 0..4 {
        fire_well(owner, 4, &mut world);
    }

    let count = owned_well_count(&mut world, owner);
    assert_eq!(count, 4, "exactly 4 wells should exist (max=4, fired=4)");

    let orders = owned_spawn_orders(&mut world, owner);
    assert_eq!(
        orders,
        vec![0, 1, 2, 3],
        "4 wells at max should have orders 0, 1, 2, 3 — none despawned"
    );
}

// ── Behavior 4: Exceeding max despawns oldest (lowest order) ────────

#[test]
fn exceeding_max_despawns_oldest_gravity_well() {
    let mut world = World::new();
    let owner = world.spawn(Position2D(Vec2::ZERO)).id();

    for _ in 0..5 {
        fire_well(owner, 4, &mut world);
    }

    let count = owned_well_count(&mut world, owner);
    assert_eq!(
        count, 4,
        "should still have exactly 4 wells after 5 fires with max=4"
    );

    let orders = owned_spawn_orders(&mut world, owner);
    assert_eq!(
        orders,
        vec![1, 2, 3, 4],
        "well with order 0 should be despawned; remaining should be 1, 2, 3, 4"
    );
}

// ── Behavior 5: Multiple excess fires despawn in FIFO order ─────────

#[test]
fn multiple_excess_fires_despawn_oldest_in_fifo_order() {
    let mut world = World::new();
    let owner = world.spawn(Position2D(Vec2::ZERO)).id();

    for _ in 0..6 {
        fire_well(owner, 4, &mut world);
    }

    let count = owned_well_count(&mut world, owner);
    assert_eq!(
        count, 4,
        "should have exactly 4 wells after 6 fires with max=4"
    );

    let orders = owned_spawn_orders(&mut world, owner);
    assert_eq!(
        orders,
        vec![2, 3, 4, 5],
        "wells with orders 0 and 1 should be despawned; remaining should be 2, 3, 4, 5"
    );
}

// ── Behavior 6: Counter is per-owner — independent ordering ─────────

#[test]
fn spawn_counter_is_per_owner_independent_ordering() {
    let mut world = World::new();
    let owner_a = world.spawn(Position2D(Vec2::new(10.0, 20.0))).id();
    let owner_b = world.spawn(Position2D(Vec2::new(30.0, 40.0))).id();

    fire_well(owner_a, 4, &mut world);
    fire_well(owner_a, 4, &mut world);
    fire_well(owner_b, 4, &mut world);
    fire_well(owner_b, 4, &mut world);

    let total = {
        let mut query = world.query::<&GravityWellConfig>();
        query.iter(&world).count()
    };
    assert_eq!(total, 4, "should have 4 total wells (2 per owner)");

    let orders_a = owned_spawn_orders(&mut world, owner_a);
    assert_eq!(
        orders_a,
        vec![0, 1],
        "owner_a wells should have independent orders 0, 1"
    );

    let orders_b = owned_spawn_orders(&mut world, owner_b);
    assert_eq!(
        orders_b,
        vec![0, 1],
        "owner_b wells should have independent orders 0, 1"
    );
}

// ── Behavior 7: Per-owner max — one at max does not affect other ────

#[test]
fn per_owner_max_enforcement_does_not_affect_other_owners() {
    let mut world = World::new();
    let owner_a = world.spawn(Position2D(Vec2::new(10.0, 20.0))).id();
    let owner_b = world.spawn(Position2D(Vec2::new(30.0, 40.0))).id();

    // owner_a fires 4 wells (at max=4)
    for _ in 0..4 {
        fire_well(owner_a, 4, &mut world);
    }

    // owner_a fires a 5th — should despawn owner_a's oldest, not owner_b's
    fire_well(owner_a, 4, &mut world);

    // owner_b fires 1 well
    fire_well(owner_b, 4, &mut world);

    let orders_a = owned_spawn_orders(&mut world, owner_a);
    assert_eq!(
        orders_a,
        vec![1, 2, 3, 4],
        "owner_a should have 4 wells with orders 1, 2, 3, 4 (order 0 despawned)"
    );

    let orders_b = owned_spawn_orders(&mut world, owner_b);
    assert_eq!(
        orders_b,
        vec![0],
        "owner_b should have 1 well with order 0 (unaffected by owner_a's despawn)"
    );

    let total = {
        let mut query = world.query::<&GravityWellConfig>();
        query.iter(&world).count()
    };
    assert_eq!(
        total, 5,
        "total wells should be 5 (4 for owner_a + 1 for owner_b)"
    );
}

// ── Behavior 8: max=1 immediately despawns existing well ────────────

#[test]
fn max_one_immediately_despawns_existing_well_on_new_fire() {
    let mut world = World::new();
    let owner = world.spawn(Position2D(Vec2::ZERO)).id();

    fire_well(owner, 1, &mut world);
    fire_well(owner, 1, &mut world);

    let count = owned_well_count(&mut world, owner);
    assert_eq!(count, 1, "max=1 should leave exactly 1 well after 2 fires");

    let orders = owned_spawn_orders(&mut world, owner);
    assert_eq!(
        orders,
        vec![1],
        "max=1: only the newest well (order 1) should survive; order 0 should be despawned"
    );
}

// ── Behavior 9: max=0 early-returns without spawning ────────────────

#[test]
fn max_zero_does_not_spawn_any_wells() {
    let mut world = World::new();
    let owner = world.spawn(Position2D(Vec2::ZERO)).id();

    fire_well(owner, 0, &mut world);

    let count = {
        let mut query = world.query::<&GravityWellConfig>();
        query.iter(&world).count()
    };
    assert_eq!(count, 0, "max=0 should spawn no gravity wells");
}

#[test]
fn max_zero_does_not_initialize_spawn_counter() {
    let mut world = World::new();
    let owner = world.spawn(Position2D(Vec2::ZERO)).id();

    fire_well(owner, 0, &mut world);

    assert!(
        !world.contains_resource::<GravityWellSpawnCounter>(),
        "max=0 should not lazily initialize GravityWellSpawnCounter"
    );
}
