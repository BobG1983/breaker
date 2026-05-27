use bevy::prelude::*;
use rantzsoft_physics2d::collision_layers::CollisionLayers;
use rantzsoft_spatial2d::components::Velocity2D;

use super::test_bolt_definition;
use crate::{
    bolt::components::{
        Bolt, BoltServing, ExtraBolt, LifetimeEndBehavior, PhantomBolt, PhantomDamagedCells,
        PhantomDedupKey, PhantomParams, PrimaryBolt,
    },
    prelude::*,
    shared::{
        BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, GameDrawLayer, Lifespan, PhantomFlicker, WALL_LAYER,
    },
};

// ── Behavior 1: .phantom(...) on Rendered+Extra+HasVelocity inserts full rendered phantom set ──

#[test]
fn phantom_on_rendered_extra_bolt_inserts_full_rendered_phantom_set() {
    let def = test_bolt_definition();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<ColorMaterial>();
    app.add_systems(Update, {
        move |mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<ColorMaterial>>| {
            Bolt::builder()
                .definition(&def)
                .at_position(Vec2::new(0.0, 50.0))
                .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
                .extra()
                .phantom(PhantomParams {
                    dedup_key: PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
                })
                .rendered(&mut meshes, &mut materials)
                .spawn(&mut commands);
        }
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Bolt>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1, "expected exactly 1 Bolt entity");
    let e = entities[0];

    assert!(
        app.world().get::<Bolt>(e).is_some(),
        "Bolt marker should be present"
    );
    assert!(
        app.world().get::<ExtraBolt>(e).is_some(),
        "ExtraBolt should be present"
    );
    assert!(
        app.world().get::<CleanupOnExit<NodeState>>(e).is_some(),
        "CleanupOnExit<NodeState> should be present for extra bolt"
    );
    assert!(
        app.world().get::<Mesh2d>(e).is_some(),
        "Mesh2d should be present (rendered terminal)"
    );
    assert!(
        app.world()
            .get::<MeshMaterial2d<ColorMaterial>>(e)
            .is_some(),
        "MeshMaterial2d<ColorMaterial> should be present (rendered terminal)"
    );
    let layer = app
        .world()
        .get::<GameDrawLayer>(e)
        .expect("GameDrawLayer should be present");
    assert!(
        matches!(layer, GameDrawLayer::Bolt),
        "GameDrawLayer should be Bolt"
    );
    assert!(
        app.world().get::<PhantomBolt>(e).is_some(),
        "PhantomBolt should be present after .phantom(...)"
    );
    assert!(
        app.world().get::<PhantomDedupKey>(e).is_some(),
        "PhantomDedupKey should be present after .phantom(...)"
    );
    assert!(
        app.world().get::<PhantomDamagedCells>(e).is_some(),
        "PhantomDamagedCells should be present after .phantom(...)"
    );
    let flicker = app
        .world()
        .get::<PhantomFlicker>(e)
        .expect("PhantomFlicker should be present on rendered phantom bolt");
    assert!(
        (flicker.frequency - 4.0).abs() < f32::EPSILON,
        "PhantomFlicker.frequency should be 4.0, got {}",
        flicker.frequency
    );
    assert!(
        (flicker.min_alpha - 0.3).abs() < f32::EPSILON,
        "PhantomFlicker.min_alpha should be 0.3, got {}",
        flicker.min_alpha
    );

    // Edge case: absence guards
    assert!(
        app.world().get::<BoltServing>(e).is_none(),
        "BoltServing must be absent (pipeline used .with_velocity, not .serving)"
    );
    assert!(
        app.world()
            .get::<crate::bolt::components::BoltLifespan>(e)
            .is_none(),
        "BoltLifespan must be absent (no .with_lifespan chained)"
    );
    assert!(
        app.world().get::<Lifespan>(e).is_none(),
        "Lifespan must be absent (Wave 2 does not insert shared Lifespan)"
    );
    assert!(
        app.world().get::<LifetimeEndBehavior>(e).is_none(),
        "LifetimeEndBehavior must be absent (.with_lifetime_end_behavior not chained)"
    );
}

// ── Behavior 2: .phantom(...) on Headless+Extra+HasVelocity inserts headless phantom set ──

#[test]
fn phantom_on_headless_extra_bolt_inserts_headless_phantom_set() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::new(0.0, 50.0))
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .extra()
        .phantom(PhantomParams {
            dedup_key: PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
        })
        .headless()
        .spawn(&mut world.commands());
    world.flush();

    assert!(
        world.get::<Bolt>(entity).is_some(),
        "Bolt should be present"
    );
    assert!(
        world.get::<ExtraBolt>(entity).is_some(),
        "ExtraBolt should be present"
    );
    assert!(
        world.get::<CleanupOnExit<NodeState>>(entity).is_some(),
        "CleanupOnExit<NodeState> should be present for extra bolt"
    );
    assert!(
        world.get::<PhantomBolt>(entity).is_some(),
        "PhantomBolt should be present"
    );
    assert!(
        world.get::<PhantomDedupKey>(entity).is_some(),
        "PhantomDedupKey should be present"
    );
    assert!(
        world.get::<PhantomDamagedCells>(entity).is_some(),
        "PhantomDamagedCells should be present"
    );

    assert!(
        world.get::<Mesh2d>(entity).is_none(),
        "Mesh2d should NOT be present on headless bolt"
    );
    assert!(
        world.get::<MeshMaterial2d<ColorMaterial>>(entity).is_none(),
        "MeshMaterial2d should NOT be present on headless bolt"
    );
    assert!(
        world.get::<GameDrawLayer>(entity).is_none(),
        "GameDrawLayer should NOT be present on headless bolt"
    );
    assert!(
        world.get::<PhantomFlicker>(entity).is_none(),
        "PhantomFlicker should NOT be present on headless phantom bolt"
    );
}

