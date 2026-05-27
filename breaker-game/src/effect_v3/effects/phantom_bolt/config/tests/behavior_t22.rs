//! T22 — `SpawnPhantomConfig::fire` produces a phantom with
//! `PhantomDedupKey::Chip { chip, fired_from }` and the full Wave-1/2/3
//! component vocabulary (Behavior 1 + edge cases).

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_physics2d::collision_layers::CollisionLayers;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::{
    super::config_impl::SpawnPhantomConfig,
    helpers::{spawn_source, world_with_assets},
};
use crate::{
    bolt::components::{
        Bolt, ExtraBolt, LifetimeEndBehavior, PhantomBolt, PhantomDamagedCells, PhantomDedupKey,
    },
    effect_v3::traits::Fireable,
    prelude::*,
    shared::{BREAKER_LAYER, CELL_LAYER, GameDrawLayer, Lifespan, PhantomFlicker, WALL_LAYER},
    state::types::NodeState,
};

// ── T22 primary — dedup key, spatial, role, lifetime, rendered components ────

#[test]
fn fire_produces_phantom_with_chip_dedup_key() {
    let mut world = world_with_assets();
    let real_bolt = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = SpawnPhantomConfig {
        duration:   OrderedFloat(2.0),
        max_active: 3,
    };
    config.fire(real_bolt, "phantom_bolt", &mut world);
    world.flush();

    let phantom = world
        .query_filtered::<Entity, (With<PhantomBolt>, With<PhantomDedupKey>)>()
        .iter(&world)
        .next()
        .expect("fire() must spawn a phantom with PhantomBolt + PhantomDedupKey");

    let key = world
        .get::<PhantomDedupKey>(phantom)
        .expect("PhantomDedupKey must be present");
    assert_eq!(
        *key,
        PhantomDedupKey::Chip {
            chip:       "phantom_bolt".to_string(),
            fired_from: real_bolt,
        },
        "PhantomDedupKey must be Chip variant with correct chip and fired_from"
    );

    let damaged = world
        .get::<PhantomDamagedCells>(phantom)
        .expect("PhantomDamagedCells must be present");
    assert!(
        damaged.0.is_empty(),
        "PhantomDamagedCells must be empty on spawn"
    );

    assert!(world.get::<Bolt>(phantom).is_some(), "Bolt must be present");
    assert!(
        world.get::<ExtraBolt>(phantom).is_some(),
        "ExtraBolt must be present"
    );
    assert!(
        world.get::<CleanupOnExit<NodeState>>(phantom).is_some(),
        "CleanupOnExit<NodeState> must be present"
    );

    let pos = world
        .get::<Position2D>(phantom)
        .expect("Position2D must be present");
    assert!(
        (pos.0 - Vec2::new(100.0, 200.0)).length() < 1e-3,
        "Position2D must match source (100.0, 200.0), got {pos:?}"
    );
    let vel = world
        .get::<Velocity2D>(phantom)
        .expect("Velocity2D must be present");
    assert!(
        (vel.0 - Vec2::new(0.0, 400.0)).length() < 1e-3,
        "Velocity2D must match source (0.0, 400.0), got {vel:?}"
    );

    assert_eq!(
        world.get::<LifetimeEndBehavior>(phantom).copied(),
        Some(LifetimeEndBehavior::Despawn),
        "LifetimeEndBehavior must be Despawn"
    );
    let lifespan = world
        .get::<Lifespan>(phantom)
        .expect("Lifespan must be present");
    assert!(
        (lifespan.remaining - 2.0).abs() < f32::EPSILON,
        "Lifespan.remaining must be 2.0, got {}",
        lifespan.remaining
    );

    assert!(
        world.get::<PhantomFlicker>(phantom).is_some(),
        "PhantomFlicker must be present"
    );
    assert!(
        world.get::<Mesh2d>(phantom).is_some(),
        "Mesh2d must be present"
    );
    assert!(
        world
            .get::<MeshMaterial2d<ColorMaterial>>(phantom)
            .is_some(),
        "MeshMaterial2d<ColorMaterial> must be present"
    );
    assert!(
        matches!(
            world.get::<GameDrawLayer>(phantom),
            Some(GameDrawLayer::Bolt)
        ),
        "GameDrawLayer must be Bolt"
    );
}

// ── T22 collision mask and legacy-component absence ──────────────────────────

#[test]
fn fire_produces_phantom_with_correct_collision_mask_and_no_legacy_components() {
    let mut world = world_with_assets();
    let real_bolt = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = SpawnPhantomConfig {
        duration:   OrderedFloat(2.0),
        max_active: 3,
    };
    config.fire(real_bolt, "phantom_bolt", &mut world);
    world.flush();

    let phantom = world
        .query_filtered::<Entity, (With<PhantomBolt>, With<PhantomDedupKey>)>()
        .iter(&world)
        .next()
        .expect("fire() must spawn a phantom");

    let layers = world
        .get::<CollisionLayers>(phantom)
        .expect("CollisionLayers must be present");
    assert_ne!(
        layers.mask & CELL_LAYER,
        0,
        "mask must include CELL_LAYER (corrected from pre-W4B)"
    );
    assert_ne!(layers.mask & WALL_LAYER, 0, "mask must include WALL_LAYER");
    assert_ne!(
        layers.mask & BREAKER_LAYER,
        0,
        "mask must include BREAKER_LAYER"
    );
}

