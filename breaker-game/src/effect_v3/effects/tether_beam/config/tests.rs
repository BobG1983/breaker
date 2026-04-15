use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::{BaseSpeed, Position2D, Velocity2D};

use super::config_impl::*;
use crate::{
    bolt::components::{Bolt, ExtraBolt},
    effect_v3::{
        components::EffectSourceChip,
        effects::tether_beam::components::{TetherBeamDamage, TetherBeamSource, TetherBeamWidth},
        traits::Fireable,
    },
    shared::{birthing::Birthing, rng::GameRng},
};

fn spawn_source(world: &mut World, pos: Vec2, vel: Vec2) -> Entity {
    world
        .spawn((Bolt, Position2D(pos), Velocity2D(vel), BaseSpeed(400.0)))
        .id()
}

#[test]
fn fire_spawns_tether_beam_source_entity() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = TetherBeamConfig {
        damage_mult: OrderedFloat(1.5),
        chain:       false,
        width:       OrderedFloat(10.0),
    };
    config.fire(source, "tether_beam", &mut world);
    world.flush();

    let beam_count = world
        .query_filtered::<Entity, With<TetherBeamSource>>()
        .iter(&world)
        .count();
    assert!(
        beam_count >= 1,
        "should spawn at least 1 TetherBeamSource entity"
    );
}

#[test]
fn tether_beam_source_references_source_as_bolt_a() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = TetherBeamConfig {
        damage_mult: OrderedFloat(1.5),
        chain:       false,
        width:       OrderedFloat(10.0),
    };
    config.fire(source, "tether_beam", &mut world);
    world.flush();

    let beams: Vec<&TetherBeamSource> = world.query::<&TetherBeamSource>().iter(&world).collect();
    assert_eq!(beams.len(), 1);
    assert_eq!(
        beams[0].bolt_a, source,
        "bolt_a should be the source entity"
    );
}

#[test]
fn chain_false_spawns_new_bolt_and_connects_beam() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = TetherBeamConfig {
        damage_mult: OrderedFloat(1.5),
        chain:       false,
        width:       OrderedFloat(10.0),
    };
    config.fire(source, "tether_beam", &mut world);
    world.flush();

    // A new ExtraBolt should exist
    let extra_bolts: Vec<Entity> = world
        .query_filtered::<Entity, With<ExtraBolt>>()
        .iter(&world)
        .collect();
    assert_eq!(
        extra_bolts.len(),
        1,
        "chain: false should spawn a new ExtraBolt"
    );

    // The beam's bolt_b should point to the new bolt
    let beams: Vec<&TetherBeamSource> = world.query::<&TetherBeamSource>().iter(&world).collect();
    assert_eq!(beams.len(), 1);
    assert_eq!(
        beams[0].bolt_b, extra_bolts[0],
        "bolt_b should be the newly spawned ExtraBolt",
    );
}

#[test]
fn tether_beam_damage_equals_damage_mult_directly() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = TetherBeamConfig {
        damage_mult: OrderedFloat(2.5),
        chain:       false,
        width:       OrderedFloat(10.0),
    };
    config.fire(source, "tether_beam", &mut world);
    world.flush();

    let damages: Vec<f32> = world
        .query::<&TetherBeamDamage>()
        .iter(&world)
        .map(|d| d.0)
        .collect();
    assert_eq!(damages.len(), 1);
    assert!(
        (damages[0] - 2.5).abs() < 1e-3,
        "TetherBeamDamage should be 2.5 (direct config value), got {}",
        damages[0],
    );
}

#[test]
fn tether_beam_source_entity_is_not_a_bolt() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = TetherBeamConfig {
        damage_mult: OrderedFloat(1.5),
        chain:       false,
        width:       OrderedFloat(10.0),
    };
    config.fire(source, "tether_beam", &mut world);
    world.flush();

    let beam_bolts = world
        .query_filtered::<Entity, (With<TetherBeamSource>, With<Bolt>)>()
        .iter(&world)
        .count();
    assert_eq!(
        beam_bolts, 0,
        "TetherBeamSource entity should NOT have Bolt marker"
    );
}

