use bevy::prelude::*;
use rantzsoft_physics2d::collision_layers::CollisionLayers;
use rantzsoft_spatial2d::components::{
    BaseSpeed, InterpolateTransform2D, MaxSpeed, MinSpeed, Position2D, Spatial, Spatial2D,
    Velocity2D,
};

use crate::{
    bolt::{
        components::{
            Bolt, BoltLifespan, BoltRadius, BoltServing, ExtraBolt, PrimaryBolt, SpawnedByEvolution,
        },
        definition::BoltDefinition,
    },
    effect::{BoundEffects, EffectKind, EffectNode},
    shared::{CleanupOnNodeExit, CleanupOnRunEnd, GameDrawLayer},
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
    }
}

// ── Section F: spawn(&mut World) -> Entity ──────────────────────────

// Behavior 27: spawn() creates entity with all build components
#[test]
fn spawn_primary_serving_creates_complete_entity() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::new(0.0, 50.0))
        .serving()
        .primary()
        .spawn(&mut world);

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

    assert!(world.get::<CleanupOnRunEnd>(entity).is_some());
    assert!(world.get::<CollisionLayers>(entity).is_some());
    assert!(world.get::<GameDrawLayer>(entity).is_some());
}

// Behavior 28: spawn() for extra bolt
#[test]
fn spawn_extra_bolt_creates_entity_with_extra_markers() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::new(50.0, 100.0))
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .extra()
        .spawn(&mut world);

    assert!(
        world.get::<ExtraBolt>(entity).is_some(),
        "should have ExtraBolt"
    );
    assert!(
        world.get::<CleanupOnNodeExit>(entity).is_some(),
        "should have CleanupOnNodeExit"
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
    let entity = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::ZERO)
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .extra()
        .spawned_by("chain_bolt")
        .spawn(&mut world);
    let spawned_by = world
        .get::<SpawnedByEvolution>(entity)
        .expect("should have SpawnedByEvolution");
    assert_eq!(spawned_by.0, "chain_bolt");
}

// Behavior 30: spawn() with .with_lifespan() inserts BoltLifespan
#[test]
fn spawn_with_lifespan_inserts_timer() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::ZERO)
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .extra()
        .with_lifespan(5.0)
        .spawn(&mut world);
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
    let entity = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::ZERO)
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .extra()
        .with_inherited_effects(&inherited)
        .spawn(&mut world);

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
    let entity = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::ZERO)
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .extra()
        .with_inherited_effects(&inherited)
        .spawn(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("should have BoundEffects even when empty");
    assert!(bound.0.is_empty());
}

// Behavior 32: spawn() without inherited/with effects does NOT insert BoundEffects
#[test]
fn spawn_without_effects_has_no_bound_effects() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::ZERO)
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .extra()
        .spawn(&mut world);

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

// Behavior 33: with_inherited_effects() clones effects
#[test]
fn inherited_effects_are_cloned_not_moved() {
    let node = EffectNode::Do(EffectKind::DamageBoost(5.0));
    let inherited = BoundEffects(vec![("chip".to_string(), node)]);

    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&test_bolt_definition())
        .at_position(Vec2::ZERO)
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .extra()
        .with_inherited_effects(&inherited)
        .spawn(&mut world);

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
