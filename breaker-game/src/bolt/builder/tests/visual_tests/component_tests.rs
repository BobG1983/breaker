use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Scale2D, Velocity2D};

use super::helpers::test_bolt_definition;
use crate::{
    bolt::components::{Bolt, BoltServing, ExtraBolt, PrimaryBolt},
    effect::{BoundEffects, EffectKind, EffectNode},
    shared::{CleanupOnNodeExit, CleanupOnRunEnd, GameDrawLayer, size::BaseRadius},
};

// ── Behavior 18: Headless bolt still has all gameplay components ──

#[test]
fn headless_primary_serving_has_all_gameplay_components() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::new(0.0, 50.0))
        .serving()
        .primary()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    assert!(world.get::<Bolt>(entity).is_some(), "should have Bolt");
    assert!(
        world.get::<PrimaryBolt>(entity).is_some(),
        "should have PrimaryBolt"
    );
    assert!(
        world.get::<BoltServing>(entity).is_some(),
        "should have BoltServing"
    );
    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - 0.0).abs() < f32::EPSILON && (pos.0.y - 50.0).abs() < f32::EPSILON,
        "should have Position2D(0.0, 50.0)"
    );
    let vel = world.get::<Velocity2D>(entity).unwrap();
    assert_eq!(vel.0, Vec2::ZERO, "serving bolt should have zero velocity");
    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 8.0).abs() < f32::EPSILON && (scale.y - 8.0).abs() < f32::EPSILON,
        "should have Scale2D (8.0, 8.0)"
    );
    assert!(
        world.get::<BaseRadius>(entity).is_some(),
        "should have BaseRadius"
    );
    assert!(
        world.get::<CleanupOnRunEnd>(entity).is_some(),
        "should have CleanupOnRunEnd"
    );
    // Headless does NOT have GameDrawLayer
    assert!(
        world.get::<GameDrawLayer>(entity).is_none(),
        "headless should NOT have GameDrawLayer::Bolt"
    );
}

#[test]
fn headless_extra_velocity_has_correct_markers() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .extra()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    assert!(
        world.get::<ExtraBolt>(entity).is_some(),
        "should have ExtraBolt"
    );
    assert!(
        world.get::<CleanupOnNodeExit>(entity).is_some(),
        "should have CleanupOnNodeExit"
    );
}

// ── Behavior 19: Headless primary serving bolt via spawn() inserts all components ──

#[test]
fn headless_spawn_with_effects_has_bound_effects() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::new(0.0, 50.0))
        .with_effects(vec![(
            "test".to_string(),
            EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 }),
        )])
        .serving()
        .primary()
        .headless()
        .spawn(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("should have BoundEffects");
    assert_eq!(bound.0.len(), 1, "should have 1 effect entry");
    assert_eq!(bound.0[0].0, "test");
}

#[test]
fn headless_spawn_without_effects_has_no_bound_effects() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::new(0.0, 50.0))
        .serving()
        .primary()
        .headless()
        .spawn(&mut world);

    // Guard against false pass
    assert!(
        world.get::<PrimaryBolt>(entity).is_some(),
        "should have PrimaryBolt"
    );
    assert!(
        world.get::<BoundEffects>(entity).is_none(),
        "should NOT have BoundEffects when no effects methods called"
    );
}

// ── Behavior 20: Headless extra velocity bolt via spawn() inserts correct markers ──

#[test]
fn headless_extra_velocity_spawn_has_correct_markers() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::new(200.0, 300.0))
        .with_velocity(Velocity2D(Vec2::new(102.9, 385.5)))
        .extra()
        .headless()
        .spawn(&mut world);

    assert!(world.get::<Bolt>(entity).is_some(), "should have Bolt");
    assert!(
        world.get::<ExtraBolt>(entity).is_some(),
        "should have ExtraBolt"
    );
    assert!(
        world.get::<PrimaryBolt>(entity).is_none(),
        "should NOT have PrimaryBolt"
    );
    assert!(
        world.get::<BoltServing>(entity).is_none(),
        "should NOT have BoltServing"
    );
    assert!(
        world.get::<CleanupOnNodeExit>(entity).is_some(),
        "should have CleanupOnNodeExit"
    );
    let vel = world.get::<Velocity2D>(entity).unwrap();
    assert!(
        (vel.0.x - 102.9).abs() < f32::EPSILON && (vel.0.y - 385.5).abs() < f32::EPSILON,
        "Velocity2D should be (102.9, 385.5)"
    );
}

#[test]
fn headless_primary_velocity_spawn_has_primary_marker() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .primary()
        .headless()
        .spawn(&mut world);

    assert!(
        world.get::<PrimaryBolt>(entity).is_some(),
        "should have PrimaryBolt"
    );
    assert!(
        world.get::<CleanupOnRunEnd>(entity).is_some(),
        "should have CleanupOnRunEnd"
    );
}

// ── Behavior 22: All 8 terminal states produce correct role/motion combinations ──

#[test]
fn headless_serving_primary_has_correct_markers() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    assert!(world.get::<PrimaryBolt>(entity).is_some());
    assert!(world.get::<BoltServing>(entity).is_some());
    assert!(world.get::<CleanupOnRunEnd>(entity).is_some());
    assert!(world.get::<Mesh2d>(entity).is_none());
}

#[test]
fn headless_serving_extra_has_correct_markers() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .extra()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    assert!(world.get::<ExtraBolt>(entity).is_some());
    assert!(world.get::<BoltServing>(entity).is_some());
    assert!(world.get::<CleanupOnNodeExit>(entity).is_some());
    assert!(world.get::<Mesh2d>(entity).is_none());
}

#[test]
fn headless_velocity_primary_has_correct_markers() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .primary()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    assert!(world.get::<PrimaryBolt>(entity).is_some());
    assert!(world.get::<BoltServing>(entity).is_none());
    assert!(world.get::<CleanupOnRunEnd>(entity).is_some());
    assert!(world.get::<Mesh2d>(entity).is_none());
}

#[test]
fn headless_velocity_extra_has_correct_markers() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .extra()
        .headless()
        .build();
    let entity = world.spawn(bundle).id();

    assert!(world.get::<ExtraBolt>(entity).is_some());
    assert!(world.get::<BoltServing>(entity).is_none());
    assert!(world.get::<CleanupOnNodeExit>(entity).is_some());
    assert!(world.get::<Mesh2d>(entity).is_none());
}

#[test]
fn rendered_serving_primary_has_mesh() {
    let def = test_bolt_definition();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<ColorMaterial>();
    app.add_systems(Update, {
        move |mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<ColorMaterial>>| {
            let bundle = Bolt::builder()
                .definition(&def)
                .at_position(Vec2::ZERO)
                .serving()
                .primary()
                .rendered(&mut meshes, &mut materials)
                .build();
            commands.spawn(bundle);
        }
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Bolt>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    let entity = entities[0];

    assert!(app.world().get::<PrimaryBolt>(entity).is_some());
    assert!(app.world().get::<BoltServing>(entity).is_some());
    assert!(app.world().get::<CleanupOnRunEnd>(entity).is_some());
    assert!(app.world().get::<Mesh2d>(entity).is_some());
}

#[test]
fn headless_build_does_not_panic_on_bare_world() {
    // Edge case: Headless build can spawn into a bare World::new() with no asset infrastructure
    let def = test_bolt_definition();
    let mut world = World::new();
    let bundle = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .headless()
        .build();
    // This must NOT panic — no assets needed for headless
    let entity = world.spawn(bundle).id();
    assert!(world.get::<Bolt>(entity).is_some());
}
