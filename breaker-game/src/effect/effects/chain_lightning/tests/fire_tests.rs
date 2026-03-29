use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D};

use super::helpers::*;
use crate::{
    bolt::BASE_BOLT_DAMAGE,
    cells::components::Cell,
    shared::{BOLT_LAYER, CELL_LAYER, CleanupOnNodeExit, GameRng, WALL_LAYER},
};

// ── Behavior 1: fire() spawns a ChainLightningRequest with targets from quadtree ──

#[test]
fn fire_spawns_request_with_single_cell_target() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(100.0, 200.0, 0.0))
        .id();

    let cell = spawn_test_cell(&mut app, 120.0, 200.0);

    // Tick to populate quadtree
    tick(&mut app);

    fire(entity, 3, 50.0, 1.5, app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningRequest>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(
        results.len(),
        1,
        "expected exactly one ChainLightningRequest entity"
    );

    let request = results[0];
    assert_eq!(
        request.targets.len(),
        1,
        "expected exactly one target (one cell within range)"
    );
    assert_eq!(
        request.targets[0].0, cell,
        "target should be the cell entity"
    );

    let expected_damage = BASE_BOLT_DAMAGE * 1.5;
    assert!(
        (request.targets[0].1 - expected_damage).abs() < f32::EPSILON,
        "expected damage {}, got {}",
        expected_damage,
        request.targets[0].1
    );
    assert!(
        (request.source.x - 100.0).abs() < f32::EPSILON,
        "expected source x 100.0, got {}",
        request.source.x
    );
    assert!(
        (request.source.y - 200.0).abs() < f32::EPSILON,
        "expected source y 200.0, got {}",
        request.source.y
    );
}

#[test]
fn fire_with_no_transform_defaults_position_to_zero() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn_empty().id();

    let _cell = spawn_test_cell(&mut app, 10.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 50.0, 1.0, app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningRequest>();
    let results: Vec<_> = query.iter(app.world()).collect();

    // Request should exist (cell is within range of origin)
    assert!(
        !results.is_empty(),
        "request should be spawned even without Transform"
    );

    let request = results[0];
    assert!(
        (request.source.x).abs() < f32::EPSILON,
        "source should default to 0.0 x"
    );
    assert!(
        (request.source.y).abs() < f32::EPSILON,
        "source should default to 0.0 y"
    );
}

// ── Behavior 2: fire() chains through multiple cells up to arcs count ──

#[test]
fn fire_chains_through_multiple_cells_up_to_arcs_count() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzPhysics2dPlugin);
    app.insert_resource(GameRng::from_seed(0));

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 40.0, 0.0);
    let _cell_c = spawn_test_cell(&mut app, 60.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 25.0, 1.0, app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningRequest>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "expected one request entity");

    let request = results[0];
    // With range=25, chain should jump from origin -> cell_a (20 away),
    // then cell_a -> cell_b (20 away), then cell_b -> cell_c (20 away).
    // All 3 cells reachable with arcs=3.
    assert_eq!(
        request.targets.len(),
        3,
        "expected 3 targets (all reachable with range 25.0 and arcs 3)"
    );

    for (_entity, damage) in &request.targets {
        let expected = BASE_BOLT_DAMAGE * 1.0;
        assert!(
            (damage - expected).abs() < f32::EPSILON,
            "each target should have damage {expected}, got {damage}"
        );
    }
}

// ── Behavior 3: fire() does not include the same cell twice ──

#[test]
fn fire_does_not_include_same_cell_twice() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzPhysics2dPlugin);
    app.insert_resource(GameRng::from_seed(0));

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell_a = spawn_test_cell(&mut app, 10.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 15.0, 0.0);

    tick(&mut app);

    fire(entity, 5, 20.0, 1.0, app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningRequest>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "expected one request entity");

    let request = results[0];
    // With only 2 cells available, even arcs=5, max targets is 2
    assert!(
        request.targets.len() <= 2,
        "targets should contain at most 2 entries (one per unique cell), got {}",
        request.targets.len()
    );

    // Check uniqueness
    let unique_entities: HashSet<Entity> = request.targets.iter().map(|(e, _)| *e).collect();
    assert_eq!(
        unique_entities.len(),
        request.targets.len(),
        "each cell entity should appear at most once in targets"
    );
}

#[test]
fn fire_single_cell_in_range_with_multiple_arcs_produces_one_target() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzPhysics2dPlugin);
    app.insert_resource(GameRng::from_seed(0));

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let cell = spawn_test_cell(&mut app, 10.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 20.0, 1.0, app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningRequest>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "expected one request entity");

    let request = results[0];
    assert_eq!(
        request.targets.len(),
        1,
        "single cell in range, arcs=3 should produce exactly 1 target"
    );
    assert_eq!(request.targets[0].0, cell);
}

