//! Section H: `spawn()` terminal method — headless
//! Section I: `spawn()` terminal method — rendered
//! Section J: `spawn_inner()` behavior insertion

use bevy::{ecs::world::CommandQueue, prelude::*};
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{Position2D, Scale2D, Spatial2D};
use rantzsoft_stateflow::CleanupOnExit;

use crate::{
    cells::{
        components::{
            Cell, CellDamageVisuals, CellHealth, CellHeight, CellRegen, CellTypeAlias, CellWidth,
            LockAdjacents, Locked, RequiredToClear,
        },
        definition::{CellBehavior, CellTypeDefinition},
    },
    shared::{BOLT_LAYER, CELL_LAYER, GameDrawLayer},
    state::types::NodeState,
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

// ── Section H: spawn() — Headless ──────────────────────────────────────────

// Behavior 34: spawn() on headless cell creates entity with core components
#[test]
fn spawn_headless_has_cell_marker_and_core_components() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::new(50.0, 100.0))
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<Cell>(entity).is_some(),
        "entity should have Cell marker"
    );
    // Guard: also check a non-#[require] component to ensure builder actually ran
    let health = world
        .get::<CellHealth>(entity)
        .expect("entity should have CellHealth from builder");
    assert!(
        (health.current - 20.0).abs() < f32::EPSILON,
        "CellHealth.current should be 20.0"
    );
}

#[test]
fn spawn_headless_has_position() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::new(50.0, 100.0))
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let pos = world
        .get::<Position2D>(entity)
        .expect("entity should have Position2D");
    assert!(
        (pos.0.x - 50.0).abs() < f32::EPSILON && (pos.0.y - 100.0).abs() < f32::EPSILON,
        "Position2D should be (50.0, 100.0), got {:?}",
        pos.0
    );
}

#[test]
fn spawn_headless_has_scale() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::new(50.0, 100.0))
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let scale = world
        .get::<Scale2D>(entity)
        .expect("entity should have Scale2D");
    assert!(
        (scale.x - 70.0).abs() < f32::EPSILON && (scale.y - 24.0).abs() < f32::EPSILON,
        "Scale2D should be (70.0, 24.0), got ({}, {})",
        scale.x,
        scale.y
    );
}

#[test]
fn spawn_headless_has_aabb() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::new(50.0, 100.0))
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let aabb = world
        .get::<Aabb2D>(entity)
        .expect("entity should have Aabb2D");
    assert!(
        (aabb.half_extents.x - 35.0).abs() < f32::EPSILON
            && (aabb.half_extents.y - 12.0).abs() < f32::EPSILON,
        "Aabb2D half_extents should be (35.0, 12.0), got {:?}",
        aabb.half_extents
    );
}

#[test]
fn spawn_headless_has_cell_width_and_height() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::new(50.0, 100.0))
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let width = world
        .get::<CellWidth>(entity)
        .expect("entity should have CellWidth");
    assert!(
        (width.value - 70.0).abs() < f32::EPSILON,
        "CellWidth should be 70.0"
    );

    let height = world
        .get::<CellHeight>(entity)
        .expect("entity should have CellHeight");
    assert!(
        (height.value - 24.0).abs() < f32::EPSILON,
        "CellHeight should be 24.0"
    );
}

#[test]
fn spawn_headless_has_health() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::new(50.0, 100.0))
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let health = world
        .get::<CellHealth>(entity)
        .expect("entity should have CellHealth");
    assert!(
        (health.current - 20.0).abs() < f32::EPSILON && (health.max - 20.0).abs() < f32::EPSILON,
        "CellHealth should be {{ current: 20.0, max: 20.0 }}"
    );
}

#[test]
fn spawn_headless_has_collision_layers() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::new(50.0, 100.0))
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let layers = world
        .get::<CollisionLayers>(entity)
        .expect("entity should have CollisionLayers");
    assert_eq!(
        layers.membership, CELL_LAYER,
        "membership should be CELL_LAYER"
    );
    assert_eq!(layers.mask, BOLT_LAYER, "mask should be BOLT_LAYER");
}

