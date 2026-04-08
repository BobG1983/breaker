use bevy::{ecs::world::CommandQueue, prelude::*};
use rantzsoft_physics2d::collision_layers::CollisionLayers;
use rantzsoft_spatial2d::components::{
    BaseSpeed, InterpolateTransform2D, MaxSpeed, MinSpeed, Position2D, Spatial, Spatial2D,
    Velocity2D,
};
use rantzsoft_stateflow::CleanupOnExit;

use crate::{
    bolt::{
        components::{
            Bolt, BoltLifespan, BoltRadius, BoltServing, ExtraBolt, PrimaryBolt, SpawnedByEvolution,
        },
        definition::BoltDefinition,
    },
    effect::{BoundEffects, EffectKind, EffectNode},
    shared::GameDrawLayer,
    state::types::{NodeState, RunState},
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

// ── Section F: spawn(&mut Commands) -> Entity ──────────────────────────

// Behavior 27: spawn() creates entity with all build components
#[test]
fn spawn_primary_serving_creates_complete_entity() {
    let mut world = World::new();
    let def = test_bolt_definition();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::new(0.0, 50.0))
            .serving()
            .primary()
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get_entity(entity).is_ok(),
        "spawned entity should exist"
    );
    assert!(world.get::<Bolt>(entity).is_some(), "should have Bolt");
    assert!(
        world.get::<PrimaryBolt>(entity).is_some(),
        "should have PrimaryBolt"
    );
    assert!(
        world.get::<BoltServing>(entity).is_some(),
        "should have BoltServing"
    );
    assert!(
        world.get::<Spatial>(entity).is_some(),
        "should have Spatial"
    );
    assert!(
        world.get::<Spatial2D>(entity).is_some(),
        "should have Spatial2D"
    );
    assert!(
        world.get::<InterpolateTransform2D>(entity).is_some(),
        "should have InterpolateTransform2D"
    );

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - 0.0).abs() < f32::EPSILON && (pos.0.y - 50.0).abs() < f32::EPSILON,
        "Position2D should be (0.0, 50.0)"
    );
    let vel = world.get::<Velocity2D>(entity).unwrap();
    assert_eq!(vel.0, Vec2::ZERO, "serving bolt velocity should be zero");

    let base = world.get::<BaseSpeed>(entity).unwrap();
    assert!((base.0 - 400.0).abs() < f32::EPSILON);
    let min = world.get::<MinSpeed>(entity).unwrap();
    assert!((min.0 - 200.0).abs() < f32::EPSILON);
    let max = world.get::<MaxSpeed>(entity).unwrap();
    assert!((max.0 - 800.0).abs() < f32::EPSILON);
    let radius = world.get::<BoltRadius>(entity).unwrap();
    assert!((radius.0 - 8.0).abs() < f32::EPSILON);

    assert!(world.get::<CleanupOnExit<RunState>>(entity).is_some());
    assert!(world.get::<CollisionLayers>(entity).is_some());
    // Headless bolts do NOT have GameDrawLayer
    assert!(world.get::<GameDrawLayer>(entity).is_none());
}

// Behavior 28: spawn() for extra bolt
#[test]
fn spawn_extra_bolt_creates_entity_with_extra_markers() {
    let mut world = World::new();
    let def = test_bolt_definition();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::new(50.0, 100.0))
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<ExtraBolt>(entity).is_some(),
        "should have ExtraBolt"
    );
    assert!(
        world.get::<CleanupOnExit<NodeState>>(entity).is_some(),
        "should have CleanupOnExit<NodeState>"
    );
    assert!(
        world.get::<PrimaryBolt>(entity).is_none(),
        "should NOT have PrimaryBolt"
    );
    assert!(
        world.get::<Spatial>(entity).is_some(),
        "should have Spatial"
    );
    assert!(
        world.get::<Spatial2D>(entity).is_some(),
        "should have Spatial2D"
    );
    assert!(
        world.get::<InterpolateTransform2D>(entity).is_some(),
        "should have InterpolateTransform2D"
    );

    let vel = world.get::<Velocity2D>(entity).unwrap();
    assert!(
        (vel.0.x - 0.0).abs() < f32::EPSILON && (vel.0.y - 400.0).abs() < f32::EPSILON,
        "Velocity2D should be (0.0, 400.0)"
    );
}

