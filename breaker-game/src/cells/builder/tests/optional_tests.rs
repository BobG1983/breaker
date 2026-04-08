//! Section F: Optional Chainable Methods (any typestate)
//! Section G: .locked(entities) Optional Method

use bevy::{ecs::world::CommandQueue, prelude::*};

use crate::{
    cells::{
        components::{
            Cell, CellDamageVisuals, CellHealth, CellRegen, CellTypeAlias, LockAdjacents, Locked,
            RequiredToClear,
        },
        definition::{CellBehavior, CellTypeDefinition},
    },
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, Target},
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

// ── Section F: Optional Chainable Methods ───────────────────────────────────

// Behavior 24: .with_behavior(CellBehavior::Regen { rate: 3.0 }) inserts CellRegen
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
        .get::<CellRegen>(entity)
        .expect("entity should have CellRegen");
    assert!(
        (regen.rate - 3.0).abs() < f32::EPSILON,
        "CellRegen rate should be 3.0, got {}",
        regen.rate
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
        .get::<CellRegen>(entity)
        .expect("entity should have CellRegen");
    assert!(
        (regen.rate - 1.0).abs() < f32::EPSILON,
        "CellRegen rate should be 1.0 (last write wins), got {}",
        regen.rate
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
        .get::<CellRegen>(entity)
        .expect("entity should have CellRegen");
    assert!(
        (regen.rate - 5.0).abs() < f32::EPSILON,
        "CellRegen rate should be 5.0 (explicit appended after definition, last write wins), got {}",
        regen.rate
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
        .get::<CellRegen>(entity)
        .expect("entity should have CellRegen");
    assert!(
        (regen.rate - 3.0).abs() < f32::EPSILON,
        "CellRegen rate should be 3.0 (explicit only), got {}",
        regen.rate
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
        .get::<LockAdjacents>(entity)
        .expect("entity should have LockAdjacents");
    assert_eq!(adjacents.0.len(), 2, "LockAdjacents should have 2 entities");
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
        .get::<LockAdjacents>(entity)
        .expect("entity should have LockAdjacents");
    assert!(adjacents.0.is_empty(), "LockAdjacents should be empty");
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
        world.get::<LockAdjacents>(entity).is_none(),
        "entity should NOT have LockAdjacents without .locked()"
    );
}