#[test]
fn spawn_headless_has_spatial2d_and_correct_position() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::new(50.0, 100.0))
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<Spatial2D>(entity).is_some(),
        "entity should have Spatial2D (via Cell #[require])"
    );
    // Guard: verify builder set the position explicitly, not just the #[require] default
    let pos = world
        .get::<Position2D>(entity)
        .expect("entity should have Position2D");
    assert!(
        (pos.0.x - 50.0).abs() < f32::EPSILON && (pos.0.y - 100.0).abs() < f32::EPSILON,
        "Position2D should be (50.0, 100.0), not the Spatial2D default"
    );
}

#[test]
fn spawn_headless_has_cleanup_on_exit_and_health() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::new(50.0, 100.0))
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<CleanupOnExit<NodeState>>(entity).is_some(),
        "entity should have CleanupOnExit<NodeState> (via Cell #[require])"
    );
    // Guard: also check non-#[require] component
    assert!(
        world.get::<CellHealth>(entity).is_some(),
        "entity should have CellHealth from builder"
    );
}

// Behavior 34 edge case: headless cell does NOT have visual components
#[test]
fn spawn_headless_has_no_visual_components() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::new(50.0, 100.0))
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    // Guard: non-#[require] component ensures builder ran
    assert!(
        world.get::<CellHealth>(entity).is_some(),
        "entity should have CellHealth from builder"
    );

    assert!(
        world.get::<Mesh2d>(entity).is_none(),
        "headless cell should NOT have Mesh2d"
    );
    assert!(
        world.get::<GameDrawLayer>(entity).is_none(),
        "headless cell should NOT have GameDrawLayer"
    );
}

// Behavior 35: spawn() returns the Entity id
#[test]
fn spawn_returns_valid_entity_with_core_components() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get_entity(entity).is_ok(),
        "spawned entity should exist in world"
    );
    assert!(
        world.get::<Cell>(entity).is_some(),
        "spawned entity should have Cell marker"
    );
    // Guard: non-#[require] component ensures builder actually populated the entity
    assert!(
        world.get::<CellHealth>(entity).is_some(),
        "spawned entity should have CellHealth from builder"
    );
    assert!(
        world.get::<CollisionLayers>(entity).is_some(),
        "spawned entity should have CollisionLayers from builder"
    );
}

// Behavior 36: spawn() on headless cell with .definition() creates entity with definition-derived components
#[test]
fn spawn_headless_with_definition_has_all_definition_components() {
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
            .position(Vec2::new(50.0, 100.0))
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    // Core components
    assert!(world.get::<Cell>(entity).is_some(), "should have Cell");
    let pos = world
        .get::<Position2D>(entity)
        .expect("should have Position2D");
    assert!(
        (pos.0.x - 50.0).abs() < f32::EPSILON && (pos.0.y - 100.0).abs() < f32::EPSILON,
        "Position2D should be (50.0, 100.0)"
    );

    // Definition-derived components
    let health = world
        .get::<CellHealth>(entity)
        .expect("should have CellHealth");
    assert!(
        (health.current - 30.0).abs() < f32::EPSILON && (health.max - 30.0).abs() < f32::EPSILON,
        "CellHealth should be {{ current: 30.0, max: 30.0 }}"
    );

    let visuals = world
        .get::<CellDamageVisuals>(entity)
        .expect("should have CellDamageVisuals");
    assert!(
        (visuals.hdr_base - 4.0).abs() < f32::EPSILON,
        "hdr_base should be 4.0"
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
        "should have RequiredToClear"
    );

    let alias = world
        .get::<CellTypeAlias>(entity)
        .expect("should have CellTypeAlias");
    assert_eq!(alias.0, "R");

    let regen = world
        .get::<CellRegen>(entity)
        .expect("should have CellRegen");
    assert!(
        (regen.rate - 2.0).abs() < f32::EPSILON,
        "CellRegen rate should be 2.0"
    );
}

