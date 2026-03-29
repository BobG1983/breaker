use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{Position2D, PreviousPosition, Scale2D, Velocity2D};

use super::effect::*;
use crate::{
    bolt::{
        components::{
            Bolt, BoltBaseSpeed, BoltLifespan, BoltMaxSpeed, BoltMinSpeed, BoltRadius, ExtraBolt,
        },
        resources::BoltConfig,
    },
    effect::{BoundEffects, EffectKind, EffectNode, StagedEffects},
    shared::{
        BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, CleanupOnNodeExit, CleanupOnRunEnd, GameDrawLayer,
        WALL_LAYER, rng::GameRng,
    },
};

fn world_with_bolt_config() -> World {
    let mut world = World::new();
    world.insert_resource(BoltConfig::default());
    world.insert_resource(GameRng::default());
    world
}

// -- fire() — bolt spawning ─────────────────────────────────────

#[test]
fn fire_spawns_requested_count_with_full_physics_components() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::new(50.0, 100.0))).id();

    fire(entity, 3, None, false, &mut world);

    // Query for spawned bolt entities (excluding the owner)
    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolts: Vec<Entity> = query.iter(&world).collect();
    assert_eq!(
        bolts.len(),
        3,
        "expected 3 bolts spawned, got {}",
        bolts.len()
    );

    for bolt in &bolts {
        // Position2D from owner
        let pos = world
            .get::<Position2D>(*bolt)
            .expect("bolt should have Position2D");
        assert_eq!(
            pos.0,
            Vec2::new(50.0, 100.0),
            "bolt Position2D should match owner"
        );

        // PreviousPosition
        let prev = world
            .get::<PreviousPosition>(*bolt)
            .expect("bolt should have PreviousPosition");
        assert_eq!(prev.0, Vec2::new(50.0, 100.0));

        // Scale2D — radius=8.0 from default config
        let scale = world
            .get::<Scale2D>(*bolt)
            .expect("bolt should have Scale2D");
        assert!((scale.x - 8.0).abs() < f32::EPSILON);
        assert!((scale.y - 8.0).abs() < f32::EPSILON);

        // Aabb2D
        let aabb = world.get::<Aabb2D>(*bolt).expect("bolt should have Aabb2D");
        assert_eq!(aabb.center, Vec2::ZERO);
        assert_eq!(aabb.half_extents, Vec2::new(8.0, 8.0));

        // CollisionLayers
        let layers = world
            .get::<CollisionLayers>(*bolt)
            .expect("bolt should have CollisionLayers");
        assert_eq!(layers.membership, BOLT_LAYER);
        assert_eq!(layers.mask, CELL_LAYER | WALL_LAYER | BREAKER_LAYER);

        // Speed components
        let base = world
            .get::<BoltBaseSpeed>(*bolt)
            .expect("bolt should have BoltBaseSpeed");
        assert!((base.0 - 400.0).abs() < f32::EPSILON);

        let min = world
            .get::<BoltMinSpeed>(*bolt)
            .expect("bolt should have BoltMinSpeed");
        assert!((min.0 - 200.0).abs() < f32::EPSILON);

        let max = world
            .get::<BoltMaxSpeed>(*bolt)
            .expect("bolt should have BoltMaxSpeed");
        assert!((max.0 - 800.0).abs() < f32::EPSILON);

        let radius = world
            .get::<BoltRadius>(*bolt)
            .expect("bolt should have BoltRadius");
        assert!((radius.0 - 8.0).abs() < f32::EPSILON);

        // CleanupOnNodeExit
        assert!(
            world.get::<CleanupOnNodeExit>(*bolt).is_some(),
            "bolt should have CleanupOnNodeExit"
        );

        // GameDrawLayer::Bolt
        assert!(
            world.get::<GameDrawLayer>(*bolt).is_some(),
            "bolt should have GameDrawLayer"
        );
    }
}

#[test]
fn fire_count_one_spawns_exactly_one_bolt() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1, None, false, &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let count = query.iter(&world).count();
    assert_eq!(
        count, 1,
        "count=1 should spawn exactly one bolt, got {count}"
    );
}

#[test]
fn fire_count_zero_spawns_no_bolts() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 0, None, false, &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let count = query.iter(&world).count();
    assert_eq!(count, 0, "count=0 should spawn zero bolts, got {count}");
}

#[test]
fn fire_spawns_bolts_with_randomized_velocity_at_base_speed() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1, None, false, &mut world);

    let mut query = world.query_filtered::<&Velocity2D, (With<Bolt>, With<ExtraBolt>)>();
    let vel = query.iter(&world).next().expect("bolt should exist");
    assert!(
        (vel.0.length() - 400.0).abs() < 1.0,
        "velocity magnitude should be base_speed (400.0), got {}",
        vel.0.length()
    );
}

