//! T23 — `max_active` filter matches by `(chip, fired_from)` pair,
//! not just by source entity (Behavior 2 + edge cases).

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{
    super::config_impl::SpawnPhantomConfig,
    helpers::{spawn_keyed_phantom, spawn_source, world_with_assets},
};
use crate::{
    bolt::components::{PhantomBolt, PhantomDedupKey},
    effect_v3::{effects::phantom_bolt::components::PhantomOwner, traits::Fireable},
};

// ── T23 primary — at-limit suppresses spawn ──────────────────────────────────

#[test]
fn max_active_at_limit_suppresses_new_spawn() {
    let mut world = world_with_assets();
    let real_bolt = spawn_source(&mut world, Vec2::ZERO, Vec2::ZERO);

    // Pre-spawn 2 phantoms keyed for (chip="phantom_bolt", fired_from=real_bolt)
    spawn_keyed_phantom(&mut world, "phantom_bolt", real_bolt);
    spawn_keyed_phantom(&mut world, "phantom_bolt", real_bolt);

    let config = SpawnPhantomConfig {
        duration:   OrderedFloat(2.0),
        max_active: 2,
    };
    config.fire(real_bolt, "phantom_bolt", &mut world);
    world.flush();

    let count = world
        .query_filtered::<Entity, (With<PhantomBolt>, With<PhantomDedupKey>)>()
        .iter(&world)
        .count();
    assert_eq!(
        count, 2,
        "spawn must be suppressed when existing (chip, fired_from) count >= max_active"
    );
}

// ── T23 edge case 2a — different chip bypasses the filter ────────────────────

#[test]
fn max_active_different_chip_bypasses_filter() {
    let mut world = world_with_assets();
    let real_bolt = spawn_source(&mut world, Vec2::new(10.0, 10.0), Vec2::new(0.0, 200.0));

    // Pre-spawn 2 phantoms keyed for "phantom_bolt"
    spawn_keyed_phantom(&mut world, "phantom_bolt", real_bolt);
    spawn_keyed_phantom(&mut world, "phantom_bolt", real_bolt);

    let config = SpawnPhantomConfig {
        duration:   OrderedFloat(2.0),
        max_active: 2,
    };
    config.fire(real_bolt, "different_chip", &mut world);
    world.flush();

    let count = world
        .query_filtered::<Entity, (With<PhantomBolt>, With<PhantomDedupKey>)>()
        .iter(&world)
        .count();
    assert_eq!(
        count, 3,
        "different chip name must bypass the max_active filter; count should grow to 3"
    );

    // The newly spawned phantom must carry the new chip name
    let new_key = world
        .query::<&PhantomDedupKey>()
        .iter(&world)
        .find(|key| matches!(key, PhantomDedupKey::Chip { chip, .. } if chip == "different_chip"));
    assert!(
        new_key.is_some(),
        "new phantom must have PhantomDedupKey::Chip with chip='different_chip'"
    );
}

// ── T23 edge case 2b — different fired_from bypasses the filter ──────────────

#[test]
fn max_active_different_fired_from_bypasses_filter() {
    let mut world = world_with_assets();
    let real_bolt = spawn_source(&mut world, Vec2::ZERO, Vec2::ZERO);
    let other = spawn_source(&mut world, Vec2::new(5.0, 5.0), Vec2::new(1.0, 1.0));

    // Pre-spawn 2 phantoms keyed for real_bolt
    spawn_keyed_phantom(&mut world, "phantom_bolt", real_bolt);
    spawn_keyed_phantom(&mut world, "phantom_bolt", real_bolt);

    let config = SpawnPhantomConfig {
        duration:   OrderedFloat(2.0),
        max_active: 2,
    };
    config.fire(other, "phantom_bolt", &mut world);
    world.flush();

    let count = world
        .query_filtered::<Entity, (With<PhantomBolt>, With<PhantomDedupKey>)>()
        .iter(&world)
        .count();
    assert_eq!(
        count, 3,
        "different fired_from must bypass the max_active filter; count should grow to 3"
    );

    // The new phantom's fired_from must be `other`
    let new_key = world.query::<&PhantomDedupKey>().iter(&world).find(
        |key| matches!(key, PhantomDedupKey::Chip { fired_from, .. } if *fired_from == other),
    );
    assert!(
        new_key.is_some(),
        "new phantom must have PhantomDedupKey::Chip with fired_from=other"
    );
}

// ── T23 edge case 2c — legacy PhantomOwner shape not counted ─────────────────

#[test]
fn max_active_ignores_legacy_phantom_owner_shape() {
    let mut world = world_with_assets();
    let real_bolt = spawn_source(&mut world, Vec2::ZERO, Vec2::ZERO);

    // Pre-spawn a legacy-shape phantom: PhantomBolt + PhantomOwner but NO PhantomDedupKey
    world.spawn((PhantomBolt, PhantomOwner(real_bolt)));

    let config = SpawnPhantomConfig {
        duration:   OrderedFloat(2.0),
        max_active: 1,
    };
    config.fire(real_bolt, "phantom_bolt", &mut world);
    world.flush();

    // New dedup-key-bearing phantom must be spawned (old legacy doesn't count)
    let dedup_count = world
        .query_filtered::<Entity, (With<PhantomBolt>, With<PhantomDedupKey>)>()
        .iter(&world)
        .count();
    assert_eq!(
        dedup_count, 1,
        "a new dedup-key-bearing phantom must be spawned; legacy PhantomOwner shape is not counted"
    );
}

// ── T23 edge case 2d — below-limit count allows spawn ────────────────────────

#[test]
fn max_active_below_limit_allows_spawn() {
    let mut world = world_with_assets();
    let real_bolt = spawn_source(&mut world, Vec2::ZERO, Vec2::ZERO);

    // Pre-spawn 1 phantom keyed for (real_bolt, "phantom_bolt"), max_active: 2
    spawn_keyed_phantom(&mut world, "phantom_bolt", real_bolt);

    let config = SpawnPhantomConfig {
        duration:   OrderedFloat(2.0),
        max_active: 2,
    };
    config.fire(real_bolt, "phantom_bolt", &mut world);
    world.flush();

    let count = world
        .query_filtered::<Entity, (With<PhantomBolt>, With<PhantomDedupKey>)>()
        .iter(&world)
        .count();
    assert_eq!(
        count, 2,
        "one pre-existing + one new = 2; spawn allowed when count < max_active"
    );
}

// ── T23 edge case 2e — max_active: 0 always skips ───────────────────────────

#[test]
fn max_active_zero_always_skips() {
    let mut world = world_with_assets();
    let real_bolt = spawn_source(&mut world, Vec2::ZERO, Vec2::ZERO);

    let config = SpawnPhantomConfig {
        duration:   OrderedFloat(2.0),
        max_active: 0,
    };
    config.fire(real_bolt, "phantom_bolt", &mut world);
    world.flush();

    let count = world
        .query_filtered::<Entity, With<PhantomBolt>>()
        .iter(&world)
        .count();
    assert_eq!(count, 0, "max_active: 0 must prevent any spawn");
}
