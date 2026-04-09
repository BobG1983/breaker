//! Section I: `.guarded()` Builder Integration

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::{
    components::{Position2D, Scale2D},
    propagation::PositionPropagation,
};

use super::helpers::*;
use crate::{
    cells::{
        builder::core::types::GuardianSpawnConfig,
        components::{
            Cell, CellHealth, CellHeight, CellWidth, GuardedCell, GuardianCell, GuardianGridStep,
            GuardianSlideSpeed, GuardianSlot, SlideTarget,
        },
        definition::{CellBehavior, GuardedBehavior},
    },
    shared::{BOLT_LAYER, CELL_LAYER},
};

// Behavior 32: .guarded() stores guardian spawn data (compile test)
#[test]
fn guarded_method_available_in_any_typestate() {
    let _builder = Cell::builder().guarded(vec![0, 1], test_guardian_config());
    // Compiles — that is the assertion.
}

// Behavior 33 (I): spawn() with .guarded() creates parent with GuardedCell marker
#[test]
fn spawn_guarded_parent_has_guarded_cell_marker() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::new(100.0, 200.0))
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .guarded(vec![0, 2], test_guardian_config())
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<GuardedCell>(entity).is_some(),
        "parent entity should have GuardedCell component"
    );
    // Parent also has all normal cell components
    assert!(
        world.get::<Cell>(entity).is_some(),
        "parent should have Cell marker"
    );
    assert!(
        world.get::<CellHealth>(entity).is_some(),
        "parent should have CellHealth"
    );
}

// Behavior 34 (I): spawn() with .guarded() creates guardian children with correct components
#[test]
fn spawn_guarded_creates_guardian_children_with_correct_components() {
    let mut world = World::new();
    let _parent = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::new(100.0, 200.0))
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .guarded(vec![0, 3], test_guardian_config())
            .headless()
            .spawn(commands)
    });

    // Count guardian children
    let guardians: Vec<(Entity, &GuardianCell)> = world
        .query::<(Entity, &GuardianCell)>()
        .iter(&world)
        .collect();
    assert_eq!(
        guardians.len(),
        2,
        "should have 2 guardian children, got {}",
        guardians.len()
    );

    // Check each guardian has required components
    for (entity, _) in &guardians {
        assert!(
            world.get::<Cell>(*entity).is_some(),
            "guardian should have Cell marker"
        );
        assert!(
            world.get::<CellHealth>(*entity).is_some(),
            "guardian should have CellHealth"
        );
        assert!(
            world.get::<Position2D>(*entity).is_some(),
            "guardian should have Position2D"
        );
        assert!(
            world.get::<GuardianSlot>(*entity).is_some(),
            "guardian should have GuardianSlot"
        );
        assert!(
            world.get::<SlideTarget>(*entity).is_some(),
            "guardian should have SlideTarget"
        );
        assert!(
            world.get::<GuardianSlideSpeed>(*entity).is_some(),
            "guardian should have GuardianSlideSpeed"
        );
        assert!(
            world.get::<GuardianGridStep>(*entity).is_some(),
            "guardian should have GuardianGridStep"
        );
        assert!(
            world.get::<ChildOf>(*entity).is_some(),
            "guardian should have ChildOf(parent)"
        );
    }
}

// Behavior 34: guardian positions are correct based on ring slot offsets
#[test]
fn guardian_positions_match_ring_slot_offsets() {
    let mut world = World::new();
    let _parent = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::new(100.0, 200.0))
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .guarded(vec![0, 3], test_guardian_config())
            .headless()
            .spawn(commands)
    });

    // Find guardian at slot 0
    let slot0_guardians: Vec<&Position2D> = world
        .query_filtered::<(&GuardianSlot, &Position2D), With<GuardianCell>>()
        .iter(&world)
        .filter(|(slot, _)| slot.0 == 0)
        .map(|(_, pos)| pos)
        .collect();
    assert_eq!(
        slot0_guardians.len(),
        1,
        "should have one guardian at slot 0"
    );
    let pos0 = slot0_guardians[0];
    // Slot 0 offset is (-1.0, 1.0), world pos = (100 + (-1)*72, 200 + 1*26) = (28.0, 226.0)
    assert!(
        (pos0.0.x - 28.0).abs() < f32::EPSILON && (pos0.0.y - 226.0).abs() < f32::EPSILON,
        "guardian at slot 0 should be at (28.0, 226.0), got {:?}",
        pos0.0
    );

    // Find guardian at slot 3
    let slot3_guardians: Vec<&Position2D> = world
        .query_filtered::<(&GuardianSlot, &Position2D), With<GuardianCell>>()
        .iter(&world)
        .filter(|(slot, _)| slot.0 == 3)
        .map(|(_, pos)| pos)
        .collect();
    assert_eq!(
        slot3_guardians.len(),
        1,
        "should have one guardian at slot 3"
    );
    let pos3 = slot3_guardians[0];
    // Slot 3 offset is (1.0, 0.0), world pos = (100 + 1*72, 200 + 0*26) = (172.0, 200.0)
    assert!(
        (pos3.0.x - 172.0).abs() < f32::EPSILON && (pos3.0.y - 200.0).abs() < f32::EPSILON,
        "guardian at slot 3 should be at (172.0, 200.0), got {:?}",
        pos3.0
    );
}