#[test]
fn chain_false_spawned_bolt_has_birthing_component() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = TetherBeamConfig {
        damage_mult: OrderedFloat(1.5),
        chain:       false,
        width:       OrderedFloat(10.0),
    };
    config.fire(source, "tether_beam", &mut world);
    world.flush();

    let birthing_count = world
        .query_filtered::<&Birthing, With<ExtraBolt>>()
        .iter(&world)
        .count();
    assert_eq!(
        birthing_count, 1,
        "spawned bolt (bolt_b) should have Birthing component"
    );
}

// ── Group D — fire() spawn-time chip attachment ────────────────────────

#[test]
fn fire_spawn_with_non_empty_source_attaches_chip_some() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = TetherBeamConfig {
        damage_mult: OrderedFloat(1.5),
        chain:       false,
        width:       OrderedFloat(10.0),
    };
    config.fire(source, "coil_chip", &mut world);
    world.flush();

    let chips: Vec<Option<String>> = world
        .query_filtered::<&EffectSourceChip, With<TetherBeamSource>>()
        .iter(&world)
        .map(|c| c.0.clone())
        .collect();
    assert_eq!(chips.len(), 1, "exactly 1 TetherBeamSource entity expected");
    assert_eq!(chips[0], Some("coil_chip".to_string()));

    // The spawned ExtraBolt must NOT carry an EffectSourceChip.
    let extra_bolt_chip_count = world
        .query_filtered::<&EffectSourceChip, With<ExtraBolt>>()
        .iter(&world)
        .count();
    assert_eq!(
        extra_bolt_chip_count, 0,
        "EffectSourceChip must be on the TetherBeamSource entity, not the ExtraBolt"
    );
}

#[test]
fn fire_spawn_with_empty_source_attaches_chip_none() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = TetherBeamConfig {
        damage_mult: OrderedFloat(1.5),
        chain:       false,
        width:       OrderedFloat(10.0),
    };
    config.fire(source, "", &mut world);
    world.flush();

    let chips: Vec<Option<String>> = world
        .query_filtered::<&EffectSourceChip, With<TetherBeamSource>>()
        .iter(&world)
        .map(|c| c.0.clone())
        .collect();
    assert_eq!(chips.len(), 1);
    assert_eq!(
        chips[0], None,
        "empty source string must map to EffectSourceChip(None)"
    );
}

#[test]
fn fire_chain_with_non_empty_source_attaches_chip_some() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let source = spawn_source(&mut world, Vec2::new(0.0, 0.0), Vec2::new(0.0, 400.0));
    let _other = spawn_source(&mut world, Vec2::new(50.0, 0.0), Vec2::new(0.0, 400.0));

    let config = TetherBeamConfig {
        damage_mult: OrderedFloat(1.5),
        chain:       true,
        width:       OrderedFloat(10.0),
    };
    config.fire(source, "coil_chip", &mut world);
    world.flush();

    let chips: Vec<Option<String>> = world
        .query_filtered::<&EffectSourceChip, With<TetherBeamSource>>()
        .iter(&world)
        .map(|c| c.0.clone())
        .collect();
    assert_eq!(chips.len(), 1, "exactly 1 TetherBeamSource entity expected");
    assert_eq!(chips[0], Some("coil_chip".to_string()));
}

#[test]
fn fire_chain_with_empty_source_attaches_chip_none() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let source = spawn_source(&mut world, Vec2::new(0.0, 0.0), Vec2::new(0.0, 400.0));
    let _other = spawn_source(&mut world, Vec2::new(50.0, 0.0), Vec2::new(0.0, 400.0));

    let config = TetherBeamConfig {
        damage_mult: OrderedFloat(1.5),
        chain:       true,
        width:       OrderedFloat(10.0),
    };
    config.fire(source, "", &mut world);
    world.flush();

    let chips: Vec<Option<String>> = world
        .query_filtered::<&EffectSourceChip, With<TetherBeamSource>>()
        .iter(&world)
        .map(|c| c.0.clone())
        .collect();
    assert_eq!(chips.len(), 1);
    assert_eq!(chips[0], None);
}

// ── Group E — fire_chain target selection ──────────────────────────────

