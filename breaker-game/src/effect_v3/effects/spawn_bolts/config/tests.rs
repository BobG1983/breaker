use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rantzsoft_spatial2d::components::BaseSpeed;

use super::config_impl::SpawnBoltsConfig;
use crate::{
    bolt::components::{BoltLifespan, ExtraBolt, PrimaryBolt},
    effect_v3::{
        effects::DamageBoostConfig,
        storage::BoundEffects,
        traits::Fireable,
        types::{EffectType, Tree},
    },
    prelude::*,
};

fn spawn_source(world: &mut World, pos: Vec2, vel: Vec2) -> Entity {
    world
        .spawn((Bolt, Position2D(pos), Velocity2D(vel), BaseSpeed(400.0)))
        .id()
}

#[test]
fn fire_spawns_count_bolts_with_extra_bolt_marker() {
    let mut world = World::new();
    let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = SpawnBoltsConfig {
        count:    3,
        lifespan: None,
        inherit:  false,
    };
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    config.fire(source, "splinter", &mut world, &mut rng);
    world.flush();

    let extra_count = world
        .query_filtered::<Entity, With<ExtraBolt>>()
        .iter(&world)
        .count();
    assert_eq!(extra_count, 3, "expected 3 ExtraBolt entities");
}

#[test]
fn fire_count_zero_spawns_no_entities() {
    let mut world = World::new();
    let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = SpawnBoltsConfig {
        count:    0,
        lifespan: None,
        inherit:  false,
    };
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    config.fire(source, "splinter", &mut world, &mut rng);
    world.flush();

    let extra_count = world
        .query_filtered::<Entity, With<ExtraBolt>>()
        .iter(&world)
        .count();
    assert_eq!(extra_count, 0, "count 0 should spawn zero entities");
}

#[test]
fn spawned_bolts_are_at_source_position() {
    let mut world = World::new();
    let source = spawn_source(&mut world, Vec2::new(150.0, 75.0), Vec2::new(0.0, 400.0));

    let config = SpawnBoltsConfig {
        count:    2,
        lifespan: None,
        inherit:  false,
    };
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    config.fire(source, "splinter", &mut world, &mut rng);
    world.flush();

    let positions: Vec<Vec2> = world
        .query_filtered::<&Position2D, With<ExtraBolt>>()
        .iter(&world)
        .map(|p| p.0)
        .collect();
    assert_eq!(positions.len(), 2);
    for pos in &positions {
        assert!(
            (*pos - Vec2::new(150.0, 75.0)).length() < 1e-3,
            "spawned bolt should be at source position, got {pos:?}",
        );
    }
}

#[test]
fn spawned_bolts_have_nonzero_velocity() {
    let mut world = World::new();
    let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = SpawnBoltsConfig {
        count:    3,
        lifespan: None,
        inherit:  false,
    };
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    config.fire(source, "splinter", &mut world, &mut rng);
    world.flush();

    let velocities: Vec<Vec2> = world
        .query_filtered::<&Velocity2D, With<ExtraBolt>>()
        .iter(&world)
        .map(|v| v.0)
        .collect();
    assert_eq!(velocities.len(), 3);
    for vel in &velocities {
        assert!(
            vel.length() > 1e-3,
            "spawned bolt should have nonzero velocity, got {vel:?}"
        );
    }
}

#[test]
fn spawned_bolts_have_lifespan_when_configured() {
    let mut world = World::new();
    let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = SpawnBoltsConfig {
        count:    2,
        lifespan: Some(OrderedFloat(3.5)),
        inherit:  false,
    };
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    config.fire(source, "splinter", &mut world, &mut rng);
    world.flush();

    let lifespans: Vec<f32> = world
        .query_filtered::<&BoltLifespan, With<ExtraBolt>>()
        .iter(&world)
        .map(|l| l.0.duration().as_secs_f32())
        .collect();
    assert_eq!(lifespans.len(), 2);
    for dur in &lifespans {
        assert!(
            (*dur - 3.5).abs() < 1e-3,
            "lifespan duration should be ~3.5s, got {dur}",
        );
    }
}

#[test]
fn spawned_bolts_have_no_lifespan_when_none() {
    let mut world = World::new();
    let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = SpawnBoltsConfig {
        count:    1,
        lifespan: None,
        inherit:  false,
    };
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    config.fire(source, "splinter", &mut world, &mut rng);
    world.flush();

    let lifespan_count = world
        .query_filtered::<&BoltLifespan, With<ExtraBolt>>()
        .iter(&world)
        .count();
    // The spawned bolt should NOT have a BoltLifespan. But first it needs
    // to exist, so we also verify ExtraBolt count.
    let extra_count = world
        .query_filtered::<Entity, With<ExtraBolt>>()
        .iter(&world)
        .count();
    assert_eq!(extra_count, 1, "should have spawned 1 ExtraBolt");
    assert_eq!(
        lifespan_count, 0,
        "lifespan: None should not add BoltLifespan"
    );
}

