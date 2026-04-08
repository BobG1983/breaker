//! Section F: Optional Chainable Methods (any typestate)
//! Section G: .locked(entities) Optional Method

use bevy::{ecs::world::CommandQueue, prelude::*};
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::{
    components::{Position2D, Scale2D},
    propagation::PositionPropagation,
};

use crate::{
    cells::{
        builder::core::types::GuardianSpawnConfig,
        components::{
            Cell, CellDamageVisuals, CellHealth, CellHeight, CellTypeAlias, CellWidth, GuardedCell,
            GuardianCell, GuardianGridStep, GuardianSlideSpeed, GuardianSlot, Locked, Locks,
            RegenRate, RequiredToClear, SlideTarget,
        },
        definition::{CellBehavior, CellTypeDefinition, GuardedBehavior},
    },
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, Target},
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

// ── Section F: Optional Chainable Methods ───────────────────────────────────

// Behavior 24: .with_behavior(CellBehavior::Regen { rate: 3.0 }) inserts RegenRate
#[test]
fn with_behavior_regen_inserts_cell_regen() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_behavior(CellBehavior::Regen { rate: 3.0 })
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let regen = world
        .get::<RegenRate>(entity)
        .expect("entity should have RegenRate");
    assert!(
        (regen.0 - 3.0).abs() < f32::EPSILON,
        "RegenRate rate should be 3.0, got {}",
        regen.0
    );
}

// Behavior 24 edge case: calling .with_behavior() twice — last write wins
#[test]
fn with_behavior_twice_last_write_wins() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_behavior(CellBehavior::Regen { rate: 3.0 })
            .with_behavior(CellBehavior::Regen { rate: 1.0 })
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let regen = world
        .get::<RegenRate>(entity)
        .expect("entity should have RegenRate");
    assert!(
        (regen.0 - 1.0).abs() < f32::EPSILON,
        "RegenRate rate should be 1.0 (last write wins), got {}",
        regen.0
    );
}

// Behavior 25: .with_behavior() combines with definition behaviors
#[test]
fn with_behavior_after_definition_explicit_wins() {
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
        .get::<RegenRate>(entity)
        .expect("entity should have RegenRate");
    assert!(
        (regen.0 - 5.0).abs() < f32::EPSILON,
        "RegenRate rate should be 5.0 (explicit appended after definition, last write wins), got {}",
        regen.0
    );
}

// Behavior 25 edge case: definition behaviors None plus explicit
#[test]
fn with_behavior_without_definition_behaviors() {
    let mut def = test_cell_definition();
    def.behaviors = None;

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .with_behavior(CellBehavior::Regen { rate: 3.0 })
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let regen = world
        .get::<RegenRate>(entity)
        .expect("entity should have RegenRate");
    assert!(
        (regen.0 - 3.0).abs() < f32::EPSILON,
        "RegenRate rate should be 3.0 (explicit only), got {}",
        regen.0
    );
}

// Behavior 26: .with_effects(vec![root_effect]) sets effects
#[test]
fn with_effects_stores_effects_for_dispatch() {
    let root_effect = RootEffect::On {
        target: Target::Bolt,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(5.0))],
    };

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_effects(vec![root_effect])
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("entity should have BoundEffects after effects dispatch");
    assert!(!bound.0.is_empty(), "BoundEffects should not be empty");
}

// Behavior 26 edge case: .with_effects(vec![]) stores empty effects
#[test]
fn with_effects_empty_vec_no_bound_effects() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_effects(vec![])
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    // Empty effects should be filtered — no BoundEffects inserted
    assert!(
        world.get::<BoundEffects>(entity).is_none(),
        "entity should NOT have BoundEffects when effects are empty (filtered)"
    );
}

// Behavior 27: .color_rgb() overrides definition color — tested in spawn_tests.rs (rendered)

// Behavior 28: .damage_visuals() overrides definition damage visuals
#[test]
fn damage_visuals_override_definition() {
    let mut def = test_cell_definition();
    def.damage_hdr_base = 4.0;
    def.damage_green_min = 0.4;
    def.damage_blue_range = 0.3;
    def.damage_blue_base = 0.1;

    let override_visuals = CellDamageVisuals {
        hdr_base: 8.0,
        green_min: 0.1,
        blue_range: 0.6,
        blue_base: 0.3,
    };

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .damage_visuals(override_visuals.clone())
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let visuals = world
        .get::<CellDamageVisuals>(entity)
        .expect("entity should have CellDamageVisuals");
    assert!(
        (visuals.hdr_base - 8.0).abs() < f32::EPSILON,
        "hdr_base should be 8.0 (override), got {}",
        visuals.hdr_base
    );
    assert!(
        (visuals.green_min - 0.1).abs() < f32::EPSILON,
        "green_min should be 0.1 (override)"
    );
    assert!(
        (visuals.blue_range - 0.6).abs() < f32::EPSILON,
        "blue_range should be 0.6 (override)"
    );
    assert!(
        (visuals.blue_base - 0.3).abs() < f32::EPSILON,
        "blue_base should be 0.3 (override)"
    );
}

// Behavior 28 edge case: .damage_visuals() without definition
#[test]
fn damage_visuals_without_definition_sets_directly() {
    let visuals = CellDamageVisuals {
        hdr_base: 8.0,
        green_min: 0.1,
        blue_range: 0.6,
        blue_base: 0.3,
    };

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .damage_visuals(visuals.clone())
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let found = world
        .get::<CellDamageVisuals>(entity)
        .expect("entity should have CellDamageVisuals");
    assert!(
        (found.hdr_base - 8.0).abs() < f32::EPSILON,
        "hdr_base should be 8.0"
    );
}

