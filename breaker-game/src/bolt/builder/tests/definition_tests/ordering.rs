use bevy::{ecs::world::CommandQueue, prelude::*};

use super::helpers::make_bolt_definition;
use crate::bolt::components::{
    Bolt, BoltAngleSpread, BoltBaseDamage, BoltDefinitionRef, BoltSpawnOffsetY,
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

// ── Section G: Optional method ordering and typestate independence ──────────

// ── Behaviors 14-17: .with_*() methods available in initial (unconfigured) typestate ──

#[test]
fn with_base_damage_available_in_initial_state() {
    // Verifies that .with_base_damage() compiles from initial state and
    // can continue chaining through to spawn.
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_base_damage(10.0)
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .headless()
            .spawn(commands)
    });
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should have BoltBaseDamage when called early in chain");
    assert!(
        (dmg.0 - 10.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 10.0, got {}",
        dmg.0
    );
}

#[test]
fn with_definition_name_available_in_initial_state() {
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_definition_name("Test".to_string())
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .headless()
            .spawn(commands)
    });
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should have BoltDefinitionRef when called early in chain");
    assert_eq!(def_ref.0, "Test");
}

#[test]
fn with_angle_spread_available_in_initial_state() {
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_angle_spread(0.5)
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .headless()
            .spawn(commands)
    });
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should have BoltAngleSpread when called early in chain");
    assert!(
        (spread.0 - 0.5).abs() < f32::EPSILON,
        "BoltAngleSpread should be 0.5, got {}",
        spread.0
    );
}

#[test]
fn with_spawn_offset_y_available_in_initial_state() {
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_spawn_offset_y(50.0)
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .headless()
            .spawn(commands)
    });
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should have BoltSpawnOffsetY when called early in chain");
    assert!(
        (offset.0 - 50.0).abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 50.0, got {}",
        offset.0
    );
}

// ── Section H: Override ordering — .with_*() before .definition() ──────────

// ── Behavior 18: .with_base_damage() called before .definition() — explicit override still wins ──

#[test]
fn with_base_damage_before_definition_explicit_override_wins() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_base_damage(25.0)
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .headless()
            .spawn(commands)
    });
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should have BoltBaseDamage");
    assert!(
        (dmg.0 - 25.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 25.0 (explicit override wins regardless of call order), got {}",
        dmg.0
    );
}

#[test]
fn with_base_damage_before_and_after_definition_last_with_wins() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_base_damage(25.0)
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_base_damage(42.0)
            .headless()
            .spawn(commands)
    });
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should have BoltBaseDamage");
    assert!(
        (dmg.0 - 42.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 42.0 (last .with_base_damage() wins), got {}",
        dmg.0
    );
}

// ── Behavior 19: .with_definition_name() called before .definition() — explicit override still wins ──

#[test]
fn with_definition_name_before_definition_explicit_override_wins() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_definition_name("Override".to_string())
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .headless()
            .spawn(commands)
    });
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should have BoltDefinitionRef");
    assert_eq!(
        def_ref.0, "Override",
        "BoltDefinitionRef should be 'Override' (explicit override wins regardless of call order), got '{}'",
        def_ref.0
    );
}

// ── Behavior 20: .with_angle_spread() called before .definition() — explicit override still wins ──

#[test]
fn with_angle_spread_before_definition_explicit_override_wins() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_angle_spread(0.8)
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .headless()
            .spawn(commands)
    });
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should have BoltAngleSpread");
    assert!(
        (spread.0 - 0.8).abs() < f32::EPSILON,
        "BoltAngleSpread should be 0.8 (explicit override wins regardless of call order), got {}",
        spread.0
    );
}

// ── Behavior 21: .with_spawn_offset_y() called before .definition() — explicit override still wins ──

#[test]
fn with_spawn_offset_y_before_definition_explicit_override_wins() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_spawn_offset_y(30.0)
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .headless()
            .spawn(commands)
    });
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should have BoltSpawnOffsetY");
    assert!(
        (offset.0 - 30.0).abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 30.0 (explicit override wins regardless of call order), got {}",
        offset.0
    );
}
