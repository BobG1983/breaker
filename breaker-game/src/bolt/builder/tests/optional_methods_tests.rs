use bevy::{ecs::world::CommandQueue, prelude::*};
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::{PreviousScale, Scale2D, Velocity2D};

use crate::{
    bolt::{
        components::{Bolt, BoltLifespan, BoltRadius, SpawnedByEvolution},
        definition::BoltDefinition,
    },
    effect::EffectKind,
};

/// Creates a `BoltDefinition` matching the values previously provided by
/// `BoltConfig::default()`, so existing assertions remain valid.
fn test_bolt_definition() -> BoltDefinition {
    BoltDefinition {
        name: "Bolt".to_string(),
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
    }
}

/// Spawns a bolt via Commands backed by a `CommandQueue`, then applies the queue.
/// Returns the Entity.
fn spawn_bolt_in_world(
    world: &mut World,
    build_fn: impl FnOnce(&mut Commands) -> Entity,
) -> Entity {
    let mut queue = CommandQueue::default();
    let entity = {
        let mut commands = Commands::new(&mut queue, world);
        build_fn(&mut commands)
    };
    queue.apply(world);
    entity
}

// ── Section D: Optional Chainable Methods ───────────────────────────

// Behavior 12: .spawned_by() stores evolution attribution
#[test]
fn spawned_by_stores_evolution_name() {
    let mut world = World::new();
    let def = test_bolt_definition();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .spawned_by("mirror_protocol")
            .headless()
            .spawn(commands)
    });
    let spawned_by = world
        .get::<SpawnedByEvolution>(entity)
        .expect("entity should have SpawnedByEvolution");
    assert_eq!(
        spawned_by.0, "mirror_protocol",
        "SpawnedByEvolution should be 'mirror_protocol'"
    );
}

#[test]
fn spawned_by_empty_string_accepted() {
    let mut world = World::new();
    let def = test_bolt_definition();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .spawned_by("")
            .headless()
            .spawn(commands)
    });
    let spawned_by = world
        .get::<SpawnedByEvolution>(entity)
        .expect("entity should have SpawnedByEvolution");
    assert_eq!(
        spawned_by.0, "",
        "SpawnedByEvolution should be empty string"
    );
}

// Behavior 13: .with_lifespan() stores bolt lifespan timer
#[test]
fn with_lifespan_stores_timer() {
    let mut world = World::new();
    let def = test_bolt_definition();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_lifespan(3.5)
            .headless()
            .spawn(commands)
    });
    let lifespan = world
        .get::<BoltLifespan>(entity)
        .expect("entity should have BoltLifespan");
    assert!(
        (lifespan.0.duration().as_secs_f32() - 3.5).abs() < 1e-3,
        "BoltLifespan timer duration should be ~3.5, got {}",
        lifespan.0.duration().as_secs_f32()
    );
}

#[test]
fn with_lifespan_zero_produces_zero_duration() {
    let mut world = World::new();
    let def = test_bolt_definition();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_lifespan(0.0)
            .headless()
            .spawn(commands)
    });
    let lifespan = world
        .get::<BoltLifespan>(entity)
        .expect("entity should have BoltLifespan");
    assert!(
        lifespan.0.duration().as_secs_f32().abs() < 1e-3,
        "BoltLifespan with 0.0 should have zero duration, got {}",
        lifespan.0.duration().as_secs_f32()
    );
}

// Behavior 14: .with_radius() overrides config-provided radius
#[test]
fn with_radius_overrides_config_radius() {
    let mut world = World::new();
    let def = test_bolt_definition();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_radius(16.0)
            .headless()
            .spawn(commands)
    });
    let radius = world.get::<BoltRadius>(entity).unwrap();
    assert!(
        (radius.0 - 16.0).abs() < f32::EPSILON,
        "BoltRadius should be 16.0 (overridden), not 8.0, got {}",
        radius.0
    );
    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 16.0).abs() < f32::EPSILON && (scale.y - 16.0).abs() < f32::EPSILON,
        "Scale2D should be (16.0, 16.0), got ({}, {})",
        scale.x,
        scale.y
    );
    let prev_scale = world.get::<PreviousScale>(entity).unwrap();
    assert!(
        (prev_scale.x - 16.0).abs() < f32::EPSILON && (prev_scale.y - 16.0).abs() < f32::EPSILON,
        "PreviousScale should be (16.0, 16.0), got ({}, {})",
        prev_scale.x,
        prev_scale.y
    );
    let aabb = world.get::<Aabb2D>(entity).unwrap();
    assert!(
        (aabb.half_extents.x - 16.0).abs() < f32::EPSILON
            && (aabb.half_extents.y - 16.0).abs() < f32::EPSILON,
        "Aabb2D half_extents should be (16.0, 16.0), got {:?}",
        aabb.half_extents
    );
}

