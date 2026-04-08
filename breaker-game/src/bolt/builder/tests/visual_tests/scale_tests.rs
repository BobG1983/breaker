use bevy::{ecs::world::CommandQueue, prelude::*};
use rantzsoft_spatial2d::components::Scale2D;

use super::helpers::test_bolt_definition;
use crate::{
    bolt::components::{Bolt, PrimaryBolt},
    shared::size::{BaseRadius, MaxRadius, MinRadius},
};

/// Spawns a bolt via Commands backed by a `CommandQueue`, then applies the queue.
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

// ── Behavior 23: Builder uses BaseRadius for radius component ──

#[test]
fn headless_bolt_has_base_radius_from_definition() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .headless()
            .spawn(commands)
    });

    let radius = world
        .get::<BaseRadius>(entity)
        .expect("should have BaseRadius");
    assert!(
        (radius.0 - 8.0).abs() < f32::EPSILON,
        "BaseRadius should be 8.0, got {}",
        radius.0
    );
}

#[test]
fn headless_bolt_with_radius_override_has_base_radius() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_radius(20.0)
            .headless()
            .spawn(commands)
    });

    let radius = world
        .get::<BaseRadius>(entity)
        .expect("should have BaseRadius");
    assert!(
        (radius.0 - 20.0).abs() < f32::EPSILON,
        "BaseRadius should be 20.0 (overridden), got {}",
        radius.0
    );
}

// ── Behavior 24: Builder without definition uses default radius as BaseRadius ──

#[test]
fn headless_bolt_without_definition_has_default_base_radius() {
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .headless()
            .spawn(commands)
    });

    let radius = world
        .get::<BaseRadius>(entity)
        .expect("should have BaseRadius");
    assert!(
        (radius.0 - 8.0).abs() < f32::EPSILON,
        "BaseRadius should default to 8.0, got {}",
        radius.0
    );
    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 8.0).abs() < f32::EPSILON && (scale.y - 8.0).abs() < f32::EPSILON,
        "Scale2D should be (8.0, 8.0)"
    );
}

#[test]
fn headless_bolt_with_zero_radius() {
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_radius(0.0)
            .headless()
            .spawn(commands)
    });

    let radius = world
        .get::<BaseRadius>(entity)
        .expect("should have BaseRadius");
    assert!(
        radius.0.abs() < f32::EPSILON,
        "BaseRadius should be 0.0, got {}",
        radius.0
    );
    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        scale.x.abs() < f32::EPSILON && scale.y.abs() < f32::EPSILON,
        "Scale2D should be (0.0, 0.0)"
    );
}

// ── Behavior 29: MinRadius/MaxRadius from definition ──

#[test]
fn headless_bolt_with_definition_radius_constraints_has_min_max_radius() {
    let mut def = test_bolt_definition();
    def.min_radius = Some(4.0);
    def.max_radius = Some(20.0);

    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .headless()
            .spawn(commands)
    });

    let min_r = world
        .get::<MinRadius>(entity)
        .expect("should have MinRadius");
    assert!(
        (min_r.0 - 4.0).abs() < f32::EPSILON,
        "MinRadius should be 4.0, got {}",
        min_r.0
    );
    let max_r = world
        .get::<MaxRadius>(entity)
        .expect("should have MaxRadius");
    assert!(
        (max_r.0 - 20.0).abs() < f32::EPSILON,
        "MaxRadius should be 20.0, got {}",
        max_r.0
    );
}

#[test]
fn headless_bolt_with_no_definition_radius_constraints_has_no_min_max_radius() {
    let def = test_bolt_definition(); // min_radius: None, max_radius: None

    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .headless()
            .spawn(commands)
    });

    // Guard against false pass
    assert!(
        world.get::<PrimaryBolt>(entity).is_some(),
        "should have PrimaryBolt"
    );
    assert!(
        world.get::<MinRadius>(entity).is_none(),
        "should NOT have MinRadius when definition has None"
    );
    assert!(
        world.get::<MaxRadius>(entity).is_none(),
        "should NOT have MaxRadius when definition has None"
    );
}