#[test]
fn fire_spawns_bolt_with_custom_base_speed() {
    let mut world = World::new();
    world.insert_resource(BoltConfig {
        base_speed: 600.0,
        ..BoltConfig::default()
    });
    world.insert_resource(GameRng::default());
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1, None, false, &mut world);

    let mut query = world.query_filtered::<&Velocity2D, (With<Bolt>, With<ExtraBolt>)>();
    let vel = query.iter(&world).next().expect("bolt should exist");
    assert!(
        (vel.0.length() - 600.0).abs() < 1.0,
        "velocity magnitude should be custom base_speed (600.0), got {}",
        vel.0.length()
    );
}

#[test]
fn fire_spawns_bolt_at_owner_position2d_not_transform() {
    let mut world = world_with_bolt_config();
    let entity = world
        .spawn((
            Position2D(Vec2::new(50.0, 75.0)),
            Transform::from_xyz(999.0, 999.0, 0.0),
        ))
        .id();

    fire(entity, 1, None, false, &mut world);

    let mut query = world.query_filtered::<&Position2D, (With<Bolt>, With<ExtraBolt>)>();
    let pos = query.iter(&world).next().expect("bolt should exist");
    assert_eq!(
        pos.0,
        Vec2::new(50.0, 75.0),
        "bolt should use Position2D (50, 75), not Transform (999, 999)"
    );
}

#[test]
fn fire_spawns_bolt_at_zero_when_owner_has_no_position2d() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn_empty().id();

    fire(entity, 1, None, false, &mut world);

    let mut query = world.query_filtered::<&Position2D, (With<Bolt>, With<ExtraBolt>)>();
    let pos = query.iter(&world).next().expect("bolt should exist");
    assert_eq!(
        pos.0,
        Vec2::ZERO,
        "bolt should default to Vec2::ZERO when owner has no Position2D"
    );
}

#[test]
fn fire_marks_bolts_with_extra_bolt_and_cleanup_on_node_exit() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1, None, false, &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query.iter(&world).next().expect("bolt should exist");

    assert!(
        world.get::<ExtraBolt>(bolt).is_some(),
        "bolt should have ExtraBolt"
    );
    assert!(
        world.get::<CleanupOnNodeExit>(bolt).is_some(),
        "bolt should have CleanupOnNodeExit"
    );
    assert!(
        world.get::<CleanupOnRunEnd>(bolt).is_none(),
        "bolt should NOT have CleanupOnRunEnd"
    );
}

#[test]
fn fire_with_lifespan_adds_bolt_lifespan_timer() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1, Some(5.0), false, &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query.iter(&world).next().expect("bolt should exist");

    let lifespan = world
        .get::<BoltLifespan>(bolt)
        .expect("bolt should have BoltLifespan with lifespan=Some(5.0)");
    assert!(
        (lifespan.0.duration().as_secs_f32() - 5.0).abs() < 0.01,
        "BoltLifespan duration should be 5.0s, got {}",
        lifespan.0.duration().as_secs_f32()
    );
}

#[test]
fn fire_with_very_short_lifespan_creates_valid_timer() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1, Some(0.01), false, &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query.iter(&world).next().expect("bolt should exist");

    let lifespan = world
        .get::<BoltLifespan>(bolt)
        .expect("bolt should have BoltLifespan with lifespan=Some(0.01)");
    assert!(
        (lifespan.0.duration().as_secs_f32() - 0.01).abs() < 0.001,
        "BoltLifespan should work with short duration 0.01, got {}",
        lifespan.0.duration().as_secs_f32()
    );
}

#[test]
fn fire_with_no_lifespan_does_not_add_bolt_lifespan() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1, None, false, &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query.iter(&world).next().expect("bolt should exist");

    assert!(
        world.get::<BoltLifespan>(bolt).is_none(),
        "bolt should NOT have BoltLifespan when lifespan=None"
    );
}

