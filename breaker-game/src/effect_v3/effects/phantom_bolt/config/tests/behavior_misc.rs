//! Behaviors 4–7 — collision mask correction, rendered terminal, source
//! verbatim, and `FireEffectCommand` dispatch chain.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_physics2d::collision_layers::CollisionLayers;

use super::{
    super::config_impl::SpawnPhantomConfig,
    helpers::{spawn_source, world_with_assets},
};
use crate::{
    bolt::components::{PhantomBolt, PhantomDedupKey},
    effect_v3::{commands::FireEffectCommand, traits::Fireable, types::EffectType},
    shared::{BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, GameDrawLayer, PhantomFlicker, WALL_LAYER},
};

// ── Behavior 4 — CELL_LAYER is now included in the collision mask ─────────────

#[test]
fn spawned_phantom_collision_mask_includes_cell_layer() {
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
        .expect("phantom must be spawned");

    let layers = world
        .get::<CollisionLayers>(phantom)
        .expect("CollisionLayers must be present");

    assert_ne!(
        layers.mask & CELL_LAYER,
        0,
        "mask must include CELL_LAYER (corrected from pre-W4B omission)"
    );
    assert_ne!(layers.mask & WALL_LAYER, 0, "mask must include WALL_LAYER");
    assert_ne!(
        layers.mask & BREAKER_LAYER,
        0,
        "mask must include BREAKER_LAYER"
    );
    assert_eq!(
        layers.mask & BOLT_LAYER,
        0,
        "mask must NOT include BOLT_LAYER (no bolt-bolt collision)"
    );
}

// ── Behavior 4 edge case 4a — membership stays BOLT_LAYER ────────────────────

#[test]
fn spawned_phantom_collision_membership_is_bolt_layer() {
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
        .expect("phantom must be spawned");

    let layers = world
        .get::<CollisionLayers>(phantom)
        .expect("CollisionLayers must be present");

    assert_eq!(
        layers.membership, BOLT_LAYER,
        "phantom membership must be BOLT_LAYER (it is a bolt for collision-source purposes)"
    );
}

// ── Behavior 5 — Rendered terminal is used ───────────────────────────────────

#[test]
fn spawned_phantom_carries_rendered_terminal_components() {
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
        .expect("phantom must be spawned");

    assert!(
        world.get::<Mesh2d>(phantom).is_some(),
        "phantom must carry Mesh2d (Rendered terminal)"
    );
    assert!(
        world
            .get::<MeshMaterial2d<ColorMaterial>>(phantom)
            .is_some(),
        "phantom must carry MeshMaterial2d<ColorMaterial> (Rendered terminal)"
    );
    assert!(
        matches!(
            world.get::<GameDrawLayer>(phantom),
            Some(GameDrawLayer::Bolt)
        ),
        "phantom must carry GameDrawLayer::Bolt"
    );
    assert!(
        world.get::<PhantomFlicker>(phantom).is_some(),
        "phantom must carry PhantomFlicker (Rendered terminal inserts when phantom.is_some())"
    );
}

// ── Behavior 5 edge case 5a — Assets grow by exactly 1 each ─────────────────

#[test]
fn fire_adds_exactly_one_mesh_and_one_material() {
    let mut world = world_with_assets();
    let real_bolt = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let mesh_count_before = world.resource::<Assets<Mesh>>().iter().count();
    let material_count_before = world.resource::<Assets<ColorMaterial>>().iter().count();

    let config = SpawnPhantomConfig {
        duration:   OrderedFloat(2.0),
        max_active: 3,
    };
    config.fire(real_bolt, "phantom_bolt", &mut world);
    world.flush();

    let mesh_count_after = world.resource::<Assets<Mesh>>().iter().count();
    let material_count_after = world.resource::<Assets<ColorMaterial>>().iter().count();

    assert_eq!(
        mesh_count_after - mesh_count_before,
        1,
        "fire() must add exactly 1 mesh to Assets<Mesh>"
    );
    assert_eq!(
        material_count_after - material_count_before,
        1,
        "fire() must add exactly 1 material to Assets<ColorMaterial>"
    );
}