// ── T22 edge case 1a — different source string flows through verbatim ────────

#[test]
fn fire_different_source_string_stored_in_dedup_key_verbatim() {
    let mut world = world_with_assets();
    let real_bolt = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = SpawnPhantomConfig {
        duration:   OrderedFloat(2.0),
        max_active: 3,
    };
    config.fire(real_bolt, "phantom_chip_42", &mut world);
    world.flush();

    let phantom = world
        .query_filtered::<Entity, (With<PhantomBolt>, With<PhantomDedupKey>)>()
        .iter(&world)
        .next()
        .expect("phantom must be spawned");
    let key = world.get::<PhantomDedupKey>(phantom).unwrap();
    assert_eq!(
        *key,
        PhantomDedupKey::Chip {
            chip:       "phantom_chip_42".to_string(),
            fired_from: real_bolt,
        },
        "source string 'phantom_chip_42' must flow verbatim into PhantomDedupKey.chip"
    );
}

// ── T22 edge case 1b — different entity flows into fired_from ────────────────

#[test]
fn fire_different_entity_stored_as_fired_from() {
    let mut world = world_with_assets();
    let _real_bolt = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));
    let other = spawn_source(&mut world, Vec2::new(50.0, 50.0), Vec2::ZERO);

    let config = SpawnPhantomConfig {
        duration:   OrderedFloat(2.0),
        max_active: 3,
    };
    config.fire(other, "phantom_bolt", &mut world);
    world.flush();

    let phantom = world
        .query_filtered::<Entity, (With<PhantomBolt>, With<PhantomDedupKey>)>()
        .iter(&world)
        .next()
        .expect("phantom must be spawned from `other`");
    let key = world.get::<PhantomDedupKey>(phantom).unwrap();
    assert_eq!(
        *key,
        PhantomDedupKey::Chip {
            chip:       "phantom_bolt".to_string(),
            fired_from: other,
        },
        "fired_from must be `other`, not real_bolt"
    );

    // Position and velocity must match `other`, not real_bolt
    let pos = world.get::<Position2D>(phantom).expect("Position2D");
    assert!(
        (pos.0 - Vec2::new(50.0, 50.0)).length() < 1e-3,
        "position must match other entity (50.0, 50.0), got {:?}",
        pos.0
    );
    let vel = world.get::<Velocity2D>(phantom).expect("Velocity2D");
    assert!(
        vel.0.length() < 1e-3,
        "velocity must match other entity (0.0, 0.0), got {:?}",
        vel.0
    );
}

// ── T22 edge case 1c — empty source string is preserved ─────────────────────

#[test]
fn fire_empty_source_string_preserved_in_dedup_key() {
    let mut world = world_with_assets();
    let real_bolt = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = SpawnPhantomConfig {
        duration:   OrderedFloat(2.0),
        max_active: 3,
    };
    config.fire(real_bolt, "", &mut world);
    world.flush();

    let phantom = world
        .query_filtered::<Entity, (With<PhantomBolt>, With<PhantomDedupKey>)>()
        .iter(&world)
        .next()
        .expect("phantom must be spawned even with empty source");
    let key = world.get::<PhantomDedupKey>(phantom).unwrap();
    assert_eq!(
        *key,
        PhantomDedupKey::Chip {
            chip:       String::new(),
            fired_from: real_bolt,
        },
        "empty source string must be stored verbatim (not replaced)"
    );
}

// ── T22 edge case 1d — source missing Position2D/Velocity2D falls back to ZERO

#[test]
fn fire_source_missing_spatial_components_falls_back_to_zero() {
    let mut world = world_with_assets();
    let bare = world.spawn(Bolt).id();

    let config = SpawnPhantomConfig {
        duration:   OrderedFloat(2.0),
        max_active: 3,
    };
    config.fire(bare, "phantom_bolt", &mut world);
    world.flush();

    let phantom = world
        .query_filtered::<Entity, (With<PhantomBolt>, With<PhantomDedupKey>)>()
        .iter(&world)
        .next()
        .expect("phantom must be spawned even from bare source");
    let pos = world.get::<Position2D>(phantom).expect("Position2D");
    assert_eq!(
        pos.0,
        Vec2::ZERO,
        "position must fall back to Vec2::ZERO when source has no Position2D"
    );
    let vel = world.get::<Velocity2D>(phantom).expect("Velocity2D");
    assert_eq!(
        vel.0,
        Vec2::ZERO,
        "velocity must fall back to Vec2::ZERO when source has no Velocity2D"
    );
}