#[test]
fn fire_with_inherit_true_copies_bound_effects() {
    let mut world = world_with_bolt_config();
    let bound = BoundEffects(vec![(
        "test_chip".to_string(),
        EffectNode::Do(EffectKind::DamageBoost(1.5)),
    )]);
    let entity = world
        .spawn((Position2D(Vec2::ZERO), bound, StagedEffects::default()))
        .id();

    fire(entity, 2, None, true, &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolts: Vec<Entity> = query.iter(&world).collect();
    assert_eq!(bolts.len(), 2, "should spawn 2 bolts");

    for bolt in &bolts {
        let effects = world
            .get::<BoundEffects>(*bolt)
            .expect("spawned bolt should have BoundEffects when inherit=true");
        assert_eq!(
            effects.0.len(),
            1,
            "BoundEffects should have 1 entry, got {}",
            effects.0.len()
        );
        assert_eq!(effects.0[0].0, "test_chip");
    }
}

#[test]
fn fire_with_inherit_true_and_empty_bound_effects_spawns_empty() {
    let mut world = world_with_bolt_config();
    let entity = world
        .spawn((Position2D(Vec2::ZERO), BoundEffects::default()))
        .id();

    fire(entity, 1, None, true, &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query.iter(&world).next().expect("bolt should exist");

    let effects = world.get::<BoundEffects>(bolt);
    // Either has BoundEffects with empty vec, or no BoundEffects at all — both are acceptable
    if let Some(effects) = effects {
        assert!(
            effects.0.is_empty(),
            "empty BoundEffects should produce empty BoundEffects on spawned bolt"
        );
    }
}

#[test]
fn fire_with_inherit_false_does_not_copy_bound_effects() {
    let mut world = world_with_bolt_config();
    let bound = BoundEffects(vec![(
        "chip".to_string(),
        EffectNode::Do(EffectKind::DamageBoost(1.5)),
    )]);
    let entity = world.spawn((Position2D(Vec2::ZERO), bound)).id();

    fire(entity, 1, None, false, &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query.iter(&world).next().expect("bolt should exist");

    let effects = world.get::<BoundEffects>(bolt);
    // Either no BoundEffects at all, or empty BoundEffects
    if let Some(effects) = effects {
        assert!(
            effects.0.is_empty(),
            "inherit=false should not copy BoundEffects to spawned bolt"
        );
    }
}

#[test]
fn fire_with_inherit_true_and_no_bound_effects_does_not_panic() {
    let mut world = world_with_bolt_config();
    // Entity has Position2D but no BoundEffects component
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    // Should not panic
    fire(entity, 1, None, true, &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let count = query.iter(&world).count();
    assert_eq!(count, 1, "bolt should still be spawned");
}

#[test]
fn fire_spawns_multiple_bolts_with_distinct_velocity_directions() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 3, None, false, &mut world);

    let mut query = world.query_filtered::<&Velocity2D, (With<Bolt>, With<ExtraBolt>)>();
    let velocities: Vec<Vec2> = query.iter(&world).map(|v| v.0).collect();
    assert_eq!(velocities.len(), 3, "should spawn 3 bolts");

    // All should have base_speed magnitude
    for vel in &velocities {
        assert!(
            (vel.length() - 400.0).abs() < 1.0,
            "each bolt velocity magnitude should be ~400.0, got {}",
            vel.length()
        );
    }

    // Probabilistically, at least two should differ in direction
    // (since each uses a separate random angle)
    let directions_differ = velocities[0].normalize() != velocities[1].normalize()
        || velocities[1].normalize() != velocities[2].normalize();
    assert!(
        directions_differ,
        "with 3 bolts and independent random angles, at least two should have different directions"
    );
}

#[test]
fn fire_uses_custom_radius_from_bolt_config() {
    let mut world = World::new();
    world.insert_resource(BoltConfig {
        radius: 6.0,
        ..BoltConfig::default()
    });
    world.insert_resource(GameRng::default());
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1, None, false, &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query.iter(&world).next().expect("bolt should exist");

    let scale = world
        .get::<Scale2D>(bolt)
        .expect("bolt should have Scale2D");
    assert!(
        (scale.x - 6.0).abs() < f32::EPSILON,
        "Scale2D.x should use custom radius (6.0)"
    );
    assert!(
        (scale.y - 6.0).abs() < f32::EPSILON,
        "Scale2D.y should use custom radius (6.0)"
    );

    let aabb = world.get::<Aabb2D>(bolt).expect("bolt should have Aabb2D");
    assert_eq!(aabb.half_extents, Vec2::new(6.0, 6.0));

    let radius = world
        .get::<BoltRadius>(bolt)
        .expect("bolt should have BoltRadius");
    assert!((radius.0 - 6.0).abs() < f32::EPSILON);
}

// -- reverse() — no-op ──────────────────────────────────────────

#[test]
fn reverse_does_not_despawn_previously_spawned_bolts() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 2, None, false, &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let count_before = query.iter(&world).count();

    reverse(entity, 2, None, false, &mut world);

    let count_after = query.iter(&world).count();
    assert_eq!(
        count_before, count_after,
        "reverse should not despawn spawned bolts"
    );
}

#[test]
fn reverse_with_no_prior_spawned_bolts_does_not_panic() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    // Should not panic
    reverse(entity, 2, None, false, &mut world);
}