#[test]
fn with_radius_small_value() {
    let mut world = World::new();
    let def = test_bolt_definition();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_radius(0.5)
            .headless()
            .spawn(commands)
    });
    let radius = world.get::<BoltRadius>(entity).unwrap();
    assert!(
        (radius.0 - 0.5).abs() < f32::EPSILON,
        "BoltRadius should be 0.5, got {}",
        radius.0
    );
}

// Behavior 15: .with_effects() stores effect nodes as BoundEffects
#[test]
fn with_effects_stores_bound_effects() {
    use crate::effect::BoundEffects;

    let mut world = World::new();
    let effects = vec![(
        "chip_a".to_string(),
        crate::effect::EffectNode::Do(EffectKind::DamageBoost(5.0)),
    )];
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&test_bolt_definition())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_effects(effects)
            .headless()
            .spawn(commands)
    });
    let bound = world
        .get::<BoundEffects>(entity)
        .expect("entity should have BoundEffects");
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should have 1 entry, got {}",
        bound.0.len()
    );
    assert_eq!(
        bound.0[0].0, "chip_a",
        "first entry chip name should be 'chip_a'"
    );
}

#[test]
fn with_effects_empty_vec_inserts_empty_bound_effects() {
    use crate::effect::BoundEffects;

    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&test_bolt_definition())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_effects(vec![])
            .headless()
            .spawn(commands)
    });
    let bound = world
        .get::<BoundEffects>(entity)
        .expect("entity should have BoundEffects with empty vec");
    assert!(
        bound.0.is_empty(),
        "BoundEffects should be empty, got {} entries",
        bound.0.len()
    );
}

// Behavior 16: .with_effects() and .with_inherited_effects() combine
#[test]
fn with_effects_and_inherited_effects_combine() {
    use crate::effect::{BoundEffects, EffectNode};

    let node_a = EffectNode::Do(EffectKind::DamageBoost(5.0));
    let node_b = EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 });
    let inherited = BoundEffects(vec![("chip_b".to_string(), node_b)]);

    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&test_bolt_definition())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_effects(vec![("chip_a".to_string(), node_a)])
            .with_inherited_effects(&inherited)
            .headless()
            .spawn(commands)
    });

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("entity should have BoundEffects");
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should have 2 entries (explicit + inherited), got {}",
        bound.0.len()
    );
    // Explicit effects first, inherited appended
    assert_eq!(
        bound.0[0].0, "chip_a",
        "first entry should be explicit 'chip_a'"
    );
    assert_eq!(
        bound.0[1].0, "chip_b",
        "second entry should be inherited 'chip_b'"
    );
}

#[test]
fn inherited_effects_before_with_effects_same_result() {
    use crate::effect::{BoundEffects, EffectNode};

    let node_a = EffectNode::Do(EffectKind::DamageBoost(5.0));
    let node_b = EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 });
    let inherited = BoundEffects(vec![("chip_b".to_string(), node_b)]);

    let mut world = World::new();
    // Order reversed: with_inherited_effects BEFORE with_effects
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&test_bolt_definition())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_inherited_effects(&inherited)
            .with_effects(vec![("chip_a".to_string(), node_a)])
            .headless()
            .spawn(commands)
    });

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("entity should have BoundEffects");
    assert_eq!(bound.0.len(), 2);
    // Same result regardless of call order: explicit first, inherited appended
    assert_eq!(bound.0[0].0, "chip_a", "explicit always first");
    assert_eq!(bound.0[1].0, "chip_b", "inherited always second");
}

// Behavior 17: Optional methods can be called in any order
#[test]
fn optional_methods_any_order() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .spawned_by("test")
            .with_lifespan(2.0)
            .with_radius(10.0)
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .extra()
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<SpawnedByEvolution>(entity).is_some(),
        "SpawnedByEvolution should be present"
    );
    assert!(
        world.get::<BoltLifespan>(entity).is_some(),
        "BoltLifespan should be present"
    );
    let radius = world.get::<BoltRadius>(entity).unwrap();
    assert!(
        (radius.0 - 10.0).abs() < f32::EPSILON,
        "BoltRadius should be 10.0, got {}",
        radius.0
    );
}

#[test]
fn no_optional_methods_defaults() {
    let mut world = World::new();
    let def = test_bolt_definition();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<SpawnedByEvolution>(entity).is_none(),
        "SpawnedByEvolution should NOT be present when not called"
    );
    assert!(
        world.get::<BoltLifespan>(entity).is_none(),
        "BoltLifespan should NOT be present when not called"
    );
    // BoltRadius should still be present from from_config with default 8.0
    let radius = world.get::<BoltRadius>(entity).unwrap();
    assert!(
        (radius.0 - 8.0).abs() < f32::EPSILON,
        "BoltRadius should default to 8.0, got {}",
        radius.0
    );
}

// Behavior 18: Optional methods available regardless of typestate
#[test]
fn optional_methods_available_in_initial_state() {
    // This test verifies that optional methods compile from any state.
    let _builder = Bolt::builder()
        .spawned_by("test")
        .with_lifespan(1.0)
        .with_radius(5.0)
        .with_effects(vec![]);
}