#[test]
fn inherit_true_copies_primary_bolt_bound_effects() {
    let mut world = World::new();

    let tree_a = Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
        multiplier: OrderedFloat(2.0),
    }));
    let source = world
        .spawn((
            Bolt,
            PrimaryBolt,
            Position2D(Vec2::new(100.0, 200.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
            BaseSpeed(400.0),
            BoundEffects(vec![("chip_a".to_string(), tree_a)]),
        ))
        .id();

    let config = SpawnBoltsConfig {
        count:    1,
        lifespan: None,
        inherit:  true,
    };
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    config.fire(source, "splinter", &mut world, &mut rng);
    world.flush();

    let inherited: Vec<&BoundEffects> = world
        .query_filtered::<&BoundEffects, With<ExtraBolt>>()
        .iter(&world)
        .collect();
    assert_eq!(inherited.len(), 1, "spawned bolt should have BoundEffects");
    assert!(
        inherited[0].0.iter().any(|(name, _)| name == "chip_a"),
        "inherited BoundEffects should contain chip_a",
    );
}

#[test]
fn inherit_false_does_not_copy_bound_effects() {
    let mut world = World::new();

    let tree_a = Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
        multiplier: OrderedFloat(2.0),
    }));
    let source = world
        .spawn((
            Bolt,
            PrimaryBolt,
            Position2D(Vec2::new(100.0, 200.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
            BaseSpeed(400.0),
            BoundEffects(vec![("chip_a".to_string(), tree_a)]),
        ))
        .id();

    let config = SpawnBoltsConfig {
        count:    1,
        lifespan: None,
        inherit:  false,
    };
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    config.fire(source, "splinter", &mut world, &mut rng);
    world.flush();

    let inherited_count = world
        .query_filtered::<&BoundEffects, With<ExtraBolt>>()
        .iter(&world)
        .count();
    assert_eq!(
        inherited_count, 0,
        "inherit: false should not copy BoundEffects"
    );
}

#[test]
fn spawned_bolts_have_bolt_and_extra_bolt_markers() {
    let mut world = World::new();
    let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = SpawnBoltsConfig {
        count:    1,
        lifespan: None,
        inherit:  false,
    };
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    config.fire(source, "splinter", &mut world, &mut rng);
    world.flush();

    let bolt_extras: Vec<Entity> = world
        .query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()
        .iter(&world)
        .collect();
    // Source has Bolt but not ExtraBolt, so only spawned entities match.
    assert_eq!(
        bolt_extras.len(),
        1,
        "spawned bolt should have both Bolt and ExtraBolt"
    );
}

// ── Deterministic spread invariants ──────────────────────────────────

/// `count == 1` short-circuits the fan formula and fires straight up
/// (angle = 0). Pinning velocity == (0.0, `base_speed`) catches any
/// regression in the special-case branch of `spread_angles`.
#[test]
fn count_one_spawns_bolt_straight_up_with_zero_x_velocity() {
    let mut world = World::new();
    let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = SpawnBoltsConfig {
        count:    1,
        lifespan: None,
        inherit:  false,
    };
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    config.fire(source, "splinter", &mut world, &mut rng);
    world.flush();

    let velocities: Vec<Vec2> = world
        .query_filtered::<&Velocity2D, With<ExtraBolt>>()
        .iter(&world)
        .map(|v| v.0)
        .collect();
    assert_eq!(velocities.len(), 1);
    // base_speed == 400.0 (set by spawn_source); angle == 0 means
    // velocity == (sin(0)*400, cos(0)*400) == (0, 400).
    assert!(
        velocities[0].x.abs() < 1e-3,
        "count==1 must fire straight up — x velocity should be 0.0, got {}",
        velocities[0].x,
    );
    assert!(
        (velocities[0].y - 400.0).abs() < 1e-3,
        "count==1 must fire straight up — y velocity should be base_speed (400.0), got {}",
        velocities[0].y,
    );
}

/// `count == 2` fires symmetrically across the upper hemisphere
/// (angles ±π/6 → ±30°). Pins the determinism contract: x velocities
/// are equal-magnitude opposite signs, y velocities are equal positive.
#[test]
fn count_two_spawns_symmetric_pair_around_vertical() {
    let mut world = World::new();
    let source = spawn_source(&mut world, Vec2::new(0.0, 0.0), Vec2::new(0.0, 400.0));

    let config = SpawnBoltsConfig {
        count:    2,
        lifespan: None,
        inherit:  false,
    };
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    config.fire(source, "splinter", &mut world, &mut rng);
    world.flush();

    let mut velocities: Vec<Vec2> = world
        .query_filtered::<&Velocity2D, With<ExtraBolt>>()
        .iter(&world)
        .map(|v| v.0)
        .collect();
    assert_eq!(velocities.len(), 2);
    // Sort by x so [left, right] order is deterministic.
    velocities.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    assert!(
        (velocities[0].x + velocities[1].x).abs() < 1e-3,
        "count==2 must produce symmetric x velocities, got {:?} and {:?}",
        velocities[0],
        velocities[1],
    );
    assert!(
        velocities[0].x < 0.0 && velocities[1].x > 0.0,
        "count==2 leftmost must be negative x, rightmost positive, got {:?} and {:?}",
        velocities[0],
        velocities[1],
    );
    assert!(
        (velocities[0].y - velocities[1].y).abs() < 1e-3,
        "count==2 must produce equal y velocities, got {} vs {}",
        velocities[0].y,
        velocities[1].y,
    );
}

#[test]
fn spawned_bolts_have_birthing_component() {
    let mut world = World::new();
    let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = SpawnBoltsConfig {
        count:    1,
        lifespan: None,
        inherit:  false,
    };
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    config.fire(source, "splinter", &mut world, &mut rng);
    world.flush();

    let birthing_count = world
        .query_filtered::<&Birthing, With<ExtraBolt>>()
        .iter(&world)
        .count();
    assert_eq!(
        birthing_count, 1,
        "spawned bolt should have Birthing component"
    );
}