// ── Behavior 6 — source string is stored verbatim, no transformation ─────────

#[test]
fn source_string_pascalcase_preserved_verbatim() {
    let mut world = world_with_assets();
    let real_bolt = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = SpawnPhantomConfig {
        duration:   OrderedFloat(2.0),
        max_active: 3,
    };
    config.fire(real_bolt, "PhantomBolt", &mut world);
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
            chip:       "PhantomBolt".to_string(),
            fired_from: real_bolt,
        },
        "PascalCase 'PhantomBolt' must be stored verbatim, not lowercased or transformed"
    );
}

// ── Behavior 6 edge case 6a — unicode/whitespace preserved ───────────────────

#[test]
fn source_string_unicode_and_whitespace_preserved_verbatim() {
    let mut world = world_with_assets();
    let real_bolt = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = SpawnPhantomConfig {
        duration:   OrderedFloat(2.0),
        max_active: 3,
    };
    config.fire(real_bolt, "phantom bolt 漢", &mut world);
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
            chip:       "phantom bolt 漢".to_string(),
            fired_from: real_bolt,
        },
        "unicode and whitespace must be preserved verbatim in PhantomDedupKey.chip"
    );
}

// ── Behavior 7 — FireEffectCommand dispatch chain reaches new impl ─────────────

#[test]
fn fire_effect_command_routes_to_spawn_phantom_config() {
    let mut world = world_with_assets();
    let real_bolt = spawn_source(&mut world, Vec2::ZERO, Vec2::ZERO);

    FireEffectCommand {
        entity: real_bolt,
        effect: EffectType::SpawnPhantom(SpawnPhantomConfig {
            duration:   OrderedFloat(2.0),
            max_active: 3,
        }),
        source: "phantom_bolt".to_string(),
    }
    .apply(&mut world);
    world.flush();

    let phantom = world
        .query_filtered::<Entity, (With<PhantomBolt>, With<PhantomDedupKey>)>()
        .iter(&world)
        .next()
        .expect("phantom must be spawned via FireEffectCommand dispatch chain");

    let key = world.get::<PhantomDedupKey>(phantom).unwrap();
    assert_eq!(
        *key,
        PhantomDedupKey::Chip {
            chip:       "phantom_bolt".to_string(),
            fired_from: real_bolt,
        },
        "dispatch chain must route to new SpawnPhantomConfig::fire impl"
    );
}

// ── Behavior 7 edge case 7a — source from FireEffectCommand flows into dedup key

#[test]
fn fire_effect_command_source_string_is_the_dedup_chip_not_variant_name() {
    let mut world = world_with_assets();
    let real_bolt = spawn_source(&mut world, Vec2::ZERO, Vec2::ZERO);

    FireEffectCommand {
        entity: real_bolt,
        effect: EffectType::SpawnPhantom(SpawnPhantomConfig {
            duration:   OrderedFloat(2.0),
            max_active: 3,
        }),
        source: "custom_source_xyz".to_string(),
    }
    .apply(&mut world);
    world.flush();

    let phantom = world
        .query_filtered::<Entity, (With<PhantomBolt>, With<PhantomDedupKey>)>()
        .iter(&world)
        .next()
        .expect("phantom must be spawned");

    let key = world.get::<PhantomDedupKey>(phantom).unwrap();
    match key {
        PhantomDedupKey::Chip { chip, .. } => {
            assert_eq!(
                chip, "custom_source_xyz",
                "chip must be 'custom_source_xyz' from FireEffectCommand.source, not 'SpawnPhantom'"
            );
        }
        PhantomDedupKey::Bolt(_) => panic!("expected Chip variant, got {key:?}"),
    }
}
