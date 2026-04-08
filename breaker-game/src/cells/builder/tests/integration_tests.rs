//! Section K: Full Integration -- Definition + Override + Optional

use bevy::{ecs::world::CommandQueue, prelude::*};
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{Position2D, Scale2D};

use crate::{
    cells::{
        components::{
            Cell, CellDamageVisuals, CellHealth, CellHeight, CellTypeAlias, CellWidth, Locked,
            Locks, RegenRate, RequiredToClear,
        },
        definition::{CellBehavior, CellTypeDefinition},
    },
    shared::{BOLT_LAYER, CELL_LAYER},
};

/// Creates a test `CellTypeDefinition` with known values.
fn test_cell_definition() -> CellTypeDefinition {
    CellTypeDefinition {
        id: "test".to_owned(),
        alias: "T".to_owned(),
        hp: 20.0,
        color_rgb: [1.0, 0.5, 0.2],
        required_to_clear: true,
        damage_hdr_base: 4.0,
        damage_green_min: 0.2,
        damage_blue_range: 0.4,
        damage_blue_base: 0.2,
        behaviors: None,
        shield: None,
        effects: None,
    }
}

/// Spawns a cell via Commands backed by a `CommandQueue`, then applies the queue.
fn spawn_cell_in_world(
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

// ── Behavior 44: Full definition with hp override and behavior addition ─────

#[test]
fn full_definition_with_hp_override_and_behavior() {
    let mut def = test_cell_definition();
    def.hp = 30.0;
    def.alias = "R".to_owned();
    def.color_rgb = [0.3, 4.0, 0.3];
    def.required_to_clear = true;
    def.damage_hdr_base = 4.0;
    def.damage_green_min = 0.4;
    def.damage_blue_range = 0.3;
    def.damage_blue_base = 0.1;
    def.behaviors = Some(vec![CellBehavior::Regen { rate: 2.0 }]);

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .override_hp(50.0)
            .with_behavior(CellBehavior::Regen { rate: 5.0 })
            .position(Vec2::new(50.0, 100.0))
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    // Override hp
    let health = world
        .get::<CellHealth>(entity)
        .expect("should have CellHealth");
    assert!(
        (health.current - 50.0).abs() < f32::EPSILON && (health.max - 50.0).abs() < f32::EPSILON,
        "CellHealth should be {{ current: 50.0, max: 50.0 }} (override), got {{ current: {}, max: {} }}",
        health.current,
        health.max
    );

    // From definition
    let visuals = world
        .get::<CellDamageVisuals>(entity)
        .expect("should have CellDamageVisuals");
    assert!(
        (visuals.hdr_base - 4.0).abs() < f32::EPSILON,
        "hdr_base should be 4.0 (from definition)"
    );
    assert!(
        (visuals.green_min - 0.4).abs() < f32::EPSILON,
        "green_min should be 0.4"
    );
    assert!(
        (visuals.blue_range - 0.3).abs() < f32::EPSILON,
        "blue_range should be 0.3"
    );
    assert!(
        (visuals.blue_base - 0.1).abs() < f32::EPSILON,
        "blue_base should be 0.1"
    );

    assert!(
        world.get::<RequiredToClear>(entity).is_some(),
        "should have RequiredToClear (from definition)"
    );

    let alias = world
        .get::<CellTypeAlias>(entity)
        .expect("should have CellTypeAlias");
    assert_eq!(alias.0, "R", "alias should be 'R' (from definition)");

    // Explicit behavior overwrites definition behavior — last write wins
    let regen = world
        .get::<RegenRate>(entity)
        .expect("should have RegenRate");
    assert!(
        (regen.0 - 5.0).abs() < f32::EPSILON,
        "RegenRate rate should be 5.0 (explicit overwrites definition), got {}",
        regen.0
    );

    // Core components
    assert!(
        world.get::<Cell>(entity).is_some(),
        "should have Cell marker"
    );
}

// Behavior 44 edge case: override hp to smaller value than definition
#[test]
fn full_definition_override_hp_smaller() {
    let mut def = test_cell_definition();
    def.hp = 30.0;

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .override_hp(10.0)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let health = world
        .get::<CellHealth>(entity)
        .expect("should have CellHealth");
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON && (health.max - 10.0).abs() < f32::EPSILON,
        "CellHealth should be {{ current: 10.0, max: 10.0 }} (override smaller)"
    );
}

// ── Behavior 45: Minimal builder — no definition, no optionals ──────────────