// ── Behavior 4: fire() only targets cells on CELL_LAYER ──

#[test]
fn fire_only_targets_cells_on_cell_layer() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    // Cell on CELL_LAYER
    let cell = spawn_test_cell(&mut app, 10.0, 0.0);

    // Entity on WALL_LAYER (not a cell)
    let wall_pos = Vec2::new(5.0, 0.0);
    app.world_mut().spawn((
        Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
        CollisionLayers::new(WALL_LAYER, 0),
        Position2D(wall_pos),
        GlobalPosition2D(wall_pos),
        Spatial2D,
    ));

    // Entity on BOLT_LAYER (not a cell)
    let bolt_pos = Vec2::new(8.0, 0.0);
    app.world_mut().spawn((
        Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
        CollisionLayers::new(BOLT_LAYER, 0),
        Position2D(bolt_pos),
        GlobalPosition2D(bolt_pos),
        Spatial2D,
    ));

    tick(&mut app);

    fire(entity, 3, 50.0, 1.0, app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningRequest>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "expected one request entity");

    let request = results[0];
    // Only CELL_LAYER entity should be in targets
    let target_entities: Vec<Entity> = request.targets.iter().map(|(e, _)| *e).collect();
    assert!(
        target_entities.contains(&cell),
        "CELL_LAYER entity should be in targets"
    );
    assert_eq!(
        target_entities.len(),
        1,
        "only CELL_LAYER entities should be targeted, got {}",
        target_entities.len()
    );
}

#[test]
fn fire_targets_entity_with_combined_cell_layer_membership() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    // Entity with CELL_LAYER | BOLT_LAYER — should be found since it includes CELL_LAYER
    let pos = Vec2::new(10.0, 0.0);
    let combined = app
        .world_mut()
        .spawn((
            Cell,
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
            CollisionLayers::new(CELL_LAYER | BOLT_LAYER, 0),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
        ))
        .id();

    tick(&mut app);

    fire(entity, 3, 50.0, 1.0, app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningRequest>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "expected one request entity");

    let request = results[0];
    let target_entities: Vec<Entity> = request.targets.iter().map(|(e, _)| *e).collect();
    assert!(
        target_entities.contains(&combined),
        "entity with CELL_LAYER in combined mask should be targeted"
    );
}

// ── Behavior 5: fire() terminates chain when no targets in range ──

#[test]
fn fire_terminates_chain_when_next_cell_out_of_range() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let cell_a = spawn_test_cell(&mut app, 10.0, 0.0);
    // cell_b is far away from cell_a — beyond range=25
    let _cell_b = spawn_test_cell(&mut app, 200.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 25.0, 1.0, app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningRequest>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "expected one request entity");

    let request = results[0];
    assert_eq!(
        request.targets.len(),
        1,
        "chain should stop after cell_a because cell_b is too far"
    );
    assert_eq!(request.targets[0].0, cell_a);
}

#[test]
fn fire_with_no_cells_in_range_spawns_request_with_empty_targets() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    // Cell far away — outside range
    let _cell = spawn_test_cell(&mut app, 500.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 50.0, 1.0, app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningRequest>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "expected one request entity");

    let request = results[0];
    assert!(
        request.targets.is_empty(),
        "no cells in range — targets should be empty"
    );
}

// ── Behavior 6: fire() uses GameRng deterministically ──

#[test]
fn fire_uses_game_rng_deterministically() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell_a = spawn_test_cell(&mut app, 10.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 0.0, 10.0);
    let _cell_c = spawn_test_cell(&mut app, -10.0, 0.0);

    tick(&mut app);

    // First call with seed 42
    app.world_mut().insert_resource(GameRng::from_seed(42));
    fire(entity, 1, 50.0, 1.0, app.world_mut());

    let mut query = app.world_mut().query::<(Entity, &ChainLightningRequest)>();
    let first_results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(first_results.len(), 1);
    let first_target = first_results[0].1.targets[0].0;
    let first_request_entity = first_results[0].0;

    // Remove first request
    app.world_mut().despawn(first_request_entity);

    // Reset RNG to same seed and fire again
    app.world_mut().insert_resource(GameRng::from_seed(42));
    fire(entity, 1, 50.0, 1.0, app.world_mut());

    let mut query2 = app.world_mut().query::<&ChainLightningRequest>();
    let second_results: Vec<_> = query2.iter(app.world()).collect();
    assert_eq!(second_results.len(), 1);
    let second_target = second_results[0].targets[0].0;

    assert_eq!(
        first_target, second_target,
        "same RNG seed should produce same target selection"
    );
}