// Behavior 34: guardian HP comes from config
#[test]
fn guardian_health_matches_config() {
    let mut world = World::new();
    let _parent = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::new(100.0, 200.0))
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .guarded(vec![0], test_guardian_config())
            .headless()
            .spawn(commands)
    });

    let guardian_health: Vec<&CellHealth> = world
        .query_filtered::<&CellHealth, With<GuardianCell>>()
        .iter(&world)
        .collect();
    assert_eq!(guardian_health.len(), 1);
    assert!(
        (guardian_health[0].current - 10.0).abs() < f32::EPSILON,
        "guardian CellHealth.current should be 10.0, got {}",
        guardian_health[0].current
    );
    assert!(
        (guardian_health[0].max - 10.0).abs() < f32::EPSILON,
        "guardian CellHealth.max should be 10.0, got {}",
        guardian_health[0].max
    );
}

// Behavior 34: guardian has SlideTarget = next clockwise slot
#[test]
fn guardian_initial_slide_target_is_next_clockwise() {
    let mut world = World::new();
    let _parent = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::new(100.0, 200.0))
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .guarded(vec![0, 3], test_guardian_config())
            .headless()
            .spawn(commands)
    });

    // Slot 0 -> SlideTarget(1)
    let targets: Vec<(&GuardianSlot, &SlideTarget)> = world
        .query_filtered::<(&GuardianSlot, &SlideTarget), With<GuardianCell>>()
        .iter(&world)
        .collect();
    for (slot, target) in &targets {
        let expected_target = (slot.0 + 1) % 8;
        assert_eq!(
            target.0, expected_target,
            "guardian at slot {} should have SlideTarget({}), got {}",
            slot.0, expected_target, target.0
        );
    }
}

// Behavior 34: guardian has PositionPropagation::Absolute
#[test]
fn guardian_has_absolute_position_propagation() {
    let mut world = World::new();
    let _parent = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .guarded(vec![0], test_guardian_config())
            .headless()
            .spawn(commands)
    });

    let props: Vec<&PositionPropagation> = world
        .query_filtered::<&PositionPropagation, With<GuardianCell>>()
        .iter(&world)
        .collect();
    assert_eq!(props.len(), 1, "should have one guardian");
    assert_eq!(
        *props[0],
        PositionPropagation::Absolute,
        "guardian should have PositionPropagation::Absolute"
    );
}

// Behavior 34: guardian collision layers
#[test]
fn guardian_has_collision_layers() {
    let mut world = World::new();
    let _parent = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .guarded(vec![0], test_guardian_config())
            .headless()
            .spawn(commands)
    });

    let layers: Vec<&CollisionLayers> = world
        .query_filtered::<&CollisionLayers, With<GuardianCell>>()
        .iter(&world)
        .collect();
    assert_eq!(layers.len(), 1);
    assert_eq!(
        layers[0].membership, CELL_LAYER,
        "guardian membership should be CELL_LAYER"
    );
    assert_eq!(
        layers[0].mask, BOLT_LAYER,
        "guardian mask should be BOLT_LAYER"
    );
}