#[test]
fn minimal_builder_no_definition_no_optionals() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(40.0, 20.0)
            .hp(10.0)
            .headless()
            .spawn(commands)
    });

    // Core components
    assert!(world.get::<Cell>(entity).is_some(), "should have Cell");

    let pos = world
        .get::<Position2D>(entity)
        .expect("should have Position2D");
    assert_eq!(pos.0, Vec2::ZERO);

    let width = world
        .get::<CellWidth>(entity)
        .expect("should have CellWidth");
    assert!(
        (width.value - 40.0).abs() < f32::EPSILON,
        "CellWidth should be 40.0"
    );

    let height = world
        .get::<CellHeight>(entity)
        .expect("should have CellHeight");
    assert!(
        (height.value - 20.0).abs() < f32::EPSILON,
        "CellHeight should be 20.0"
    );

    let scale = world.get::<Scale2D>(entity).expect("should have Scale2D");
    assert!(
        (scale.x - 40.0).abs() < f32::EPSILON && (scale.y - 20.0).abs() < f32::EPSILON,
        "Scale2D should be (40.0, 20.0)"
    );

    let aabb = world.get::<Aabb2D>(entity).expect("should have Aabb2D");
    assert!(
        (aabb.half_extents.x - 20.0).abs() < f32::EPSILON
            && (aabb.half_extents.y - 10.0).abs() < f32::EPSILON,
        "Aabb2D half_extents should be (20.0, 10.0)"
    );

    let health = world
        .get::<CellHealth>(entity)
        .expect("should have CellHealth");
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON && (health.max - 10.0).abs() < f32::EPSILON,
        "CellHealth should be {{ current: 10.0, max: 10.0 }}"
    );

    let layers = world
        .get::<CollisionLayers>(entity)
        .expect("should have CollisionLayers");
    assert_eq!(layers.membership, CELL_LAYER);
    assert_eq!(layers.mask, BOLT_LAYER);

    // No optional components
    assert!(
        world.get::<CellDamageVisuals>(entity).is_none(),
        "should NOT have CellDamageVisuals"
    );
    assert!(
        world.get::<CellTypeAlias>(entity).is_none(),
        "should NOT have CellTypeAlias"
    );
    assert!(
        world.get::<RequiredToClear>(entity).is_none(),
        "should NOT have RequiredToClear"
    );
    assert!(
        world.get::<RegenRate>(entity).is_none(),
        "should NOT have RegenRate"
    );
    assert!(
        world.get::<Locked>(entity).is_none(),
        "should NOT have Locked"
    );
    assert!(
        world.get::<Locks>(entity).is_none(),
        "should NOT have Locks"
    );
}

// ── Behavior 46: Builder with locked and definition ─────────────────────────

#[test]
fn builder_with_locked_and_definition() {
    let mut def = test_cell_definition();
    def.required_to_clear = true;

    let mut world = World::new();
    let e1 = world.spawn_empty().id();
    let e2 = world.spawn_empty().id();
    let e3 = world.spawn_empty().id();

    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::new(0.0, 200.0))
            .dimensions(70.0, 24.0)
            .locked(vec![e1, e2, e3])
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<Locked>(entity).is_some(), "should have Locked");
    let adjacents = world.get::<Locks>(entity).expect("should have Locks");
    assert_eq!(adjacents.0.len(), 3);
    assert_eq!(adjacents.0[0], e1);
    assert_eq!(adjacents.0[1], e2);
    assert_eq!(adjacents.0[2], e3);

    assert!(
        world.get::<RequiredToClear>(entity).is_some(),
        "should have RequiredToClear (from definition)"
    );
}

// Behavior 46 edge case: locked with required_to_clear false
#[test]
fn builder_locked_with_required_false_still_has_lock() {
    let mut def = test_cell_definition();
    def.required_to_clear = false;

    let mut world = World::new();
    let e1 = world.spawn_empty().id();

    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .locked(vec![e1])
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<Locked>(entity).is_some(),
        "should have Locked even with required_to_clear: false"
    );
    assert!(
        world.get::<RequiredToClear>(entity).is_none(),
        "should NOT have RequiredToClear when definition has required_to_clear: false"
    );
}

// ── Behavior 2: Cell::builder() defaults produce a cell with no optional components ─

#[test]
fn builder_defaults_no_optional_components() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    // Guard: non-#[require] component ensures builder actually populated the entity
    let health = world
        .get::<CellHealth>(entity)
        .expect("entity should have CellHealth from builder");
    assert!(
        (health.current - 20.0).abs() < f32::EPSILON,
        "CellHealth.current should be 20.0, not default"
    );

    assert!(
        world.get::<CellTypeAlias>(entity).is_none(),
        "should NOT have CellTypeAlias"
    );
    assert!(
        world.get::<RequiredToClear>(entity).is_none(),
        "should NOT have RequiredToClear"
    );
    assert!(
        world.get::<CellDamageVisuals>(entity).is_none(),
        "should NOT have CellDamageVisuals"
    );
    assert!(
        world.get::<RegenRate>(entity).is_none(),
        "should NOT have RegenRate"
    );
    assert!(
        world.get::<Locked>(entity).is_none(),
        "should NOT have Locked"
    );
    assert!(
        world.get::<Locks>(entity).is_none(),
        "should NOT have Locks"
    );
}