#[test]
fn fire_single_candidate_always_selected_regardless_of_rng_seed() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzPhysics2dPlugin);
    app.insert_resource(GameRng::from_seed(999));

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let cell = spawn_test_cell(&mut app, 10.0, 0.0);

    tick(&mut app);

    fire(entity, 1, 50.0, 1.0, app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningRequest>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].targets[0].0, cell,
        "only one candidate — must be selected regardless of RNG state"
    );
}

// ── Behavior 7: fire() applies damage_mult to BASE_BOLT_DAMAGE ──

#[test]
fn fire_applies_damage_mult_to_base_bolt_damage() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell = spawn_test_cell(&mut app, 10.0, 0.0);

    tick(&mut app);

    fire(entity, 1, 50.0, 2.5, app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningRequest>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1);

    let expected_damage = BASE_BOLT_DAMAGE * 2.5;
    assert!(
        (results[0].targets[0].1 - expected_damage).abs() < f32::EPSILON,
        "expected damage {}, got {}",
        expected_damage,
        results[0].targets[0].1
    );
}

#[test]
fn fire_with_zero_damage_mult_produces_zero_damage() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell = spawn_test_cell(&mut app, 10.0, 0.0);

    tick(&mut app);

    fire(entity, 1, 50.0, 0.0, app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningRequest>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1);

    assert!(
        (results[0].targets[0].1 - 0.0).abs() < f32::EPSILON,
        "damage_mult 0.0 should produce damage 0.0, got {}",
        results[0].targets[0].1
    );
}

// ── Behavior 8: fire() with arcs=0 spawns no request entity ──

#[test]
fn fire_with_arcs_zero_spawns_no_request() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell = spawn_test_cell(&mut app, 10.0, 0.0);

    tick(&mut app);

    fire(entity, 0, 50.0, 1.0, app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningRequest>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert!(
        results.is_empty(),
        "arcs=0 should not spawn any request entity"
    );
}

// ── Behavior 9: fire() with range=0 spawns request with empty targets ──

#[test]
fn fire_with_zero_range_spawns_request_with_empty_targets() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    // Cell at same position
    let _cell = spawn_test_cell(&mut app, 0.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 0.0, 1.0, app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningRequest>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "request should be spawned");

    assert!(
        results[0].targets.is_empty(),
        "range=0.0 should produce empty targets (circle query with radius 0 returns nothing)"
    );
    assert!(
        (results[0].source.x).abs() < f32::EPSILON,
        "source should be (0, 0)"
    );
}

// ── Behavior 10: fire() with no cells in range spawns request with empty targets ──

#[test]
fn fire_with_empty_quadtree_spawns_request_with_empty_targets() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    // No cells spawned at all
    tick(&mut app);

    fire(entity, 3, 50.0, 1.0, app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningRequest>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "request should be spawned");

    assert!(
        results[0].targets.is_empty(),
        "no cells in quadtree — targets should be empty"
    );
}

// ── Behavior 1 edge case: request entity has CleanupOnNodeExit ──

#[test]
fn fire_request_entity_has_cleanup_on_node_exit() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell = spawn_test_cell(&mut app, 10.0, 0.0);

    tick(&mut app);

    fire(entity, 1, 50.0, 1.0, app.world_mut());

    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<ChainLightningRequest>>();
    let request_entity = query
        .iter(app.world())
        .next()
        .expect("request should exist");

    assert!(
        app.world()
            .get::<CleanupOnNodeExit>(request_entity)
            .is_some(),
        "ChainLightningRequest entity should have CleanupOnNodeExit"
    );
}

// ── Damage scaling: fire() includes EffectiveDamageMultiplier in pre-computed damage ──

#[test]
fn fire_scales_damage_by_effective_damage_multiplier() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            crate::effect::EffectiveDamageMultiplier(2.0),
        ))
        .id();

    let _cell_a = spawn_test_cell(&mut app, 30.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 60.0, 0.0);

    tick(&mut app);

    fire(entity, 2, 100.0, 1.5, app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningRequest>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "expected one request entity");

    let request = results[0];
    assert_eq!(
        request.targets.len(),
        2,
        "expected 2 targets, got {}",
        request.targets.len()
    );

    // damage = BASE_BOLT_DAMAGE * damage_mult * EDM = 10.0 * 1.5 * 2.0 = 30.0
    let expected_damage = BASE_BOLT_DAMAGE * 1.5 * 2.0;
    for (_entity, damage) in &request.targets {
        assert!(
            (damage - expected_damage).abs() < f32::EPSILON,
            "expected damage {expected_damage} (10.0 * 1.5 * 2.0), got {damage}",
        );
    }
}