#[test]
fn phantom_on_headless_primary_bolt_inserts_primary_phantom_set_and_skips_flicker() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::new(0.0, 50.0))
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .primary()
        .phantom(PhantomParams {
            dedup_key: PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
        })
        .headless()
        .spawn(&mut world.commands());
    world.flush();

    assert!(
        world.get::<Bolt>(entity).is_some(),
        "Bolt should be present"
    );
    assert!(
        world.get::<PrimaryBolt>(entity).is_some(),
        "PrimaryBolt should be present"
    );
    assert!(
        world.get::<CleanupOnExit<RunState>>(entity).is_some(),
        "CleanupOnExit<RunState> should be present for primary bolt"
    );
    assert!(
        world.get::<PhantomBolt>(entity).is_some(),
        "PhantomBolt should be present"
    );
    assert!(
        world.get::<PhantomDedupKey>(entity).is_some(),
        "PhantomDedupKey should be present"
    );
    assert!(
        world.get::<PhantomDamagedCells>(entity).is_some(),
        "PhantomDamagedCells should be present"
    );

    assert!(
        world.get::<ExtraBolt>(entity).is_none(),
        "ExtraBolt must NOT be present on primary bolt"
    );
    assert!(
        world.get::<CleanupOnExit<NodeState>>(entity).is_none(),
        "CleanupOnExit<NodeState> must NOT be present on primary bolt"
    );
    assert!(
        world.get::<Mesh2d>(entity).is_none(),
        "Mesh2d must NOT be present on headless bolt"
    );
    assert!(
        world.get::<MeshMaterial2d<ColorMaterial>>(entity).is_none(),
        "MeshMaterial2d must NOT be present on headless bolt"
    );
    assert!(
        world.get::<GameDrawLayer>(entity).is_none(),
        "GameDrawLayer must NOT be present on headless bolt"
    );
    assert!(
        world.get::<PhantomFlicker>(entity).is_none(),
        "PhantomFlicker must NOT be present on headless phantom bolt — flicker is Visual-gated, not Role-gated"
    );
}