// Behavior 36 edge case: definition with required_to_clear false
#[test]
fn spawn_headless_definition_required_false_no_marker() {
    let mut def = test_cell_definition();
    def.required_to_clear = false;

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    // Guard: CellHealth ensures builder ran with definition hp
    let health = world
        .get::<CellHealth>(entity)
        .expect("should have CellHealth from definition");
    assert!(
        (health.current - 20.0).abs() < f32::EPSILON,
        "CellHealth should come from definition hp"
    );

    assert!(
        world.get::<RequiredToClear>(entity).is_none(),
        "should NOT have RequiredToClear when definition has required_to_clear: false"
    );
}

// Behavior 36 edge case: definition with behaviors None
#[test]
fn spawn_headless_definition_behaviors_none_no_regen() {
    let mut def = test_cell_definition();
    def.behaviors = None;

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    // Guard: CellHealth ensures builder ran with definition hp
    let health = world
        .get::<CellHealth>(entity)
        .expect("should have CellHealth from definition");
    assert!(
        (health.current - 20.0).abs() < f32::EPSILON,
        "CellHealth should come from definition hp"
    );

    assert!(
        world.get::<CellRegen>(entity).is_none(),
        "should NOT have CellRegen when definition has behaviors: None"
    );
}

// ── Section I: spawn() — Rendered ──────────────────────────────────────────

// Behavior 37: spawn() on rendered cell creates entity with visual components
#[test]
fn spawn_rendered_has_visual_components() {
    let mut world = World::new();
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<ColorMaterial>::default();

    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::new(50.0, 100.0))
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .rendered(&mut meshes, &mut materials)
            .spawn(commands)
    });

    assert!(
        world.get::<Cell>(entity).is_some(),
        "entity should have Cell marker"
    );
    assert!(
        world.get::<Mesh2d>(entity).is_some(),
        "rendered cell should have Mesh2d"
    );
    assert!(
        world.get::<GameDrawLayer>(entity).is_some(),
        "rendered cell should have GameDrawLayer"
    );
    let draw_layer = world.get::<GameDrawLayer>(entity).unwrap();
    assert!(
        matches!(draw_layer, GameDrawLayer::Cell),
        "GameDrawLayer should be Cell"
    );
}

// ── Section J: spawn_inner() Behavior Insertion ─────────────────────────────

// Behavior 41: CellBehavior::Regen { rate } inserts CellRegen component
#[test]
fn spawn_inner_regen_behavior_inserts_cell_regen() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_behavior(CellBehavior::Regen { rate: 2.0 })
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let regen = world
        .get::<CellRegen>(entity)
        .expect("entity should have CellRegen");
    assert!(
        (regen.rate - 2.0).abs() < f32::EPSILON,
        "CellRegen rate should be 2.0"
    );
}

// Behavior 41 edge case: multiple regen behaviors — last write wins
#[test]
fn spawn_inner_multiple_regen_last_write_wins() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![CellBehavior::Regen { rate: 2.0 }]);

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .with_behavior(CellBehavior::Regen { rate: 5.0 })
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let regen = world
        .get::<CellRegen>(entity)
        .expect("entity should have CellRegen");
    assert!(
        (regen.rate - 5.0).abs() < f32::EPSILON,
        "CellRegen rate should be 5.0 (last write wins)"
    );
}

// Behavior 42: Empty behaviors vec inserts no behavior components
#[test]
fn spawn_inner_empty_behaviors_no_regen() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    // Guard: non-#[require] component
    assert!(
        world.get::<CellHealth>(entity).is_some(),
        "entity should have CellHealth from builder"
    );
    assert!(
        world.get::<CellRegen>(entity).is_none(),
        "entity should NOT have CellRegen without behaviors"
    );
}