// Behavior 29: spawn() with .spawned_by() inserts SpawnedByEvolution
#[test]
fn spawn_with_spawned_by_inserts_evolution_marker() {
    let mut world = World::new();
    let def = test_bolt_definition();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .spawned_by("chain_bolt")
            .headless()
            .spawn(commands)
    });
    let spawned_by = world
        .get::<SpawnedByEvolution>(entity)
        .expect("should have SpawnedByEvolution");
    assert_eq!(spawned_by.0, "chain_bolt");
}

// Behavior 30: spawn() with .with_lifespan() inserts BoltLifespan
#[test]
fn spawn_with_lifespan_inserts_timer() {
    let mut world = World::new();
    let def = test_bolt_definition();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_lifespan(5.0)
            .headless()
            .spawn(commands)
    });
    let lifespan = world
        .get::<BoltLifespan>(entity)
        .expect("should have BoltLifespan");
    assert!(
        (lifespan.0.duration().as_secs_f32() - 5.0).abs() < 1e-3,
        "BoltLifespan duration should be ~5.0"
    );
}

// ── Section G: with_inherited_effects() and Effect Transfer ──────────

// Behavior 31: with_inherited_effects() stores effects for spawn-time insertion
#[test]
fn spawn_with_inherited_effects_inserts_bound_effects() {
    let node_a = EffectNode::Do(EffectKind::DamageBoost(5.0));
    let node_b = EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 });
    let inherited = BoundEffects(vec![
        ("chip_a".to_string(), node_a),
        ("chip_b".to_string(), node_b),
    ]);

    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&test_bolt_definition())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_inherited_effects(&inherited)
            .headless()
            .spawn(commands)
    });

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("should have BoundEffects");
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should have 2 inherited entries"
    );
    assert_eq!(bound.0[0].0, "chip_a");
    assert_eq!(bound.0[1].0, "chip_b");
}

#[test]
fn spawn_with_empty_inherited_effects_inserts_empty_bound_effects() {
    let inherited = BoundEffects(vec![]);

    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&test_bolt_definition())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_inherited_effects(&inherited)
            .headless()
            .spawn(commands)
    });

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("should have BoundEffects even when empty");
    assert!(bound.0.is_empty());
}

// Behavior 32: spawn() without inherited/with effects does NOT insert BoundEffects
#[test]
fn spawn_without_effects_has_no_bound_effects() {
    let mut world = World::new();
    let def = test_bolt_definition();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .headless()
            .spawn(commands)
    });

    // Guard against false pass — verify a non-#[require] component is present
    assert!(
        world.get::<ExtraBolt>(entity).is_some(),
        "entity should have ExtraBolt marker from builder"
    );
    assert!(
        world.get::<BoundEffects>(entity).is_none(),
        "entity should NOT have BoundEffects when no effects methods called"
    );
}

// Behavior: explicit effects appear before inherited in BoundEffects
#[test]
fn spawn_with_both_effects_orders_explicit_before_inherited() {
    let explicit = vec![(
        "explicit_chip".to_string(),
        EffectNode::Do(EffectKind::DamageBoost(2.0)),
    )];
    let inherited = BoundEffects(vec![(
        "inherited_chip".to_string(),
        EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 }),
    )]);

    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&test_bolt_definition())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_effects(explicit)
            .with_inherited_effects(&inherited)
            .headless()
            .spawn(commands)
    });

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("should have BoundEffects");
    assert_eq!(bound.0.len(), 2, "should have 2 entries total");
    assert_eq!(
        bound.0[0].0, "explicit_chip",
        "explicit effects should come first"
    );
    assert_eq!(
        bound.0[1].0, "inherited_chip",
        "inherited effects should come second"
    );
}

// Behavior 33: with_inherited_effects() clones effects
#[test]
fn inherited_effects_are_cloned_not_moved() {
    let node = EffectNode::Do(EffectKind::DamageBoost(5.0));
    let inherited = BoundEffects(vec![("chip".to_string(), node)]);

    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&test_bolt_definition())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_inherited_effects(&inherited)
            .headless()
            .spawn(commands)
    });

    // Original reference is still valid (it was borrowed, not consumed)
    assert_eq!(
        inherited.0.len(),
        1,
        "original BoundEffects should still have its entries"
    );
    assert_eq!(inherited.0[0].0, "chip");

    // Spawned entity also has the effects
    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(bound.0.len(), 1);
    assert_eq!(bound.0[0].0, "chip");
}