// ── Behavior 3: spawned phantom collision mask equals default CELL|WALL|BREAKER ──

#[test]
fn phantom_headless_extra_has_default_collision_mask() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::new(0.0, 50.0))
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .extra()
        .phantom(PhantomParams {
            dedup_key: PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
        })
        .headless()
        .spawn(&mut world.commands());
    world.flush();

    let layers = world
        .get::<CollisionLayers>(entity)
        .expect("CollisionLayers should be present");
    assert_eq!(
        layers.membership, BOLT_LAYER,
        "membership should be BOLT_LAYER"
    );
    assert_eq!(
        layers.mask,
        CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
        "mask should be CELL_LAYER | WALL_LAYER | BREAKER_LAYER"
    );
}

// ── Behavior 4: .with_lifetime_end_behavior(RevertToNormalBolt) inserts component ──

#[test]
fn with_lifetime_end_behavior_revert_inserts_matching_component() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::new(0.0, 50.0))
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .extra()
        .with_lifetime_end_behavior(LifetimeEndBehavior::RevertToNormalBolt)
        .headless()
        .spawn(&mut world.commands());
    world.flush();

    let behavior = world
        .get::<LifetimeEndBehavior>(entity)
        .expect("LifetimeEndBehavior should be present");
    assert_eq!(
        *behavior,
        LifetimeEndBehavior::RevertToNormalBolt,
        "LifetimeEndBehavior should be RevertToNormalBolt"
    );
}

#[test]
fn with_lifetime_end_behavior_last_call_wins() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::new(0.0, 50.0))
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .extra()
        .with_lifetime_end_behavior(LifetimeEndBehavior::Despawn)
        .with_lifetime_end_behavior(LifetimeEndBehavior::RevertToNormalBolt)
        .headless()
        .spawn(&mut world.commands());
    world.flush();

    let behavior = world
        .get::<LifetimeEndBehavior>(entity)
        .expect("LifetimeEndBehavior should be present");
    assert_eq!(
        *behavior,
        LifetimeEndBehavior::RevertToNormalBolt,
        "last .with_lifetime_end_behavior call should win (RevertToNormalBolt)"
    );
}

// ── Behavior 5: .with_lifetime_end_behavior(Despawn) inserts matching component ──

#[test]
fn with_lifetime_end_behavior_despawn_inserts_matching_component() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::new(0.0, 50.0))
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .extra()
        .with_lifetime_end_behavior(LifetimeEndBehavior::Despawn)
        .headless()
        .spawn(&mut world.commands());
    world.flush();

    let behavior = world
        .get::<LifetimeEndBehavior>(entity)
        .expect("LifetimeEndBehavior should be present");
    assert_eq!(
        *behavior,
        LifetimeEndBehavior::Despawn,
        "LifetimeEndBehavior should be Despawn"
    );
}

#[test]
fn no_lifetime_end_behavior_call_leaves_component_absent() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::new(0.0, 50.0))
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .extra()
        .headless()
        .spawn(&mut world.commands());
    world.flush();

    assert!(
        world.get::<LifetimeEndBehavior>(entity).is_none(),
        "LifetimeEndBehavior must be absent when .with_lifetime_end_behavior was not called"
    );
}

#[test]
fn no_phantom_call_leaves_phantom_bolt_absent() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::new(0.0, 50.0))
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .extra()
        .headless()
        .spawn(&mut world.commands());
    world.flush();

    assert!(
        world.get::<PhantomBolt>(entity).is_none(),
        "PhantomBolt must be absent when .phantom(...) was not called"
    );
    assert!(
        world.get::<PhantomFlicker>(entity).is_none(),
        "PhantomFlicker must be absent when .phantom(...) was not called"
    );
    assert!(
        world.get::<PhantomDedupKey>(entity).is_none(),
        "PhantomDedupKey must be absent when .phantom(...) was not called"
    );
    assert!(
        world.get::<PhantomDamagedCells>(entity).is_none(),
        "PhantomDamagedCells must be absent when .phantom(...) was not called"
    );
}