// Behavior 35: Guardian dimensions are square (cell_height x cell_height)
#[test]
fn guardian_dimensions_are_square() {
    let mut world = World::new();
    let _parent = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0) // parent is rectangular
            .hp(20.0)
            .guarded(vec![0], test_guardian_config()) // cell_height = 24.0
            .headless()
            .spawn(commands)
    });

    let scales: Vec<&Scale2D> = world
        .query_filtered::<&Scale2D, With<GuardianCell>>()
        .iter(&world)
        .collect();
    assert_eq!(scales.len(), 1);
    assert!(
        (scales[0].x - 24.0).abs() < f32::EPSILON && (scales[0].y - 24.0).abs() < f32::EPSILON,
        "guardian Scale2D should be (24.0, 24.0), got ({}, {})",
        scales[0].x,
        scales[0].y
    );

    let aabbs: Vec<&Aabb2D> = world
        .query_filtered::<&Aabb2D, With<GuardianCell>>()
        .iter(&world)
        .collect();
    assert_eq!(aabbs.len(), 1);
    assert!(
        (aabbs[0].half_extents.x - 12.0).abs() < f32::EPSILON
            && (aabbs[0].half_extents.y - 12.0).abs() < f32::EPSILON,
        "guardian Aabb2D half_extents should be (12.0, 12.0), got {:?}",
        aabbs[0].half_extents
    );

    let widths: Vec<&CellWidth> = world
        .query_filtered::<&CellWidth, With<GuardianCell>>()
        .iter(&world)
        .collect();
    assert_eq!(widths.len(), 1);
    assert!(
        (widths[0].value - 24.0).abs() < f32::EPSILON,
        "guardian CellWidth should be 24.0, got {}",
        widths[0].value
    );

    let heights: Vec<&CellHeight> = world
        .query_filtered::<&CellHeight, With<GuardianCell>>()
        .iter(&world)
        .collect();
    assert_eq!(heights.len(), 1);
    assert!(
        (heights[0].value - 24.0).abs() < f32::EPSILON,
        "guardian CellHeight should be 24.0, got {}",
        heights[0].value
    );
}

// Behavior 36: .guarded() with empty slots spawns parent with GuardedCell but no children
#[test]
fn guarded_empty_slots_no_guardian_children() {
    let mut world = World::new();
    let parent = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .guarded(vec![], test_guardian_config())
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<GuardedCell>(parent).is_some(),
        "parent should have GuardedCell even with empty slots"
    );
    let guardian_count = world.query::<&GuardianCell>().iter(&world).count();
    assert_eq!(
        guardian_count, 0,
        "no guardian children should be spawned with empty slots"
    );
}

// ── Behavior 39: Combined path (.definition + .guarded) spawns exactly N guardians ──

#[test]
fn combined_definition_and_guarded_spawns_exactly_n_guardians() {
    let mut guarded_def = test_cell_definition();
    guarded_def.behaviors = Some(vec![CellBehavior::Guarded(GuardedBehavior {
        guardian_hp_fraction: 0.5,
        guardian_color_rgb: [0.5, 0.8, 1.0],
        slide_speed: 30.0,
    })]);
    let config = GuardianSpawnConfig {
        hp: 10.0,
        color_rgb: [0.5, 0.8, 1.0],
        slide_speed: 30.0,
        cell_height: 24.0,
        step_x: 72.0,
        step_y: 26.0,
    };

    let mut world = World::new();
    let parent = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&guarded_def)
            .position(Vec2::new(100.0, 200.0))
            .dimensions(70.0, 24.0)
            .guarded(vec![0], config)
            .headless()
            .spawn(commands)
    });

    // Parent should have GuardedCell (idempotent insert from behavior match + .guarded())
    assert!(
        world.get::<GuardedCell>(parent).is_some(),
        "parent should have GuardedCell marker"
    );

    // Exactly 1 guardian child, not 2
    let guardian_count = world
        .query_filtered::<Entity, With<GuardianCell>>()
        .iter(&world)
        .count();
    assert_eq!(
        guardian_count, 1,
        "combined .definition() + .guarded() should spawn exactly 1 guardian, not 2; got {guardian_count}"
    );
}

// ── Behavior 50: Guardian entity has CleanupOnExit<NodeState> ──

#[test]
fn guardian_has_cleanup_on_exit_node_state() {
    use rantzsoft_stateflow::CleanupOnExit;

    use crate::state::types::NodeState;

    let config = GuardianSpawnConfig {
        hp: 10.0,
        color_rgb: [0.5, 0.8, 1.0],
        slide_speed: 30.0,
        cell_height: 24.0,
        step_x: 72.0,
        step_y: 26.0,
    };

    let mut world = World::new();
    let _parent = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::new(100.0, 200.0))
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .guarded(vec![0], config)
            .headless()
            .spawn(commands)
    });

    // Find the guardian child entity
    let guardian = world
        .query_filtered::<Entity, With<GuardianCell>>()
        .iter(&world)
        .next()
        .expect("should have spawned a guardian child");

    assert!(
        world.get::<CleanupOnExit<NodeState>>(guardian).is_some(),
        "guardian should have CleanupOnExit<NodeState> (via Cell #[require])"
    );
}
