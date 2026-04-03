use bevy::{ecs::world::CommandQueue, prelude::*};

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

// ── Section A: .with_base_damage() standalone ──────────────────────────────

// ── Behavior 1: .with_base_damage() inserts BoltBaseDamage on spawned entity ──

#[test]
fn with_base_damage_inserts_bolt_base_damage() {
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_base_damage(15.0)
            .headless()
            .spawn(commands)
    });
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should have BoltBaseDamage");
    assert!(
        (dmg.0 - 15.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 15.0, got {}",
        dmg.0
    );
}

#[test]
fn with_base_damage_zero_is_valid() {
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_base_damage(0.0)
            .headless()
            .spawn(commands)
    });
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should have BoltBaseDamage with zero value");
    assert!(
        dmg.0.abs() < f32::EPSILON,
        "BoltBaseDamage should be 0.0, got {}",
        dmg.0
    );
}

// ── Behavior 2: .with_base_damage() does not insert other definition components ──

#[test]
fn with_base_damage_does_not_insert_other_definition_components() {
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_base_damage(15.0)
            .headless()
            .spawn(commands)
    });
    assert!(
        world.get::<BoltDefinitionRef>(entity).is_none(),
        "entity should NOT have BoltDefinitionRef when only .with_base_damage() was called"
    );
    assert!(
        world.get::<BoltAngleSpread>(entity).is_none(),
        "entity should NOT have BoltAngleSpread when only .with_base_damage() was called"
    );
    assert!(
        world.get::<BoltSpawnOffsetY>(entity).is_none(),
        "entity should NOT have BoltSpawnOffsetY when only .with_base_damage() was called"
    );
}

// ── Section B: .with_definition_name() standalone ──────────────────────────

// ── Behavior 3: .with_definition_name() inserts BoltDefinitionRef on spawned entity ──

#[test]
fn with_definition_name_inserts_bolt_definition_ref() {
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_definition_name("CustomBolt".to_string())
            .headless()
            .spawn(commands)
    });
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should have BoltDefinitionRef");
    assert_eq!(
        def_ref.0, "CustomBolt",
        "BoltDefinitionRef should be 'CustomBolt', got '{}'",
        def_ref.0
    );
}

#[test]
fn with_definition_name_empty_string_accepted() {
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_definition_name(String::new())
            .headless()
            .spawn(commands)
    });
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should have BoltDefinitionRef with empty string");
    assert_eq!(
        def_ref.0, "",
        "BoltDefinitionRef should be empty, got '{}'",
        def_ref.0
    );
}

// ── Behavior 4: .with_definition_name() does not insert other definition components ──

#[test]
fn with_definition_name_does_not_insert_other_definition_components() {
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_definition_name("CustomBolt".to_string())
            .headless()
            .spawn(commands)
    });
    assert!(
        world.get::<BoltBaseDamage>(entity).is_none(),
        "entity should NOT have BoltBaseDamage when only .with_definition_name() was called"
    );
    assert!(
        world.get::<BoltAngleSpread>(entity).is_none(),
        "entity should NOT have BoltAngleSpread when only .with_definition_name() was called"
    );
    assert!(
        world.get::<BoltSpawnOffsetY>(entity).is_none(),
        "entity should NOT have BoltSpawnOffsetY when only .with_definition_name() was called"
    );
}

// ── Section C: .with_angle_spread() standalone ─────────────────────────────

// ── Behavior 5: .with_angle_spread() inserts BoltAngleSpread on spawned entity ──

#[test]
fn with_angle_spread_inserts_bolt_angle_spread() {
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_angle_spread(0.35)
            .headless()
            .spawn(commands)
    });
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should have BoltAngleSpread");
    assert!(
        (spread.0 - 0.35).abs() < f32::EPSILON,
        "BoltAngleSpread should be 0.35, got {}",
        spread.0
    );
}

#[test]
fn with_angle_spread_zero_is_valid() {
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_angle_spread(0.0)
            .headless()
            .spawn(commands)
    });
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should have BoltAngleSpread with zero value");
    assert!(
        spread.0.abs() < f32::EPSILON,
        "BoltAngleSpread should be 0.0, got {}",
        spread.0
    );
}

// ── Behavior 6: .with_angle_spread() does not insert other definition components ──

#[test]
fn with_angle_spread_does_not_insert_other_definition_components() {
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_angle_spread(0.35)
            .headless()
            .spawn(commands)
    });
    assert!(
        world.get::<BoltBaseDamage>(entity).is_none(),
        "entity should NOT have BoltBaseDamage when only .with_angle_spread() was called"
    );
    assert!(
        world.get::<BoltDefinitionRef>(entity).is_none(),
        "entity should NOT have BoltDefinitionRef when only .with_angle_spread() was called"
    );
    assert!(
        world.get::<BoltSpawnOffsetY>(entity).is_none(),
        "entity should NOT have BoltSpawnOffsetY when only .with_angle_spread() was called"
    );
}

// ── Section D: .with_spawn_offset_y() standalone ───────────────────────────

// ── Behavior 7: .with_spawn_offset_y() inserts BoltSpawnOffsetY on spawned entity ──

#[test]
fn with_spawn_offset_y_inserts_bolt_spawn_offset_y() {
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_spawn_offset_y(40.0)
            .headless()
            .spawn(commands)
    });
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should have BoltSpawnOffsetY");
    assert!(
        (offset.0 - 40.0).abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 40.0, got {}",
        offset.0
    );
}

#[test]
fn with_spawn_offset_y_zero_is_valid() {
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_spawn_offset_y(0.0)
            .headless()
            .spawn(commands)
    });
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should have BoltSpawnOffsetY with zero value");
    assert!(
        offset.0.abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 0.0, got {}",
        offset.0
    );
}

// ── Behavior 8: .with_spawn_offset_y() does not insert other definition components ──

#[test]
fn with_spawn_offset_y_does_not_insert_other_definition_components() {
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_spawn_offset_y(40.0)
            .headless()
            .spawn(commands)
    });
    assert!(
        world.get::<BoltBaseDamage>(entity).is_none(),
        "entity should NOT have BoltBaseDamage when only .with_spawn_offset_y() was called"
    );
    assert!(
        world.get::<BoltDefinitionRef>(entity).is_none(),
        "entity should NOT have BoltDefinitionRef when only .with_spawn_offset_y() was called"
    );
    assert!(
        world.get::<BoltAngleSpread>(entity).is_none(),
        "entity should NOT have BoltAngleSpread when only .with_spawn_offset_y() was called"
    );
}
