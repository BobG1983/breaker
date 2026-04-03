use bevy::{ecs::world::CommandQueue, prelude::*};
use rantzsoft_spatial2d::components::Velocity2D;

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

// ── Section E: Override semantics — .with_*() after .definition() ──────────

// ── Behavior 9: .with_base_damage() overrides definition's base_damage ──

#[test]
fn with_base_damage_overrides_definition_base_damage() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_base_damage(25.0)
            .headless()
            .spawn(commands)
    });
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should have BoltBaseDamage");
    assert!(
        (dmg.0 - 25.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 25.0 (override), not 10.0 (definition), got {}",
        dmg.0
    );
    // Other definition fields remain intact
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should still have BoltDefinitionRef from definition");
    assert_eq!(def_ref.0, "Bolt");
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should still have BoltAngleSpread from definition");
    assert!(
        (spread.0 - 0.524).abs() < f32::EPSILON,
        "BoltAngleSpread should be 0.524 from definition, got {}",
        spread.0
    );
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should still have BoltSpawnOffsetY from definition");
    assert!(
        (offset.0 - 54.0).abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 54.0 from definition, got {}",
        offset.0
    );
}

#[test]
fn with_base_damage_zero_overrides_definition_base_damage() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_base_damage(0.0)
            .headless()
            .spawn(commands)
    });
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should have BoltBaseDamage");
    assert!(
        dmg.0.abs() < f32::EPSILON,
        "BoltBaseDamage should be 0.0 (zero override wins), got {}",
        dmg.0
    );
}

// ── Behavior 10: .with_definition_name() overrides definition's name ──

#[test]
fn with_definition_name_overrides_definition_name() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_definition_name("OverrideName".to_string())
            .headless()
            .spawn(commands)
    });
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should have BoltDefinitionRef");
    assert_eq!(
        def_ref.0, "OverrideName",
        "BoltDefinitionRef should be 'OverrideName' (override), not 'Bolt' (definition), got '{}'",
        def_ref.0
    );
    // Other definition fields remain intact
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should still have BoltBaseDamage from definition");
    assert!(
        (dmg.0 - 10.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 10.0 from definition, got {}",
        dmg.0
    );
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should still have BoltAngleSpread from definition");
    assert!(
        (spread.0 - 0.524).abs() < f32::EPSILON,
        "BoltAngleSpread should be 0.524 from definition, got {}",
        spread.0
    );
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should still have BoltSpawnOffsetY from definition");
    assert!(
        (offset.0 - 54.0).abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 54.0 from definition, got {}",
        offset.0
    );
}

#[test]
fn with_definition_name_empty_overrides_definition_name() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_definition_name(String::new())
            .headless()
            .spawn(commands)
    });
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should have BoltDefinitionRef");
    assert_eq!(
        def_ref.0, "",
        "BoltDefinitionRef should be '' (empty override wins), got '{}'",
        def_ref.0
    );
}

// ── Behavior 11: .with_angle_spread() overrides definition's default angle spread ──

#[test]
fn with_angle_spread_overrides_definition_angle_spread() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_angle_spread(1.0)
            .headless()
            .spawn(commands)
    });
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should have BoltAngleSpread");
    assert!(
        (spread.0 - 1.0).abs() < f32::EPSILON,
        "BoltAngleSpread should be 1.0 (override), not 0.524 (definition), got {}",
        spread.0
    );
    // Other definition fields remain intact
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should still have BoltBaseDamage from definition");
    assert!((dmg.0 - 10.0).abs() < f32::EPSILON);
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should still have BoltDefinitionRef from definition");
    assert_eq!(def_ref.0, "Bolt");
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should still have BoltSpawnOffsetY from definition");
    assert!((offset.0 - 54.0).abs() < f32::EPSILON);
}

#[test]
fn with_angle_spread_zero_overrides_definition_angle_spread() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_angle_spread(0.0)
            .headless()
            .spawn(commands)
    });
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should have BoltAngleSpread");
    assert!(
        spread.0.abs() < f32::EPSILON,
        "BoltAngleSpread should be 0.0 (zero override wins), got {}",
        spread.0
    );
}

// ── Behavior 12: .with_spawn_offset_y() overrides definition's default spawn offset ──

#[test]
fn with_spawn_offset_y_overrides_definition_spawn_offset() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_spawn_offset_y(30.0)
            .headless()
            .spawn(commands)
    });
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should have BoltSpawnOffsetY");
    assert!(
        (offset.0 - 30.0).abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 30.0 (override), not 54.0 (definition), got {}",
        offset.0
    );
    // Other definition fields remain intact
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should still have BoltBaseDamage from definition");
    assert!((dmg.0 - 10.0).abs() < f32::EPSILON);
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should still have BoltDefinitionRef from definition");
    assert_eq!(def_ref.0, "Bolt");
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should still have BoltAngleSpread from definition");
    assert!((spread.0 - 0.524).abs() < f32::EPSILON);
}

#[test]
fn with_spawn_offset_y_zero_overrides_definition_spawn_offset() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_spawn_offset_y(0.0)
            .headless()
            .spawn(commands)
    });
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should have BoltSpawnOffsetY");
    assert!(
        offset.0.abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 0.0 (zero override wins), got {}",
        offset.0
    );
}

// ── Section F: All four .with_*() methods combined (no definition) ─────────

// ── Behavior 13: All four .with_*() methods together replace .definition() for definition-related fields ──

#[test]
fn all_four_with_methods_together_without_definition() {
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .with_speed(500.0, 250.0, 1000.0)
            .with_angle(0.1, 0.1)
            .at_position(Vec2::new(100.0, 200.0))
            .with_velocity(Velocity2D(Vec2::new(0.0, 500.0)))
            .extra()
            .with_base_damage(20.0)
            .with_definition_name("SyntheticBolt".to_string())
            .with_angle_spread(0.6)
            .with_spawn_offset_y(45.0)
            .headless()
            .spawn(commands)
    });
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should have BoltBaseDamage");
    assert!(
        (dmg.0 - 20.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 20.0, got {}",
        dmg.0
    );
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should have BoltDefinitionRef");
    assert_eq!(def_ref.0, "SyntheticBolt");
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should have BoltAngleSpread");
    assert!(
        (spread.0 - 0.6).abs() < f32::EPSILON,
        "BoltAngleSpread should be 0.6, got {}",
        spread.0
    );
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should have BoltSpawnOffsetY");
    assert!(
        (offset.0 - 45.0).abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 45.0, got {}",
        offset.0
    );
}

#[test]
fn all_four_with_methods_plus_definition_overrides_win() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = spawn_bolt_in_world(&mut world, |commands| {
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_base_damage(99.0)
            .with_definition_name("AllOverride".to_string())
            .with_angle_spread(2.0)
            .with_spawn_offset_y(100.0)
            .headless()
            .spawn(commands)
    });
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should have BoltBaseDamage");
    assert!(
        (dmg.0 - 99.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 99.0 (override), got {}",
        dmg.0
    );
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should have BoltDefinitionRef");
    assert_eq!(
        def_ref.0, "AllOverride",
        "BoltDefinitionRef should be 'AllOverride' (override), got '{}'",
        def_ref.0
    );
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should have BoltAngleSpread");
    assert!(
        (spread.0 - 2.0).abs() < f32::EPSILON,
        "BoltAngleSpread should be 2.0 (override), got {}",
        spread.0
    );
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should have BoltSpawnOffsetY");
    assert!(
        (offset.0 - 100.0).abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 100.0 (override), got {}",
        offset.0
    );
}