#[test]
fn fire_chain_picks_nearest_other_bolt_by_squared_distance() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let source = spawn_source(&mut world, Vec2::new(0.0, 0.0), Vec2::new(0.0, 400.0));
    let nearest_bolt_entity = spawn_source(&mut world, Vec2::new(10.0, 0.0), Vec2::new(0.0, 400.0));
    let _mid = spawn_source(&mut world, Vec2::new(50.0, 0.0), Vec2::new(0.0, 400.0));
    let _far = spawn_source(&mut world, Vec2::new(200.0, 0.0), Vec2::new(0.0, 400.0));

    let config = TetherBeamConfig {
        damage_mult: OrderedFloat(1.5),
        chain:       true,
        width:       OrderedFloat(10.0),
    };
    config.fire(source, "coil_chip", &mut world);
    world.flush();

    let beams: Vec<TetherBeamSource> = world
        .query::<&TetherBeamSource>()
        .iter(&world)
        .cloned()
        .collect();
    assert_eq!(beams.len(), 1);
    assert_eq!(beams[0].bolt_a, source);
    assert_eq!(
        beams[0].bolt_b, nearest_bolt_entity,
        "chain target must be the nearest other bolt"
    );
}

#[test]
fn fire_chain_with_only_source_bolt_is_noop() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let source = spawn_source(&mut world, Vec2::new(0.0, 0.0), Vec2::new(0.0, 400.0));

    let config = TetherBeamConfig {
        damage_mult: OrderedFloat(1.5),
        chain:       true,
        width:       OrderedFloat(10.0),
    };
    config.fire(source, "coil_chip", &mut world);
    world.flush();

    let beam_count = world.query::<&TetherBeamSource>().iter(&world).count();
    assert_eq!(
        beam_count, 0,
        "no beam should be spawned with only the source bolt"
    );

    let chip_count = world
        .query_filtered::<&EffectSourceChip, With<TetherBeamSource>>()
        .iter(&world)
        .count();
    assert_eq!(
        chip_count, 0,
        "no EffectSourceChip should be spawned either"
    );
}

#[test]
fn fire_chain_with_two_equidistant_bolts_spawns_exactly_one_beam() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let source = spawn_source(&mut world, Vec2::new(0.0, 0.0), Vec2::new(0.0, 400.0));
    let right_bolt = spawn_source(&mut world, Vec2::new(50.0, 0.0), Vec2::new(0.0, 400.0));
    let left_bolt = spawn_source(&mut world, Vec2::new(-50.0, 0.0), Vec2::new(0.0, 400.0));

    let config = TetherBeamConfig {
        damage_mult: OrderedFloat(1.5),
        chain:       true,
        width:       OrderedFloat(10.0),
    };
    config.fire(source, "coil_chip", &mut world);
    world.flush();

    let beams: Vec<TetherBeamSource> = world
        .query::<&TetherBeamSource>()
        .iter(&world)
        .cloned()
        .collect();
    assert_eq!(beams.len(), 1, "exactly 1 beam should be spawned");
    assert_eq!(beams[0].bolt_a, source);
    assert!(
        beams[0].bolt_b == left_bolt || beams[0].bolt_b == right_bolt,
        "beam target must be one of the two equidistant bolts"
    );
}

// ── Group B (width) — fire_spawn/fire_chain stamp TetherBeamWidth ──────

/// Behavior 8: `fire_spawn` (chain=false) stamps `TetherBeamWidth` on
/// the `TetherBeamSource` entity with the exact config value.
#[test]
fn fire_spawn_stamps_tether_beam_width_from_config() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = TetherBeamConfig {
        damage_mult: OrderedFloat(1.5),
        chain:       false,
        width:       OrderedFloat(7.25),
    };
    config.fire(source, "tether_beam", &mut world);
    world.flush();

    let widths: Vec<f32> = world
        .query_filtered::<&TetherBeamWidth, With<TetherBeamSource>>()
        .iter(&world)
        .map(|w| w.0)
        .collect();
    assert_eq!(
        widths.len(),
        1,
        "exactly 1 TetherBeamSource entity must carry TetherBeamWidth"
    );
    assert!(
        (widths[0] - 7.25).abs() < 1e-6,
        "TetherBeamWidth must equal config.width.0 (7.25), got {}",
        widths[0]
    );

    // The spawned ExtraBolt must NOT receive a TetherBeamWidth.
    let extra_bolt_widths = world
        .query_filtered::<&TetherBeamWidth, With<ExtraBolt>>()
        .iter(&world)
        .count();
    assert_eq!(
        extra_bolt_widths, 0,
        "TetherBeamWidth must be on the TetherBeamSource entity, not the spawned ExtraBolt"
    );
}