// Behavior 29: .alias() overrides definition alias
#[test]
fn alias_override_definition() {
    let mut def = test_cell_definition();
    def.alias = "R".to_owned();

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .alias("X".to_owned())
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let alias = world
        .get::<CellTypeAlias>(entity)
        .expect("entity should have CellTypeAlias");
    assert_eq!(alias.0, "X", "CellTypeAlias should be 'X' (override)");
}

// Behavior 29 edge case: .alias() without definition
#[test]
fn alias_without_definition_sets_directly() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .alias("test".to_owned())
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let alias = world
        .get::<CellTypeAlias>(entity)
        .expect("entity should have CellTypeAlias");
    assert_eq!(alias.0, "test", "CellTypeAlias should be 'test'");
}

// Behavior 30: .required_to_clear(true) overrides definition
#[test]
fn required_to_clear_true_overrides_false_definition() {
    let mut def = test_cell_definition();
    def.required_to_clear = false;

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .required_to_clear(true)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<RequiredToClear>(entity).is_some(),
        "should have RequiredToClear (override true)"
    );
}

// Behavior 30 edge case: .required_to_clear(false) overrides definition true
#[test]
fn required_to_clear_false_overrides_true_definition() {
    let mut def = test_cell_definition();
    def.required_to_clear = true;

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .required_to_clear(false)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<RequiredToClear>(entity).is_none(),
        "should NOT have RequiredToClear (override false)"
    );
}

// ── Section G: .locked(entities) ────────────────────────────────────────────

// Behavior 31: .locked(entities) stores lock data for spawn
#[test]
fn locked_inserts_locked_and_lock_adjacents() {
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
        .get::<Locks>(entity)
        .expect("entity should have Locks");
    assert_eq!(adjacents.0.len(), 2, "Locks should have 2 entities");
    assert_eq!(adjacents.0[0], e1);
    assert_eq!(adjacents.0[1], e2);
}

// Behavior 31 edge case: locked with empty vec
#[test]
fn locked_empty_vec_still_inserts_markers() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .locked(vec![])
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<Locked>(entity).is_some(),
        "entity should have Locked marker even with empty vec"
    );
    let adjacents = world
        .get::<Locks>(entity)
        .expect("entity should have Locks");
    assert!(adjacents.0.is_empty(), "Locks should be empty");
}

// Behavior 32: .locked() is available in any typestate (compile test)
#[test]
fn locked_available_in_any_typestate() {
    let _builder = Cell::builder().locked(vec![]);
    // Compiles — that is the assertion.
}

// Behavior 33: Cell without .locked() has no lock components
#[test]
fn no_locked_has_no_lock_components() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    // Guard: non-#[require] component ensures builder actually ran
    assert!(
        world.get::<CellHealth>(entity).is_some(),
        "entity should have CellHealth from builder"
    );

    assert!(
        world.get::<Locked>(entity).is_none(),
        "entity should NOT have Locked without .locked()"
    );
    assert!(
        world.get::<Locks>(entity).is_none(),
        "entity should NOT have Locks without .locked()"
    );
}

// ── Section I: .guarded() Builder Integration ──────────────────────────────

fn test_guardian_config() -> GuardianSpawnConfig {
    GuardianSpawnConfig {
        hp: 10.0,
        color_rgb: [0.5, 0.8, 1.0],
        slide_speed: 30.0,
        cell_height: 24.0,
        step_x: 72.0,
        step_y: 26.0,
    }
}

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

// ── Section J: CellBehavior::Guarded Insertion in spawn_inner ──────────────

// Behavior 38: CellBehavior::Guarded inserts GuardedCell marker
#[test]
fn behavior_guarded_inserts_guarded_cell_marker() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_behavior(CellBehavior::Guarded(GuardedBehavior {
                guardian_hp: 10.0,
                guardian_color_rgb: [0.5, 0.8, 1.0],
                slide_speed: 30.0,
            }))
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<GuardedCell>(entity).is_some(),
        "entity with CellBehavior::Guarded should have GuardedCell component"
    );
}

// Behavior 38 edge case: behavior does NOT spawn guardian children
#[test]
fn behavior_guarded_does_not_spawn_children() {
    let mut world = World::new();
    let _entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_behavior(CellBehavior::Guarded(GuardedBehavior {
                guardian_hp: 10.0,
                guardian_color_rgb: [0.5, 0.8, 1.0],
                slide_speed: 30.0,
            }))
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let guardian_count = world.query::<&GuardianCell>().iter(&world).count();
    assert_eq!(
        guardian_count, 0,
        "behavior alone should NOT spawn guardian children"
    );
}

// Behavior 40: Cell without Guarded behavior has no GuardedCell component
#[test]
fn cell_without_guarded_behavior_has_no_guarded_cell() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    // Guard: builder ran
    assert!(world.get::<CellHealth>(entity).is_some());
    assert!(
        world.get::<GuardedCell>(entity).is_none(),
        "cell without Guarded behavior should NOT have GuardedCell"
    );
}

// Behavior 40 edge case: Regen only has no GuardedCell
#[test]
fn cell_with_regen_only_has_no_guarded_cell() {
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

    assert!(
        world.get::<GuardedCell>(entity).is_none(),
        "cell with only Regen behavior should NOT have GuardedCell"
    );
}

// ── Behavior 39: Combined path (.definition + .guarded) spawns exactly N guardians ──

#[test]
fn combined_definition_and_guarded_spawns_exactly_n_guardians() {
    let mut guarded_def = test_cell_definition();
    guarded_def.behaviors = Some(vec![CellBehavior::Guarded(GuardedBehavior {
        guardian_hp: 10.0,
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