// Behavior 43: .locked(entities) inserts Locked and LockAdjacents via spawn_inner
#[test]
fn spawn_inner_locked_inserts_markers() {
    let mut world = World::new();
    let e1 = world.spawn_empty().id();
    let e2 = world.spawn_empty().id();

    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .locked(vec![e1, e2])
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<Locked>(entity).is_some(),
        "entity should have Locked marker"
    );
    let adjacents = world
        .get::<LockAdjacents>(entity)
        .expect("entity should have LockAdjacents");
    assert_eq!(adjacents.0.len(), 2);
    assert_eq!(adjacents.0[0], e1);
    assert_eq!(adjacents.0[1], e2);
}

// Behavior 43 edge case: locked with single entity
#[test]
fn spawn_inner_locked_single_entity() {
    let mut world = World::new();
    let e1 = world.spawn_empty().id();

    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .locked(vec![e1])
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let adjacents = world
        .get::<LockAdjacents>(entity)
        .expect("entity should have LockAdjacents");
    assert_eq!(adjacents.0.len(), 1);
    assert_eq!(adjacents.0[0], e1);
}

// ── Section I (continued): Rendered Cell Color Tests ────────────────────────

// Behavior 38: Rendered cell uses default white color when no definition or color_rgb override
#[test]
fn spawn_rendered_default_white_color() {
    let mut world = World::new();
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<ColorMaterial>::default();

    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .rendered(&mut meshes, &mut materials)
            .spawn(commands)
    });

    // Get the MeshMaterial2d handle
    let mat_handle = world
        .get::<MeshMaterial2d<ColorMaterial>>(entity)
        .expect("rendered cell should have MeshMaterial2d");
    let material = materials
        .get(&mat_handle.0)
        .expect("material handle should be valid");
    let color = material.color.to_linear();
    assert!(
        (color.red - 1.0).abs() < f32::EPSILON
            && (color.green - 1.0).abs() < f32::EPSILON
            && (color.blue - 1.0).abs() < f32::EPSILON,
        "default color should be white (1.0, 1.0, 1.0), got ({}, {}, {})",
        color.red,
        color.green,
        color.blue
    );
}

// Behavior 39: Rendered cell uses definition color when definition is set
#[test]
fn spawn_rendered_definition_color() {
    let mut def = test_cell_definition();
    def.color_rgb = [0.3, 4.0, 0.3];

    let mut world = World::new();
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<ColorMaterial>::default();

    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .rendered(&mut meshes, &mut materials)
            .spawn(commands)
    });

    let mat_handle = world
        .get::<MeshMaterial2d<ColorMaterial>>(entity)
        .expect("rendered cell should have MeshMaterial2d");
    let material = materials
        .get(&mat_handle.0)
        .expect("material handle should be valid");
    let color = material.color.to_linear();
    assert!(
        (color.red - 0.3).abs() < 1e-3
            && (color.green - 4.0).abs() < 1e-3
            && (color.blue - 0.3).abs() < 1e-3,
        "color should be from definition (0.3, 4.0, 0.3), got ({}, {}, {})",
        color.red,
        color.green,
        color.blue
    );
}

// Behavior 40: Rendered cell uses color_rgb override when both definition and override are set
#[test]
fn spawn_rendered_color_override_wins() {
    let mut def = test_cell_definition();
    def.color_rgb = [0.3, 4.0, 0.3];

    let mut world = World::new();
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<ColorMaterial>::default();

    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .color_rgb([1.0, 2.0, 3.0])
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .rendered(&mut meshes, &mut materials)
            .spawn(commands)
    });

    let mat_handle = world
        .get::<MeshMaterial2d<ColorMaterial>>(entity)
        .expect("rendered cell should have MeshMaterial2d");
    let material = materials
        .get(&mat_handle.0)
        .expect("material handle should be valid");
    let color = material.color.to_linear();
    assert!(
        (color.red - 1.0).abs() < 1e-3
            && (color.green - 2.0).abs() < 1e-3
            && (color.blue - 3.0).abs() < 1e-3,
        "color should be override (1.0, 2.0, 3.0), got ({}, {}, {})",
        color.red,
        color.green,
        color.blue
    );
}