/// Behavior 9: `fire_chain` (chain=true) stamps `TetherBeamWidth` on
/// the `TetherBeamSource` entity with the exact config value.
#[test]
fn fire_chain_stamps_tether_beam_width_from_config() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let source = spawn_source(&mut world, Vec2::new(0.0, 0.0), Vec2::new(0.0, 400.0));
    let _other = spawn_source(&mut world, Vec2::new(50.0, 0.0), Vec2::new(0.0, 400.0));

    let config = TetherBeamConfig {
        damage_mult: OrderedFloat(1.5),
        chain:       true,
        width:       OrderedFloat(12.0),
    };
    config.fire(source, "coil_chip", &mut world);
    world.flush();

    let widths: Vec<f32> = world
        .query_filtered::<&TetherBeamWidth, With<TetherBeamSource>>()
        .iter(&world)
        .map(|w| w.0)
        .collect();
    assert_eq!(widths.len(), 1);
    assert!(
        (widths[0] - 12.0).abs() < 1e-6,
        "fire_chain must stamp TetherBeamWidth(12.0), got {}",
        widths[0]
    );
}

/// Behavior 9 edge case: when `fire_chain` is a no-op (only source
/// bolt in world), NO `TetherBeamWidth` component is spawned either.
#[test]
fn fire_chain_noop_does_not_spawn_tether_beam_width() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let source = spawn_source(&mut world, Vec2::new(0.0, 0.0), Vec2::new(0.0, 400.0));

    let config = TetherBeamConfig {
        damage_mult: OrderedFloat(1.5),
        chain:       true,
        width:       OrderedFloat(12.0),
    };
    config.fire(source, "coil_chip", &mut world);
    world.flush();

    let width_count = world
        .query_filtered::<&TetherBeamWidth, With<TetherBeamSource>>()
        .iter(&world)
        .count();
    assert_eq!(
        width_count, 0,
        "fire_chain no-op (only source bolt) must not spawn TetherBeamWidth"
    );
}

/// Behavior 10: `fire_spawn` with width 0.0 stamps
/// `TetherBeamWidth`(0.0) verbatim — no clamping or defaulting to 10.0.
#[test]
fn fire_spawn_with_width_zero_stamps_zero_verbatim() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = TetherBeamConfig {
        damage_mult: OrderedFloat(1.5),
        chain:       false,
        width:       OrderedFloat(0.0),
    };
    config.fire(source, "tether_beam", &mut world);
    world.flush();

    let widths: Vec<f32> = world
        .query_filtered::<&TetherBeamWidth, With<TetherBeamSource>>()
        .iter(&world)
        .map(|w| w.0)
        .collect();
    assert_eq!(widths.len(), 1);
    assert!(
        widths[0].abs() < 1e-6,
        "TetherBeamWidth must be exactly 0.0 — no clamp/floor/default, got {}",
        widths[0]
    );
}

/// Behavior 11: `fire_spawn` with width 1000.0 stamps
/// `TetherBeamWidth` verbatim — no upper clamp.
#[test]
fn fire_spawn_with_width_large_stamps_verbatim() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

    let config = TetherBeamConfig {
        damage_mult: OrderedFloat(1.5),
        chain:       false,
        width:       OrderedFloat(1000.0),
    };
    config.fire(source, "tether_beam", &mut world);
    world.flush();

    let widths: Vec<f32> = world
        .query_filtered::<&TetherBeamWidth, With<TetherBeamSource>>()
        .iter(&world)
        .map(|w| w.0)
        .collect();
    assert_eq!(widths.len(), 1);
    assert!(
        (widths[0] - 1000.0).abs() < 1e-3,
        "TetherBeamWidth must be 1000.0, got {}",
        widths[0]
    );
}
