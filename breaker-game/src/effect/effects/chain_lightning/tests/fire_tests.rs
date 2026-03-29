use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D};

use super::helpers::*;
use crate::{
    bolt::BASE_BOLT_DAMAGE,
    cells::messages::DamageCell,
    effect::core::EffectSourceChip,
    shared::{BOLT_LAYER, CELL_LAYER, CleanupOnNodeExit, GameRng, WALL_LAYER},
};

// ── Behavior 1: fire() damages the first valid target cell immediately via DamageCell ──

#[test]
fn fire_damages_first_target_immediately_via_damage_cell() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(100.0, 200.0, 0.0))
        .id();

    let cell = spawn_test_cell(&mut app, 120.0, 200.0);

    // Tick to populate quadtree
    tick(&mut app);

    fire(entity, 3, 50.0, 1.5, 200.0, "", app.world_mut());

    // DamageCell should be present immediately after fire(), without needing a tick
    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();

    assert_eq!(
        written.len(),
        1,
        "fire() should write exactly 1 DamageCell for the first target, got {}",
        written.len()
    );
    assert_eq!(
        written[0].cell, cell,
        "DamageCell should target the spawned cell"
    );

    let expected_damage = BASE_BOLT_DAMAGE * 1.5;
    assert!(
        (written[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {expected_damage}, got {}",
        written[0].damage
    );
    assert_eq!(
        written[0].source_chip, None,
        "source_chip should be None for empty chip name"
    );
}

#[test]
fn fire_scales_damage_by_effective_damage_multiplier() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(100.0, 200.0, 0.0),
            crate::effect::EffectiveDamageMultiplier(2.0),
        ))
        .id();

    let _cell = spawn_test_cell(&mut app, 120.0, 200.0);

    tick(&mut app);

    fire(entity, 3, 50.0, 1.5, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();

    assert_eq!(written.len(), 1, "expected 1 DamageCell");

    // damage = BASE_BOLT_DAMAGE * 1.5 * 2.0 = 30.0
    let expected_damage = BASE_BOLT_DAMAGE * 1.5 * 2.0;
    assert!(
        (written[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {expected_damage} (10.0 * 1.5 * 2.0), got {}",
        written[0].damage
    );
}

// ── Behavior 2: fire() spawns a ChainLightningChain entity with correct initial state ──

#[test]
fn fire_spawns_chain_entity_with_correct_initial_state() {
    let mut app = chain_lightning_test_app();
    app.world_mut().insert_resource(GameRng::from_seed(0));

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 40.0, 0.0);
    let _cell_c = spawn_test_cell(&mut app, 60.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 25.0, 1.0, 200.0, "", app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningChain>();
    let chains: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(
        chains.len(),
        1,
        "expected exactly one ChainLightningChain entity, got {}",
        chains.len()
    );

    let chain = chains[0];
    assert_eq!(
        chain.remaining_jumps, 2,
        "remaining_jumps should be 2 (arcs=3 minus 1 for initial target)"
    );

    let expected_damage = BASE_BOLT_DAMAGE * 1.0;
    assert!(
        (chain.damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {expected_damage}, got {}",
        chain.damage
    );

    assert_eq!(
        chain.hit_set.len(),
        1,
        "hit_set should contain exactly the first target"
    );

    assert!(
        matches!(chain.state, ChainState::Idle),
        "initial state should be Idle"
    );

    assert!(
        (chain.range - 25.0).abs() < f32::EPSILON,
        "range should be 25.0, got {}",
        chain.range
    );

    assert!(
        (chain.arc_speed - 200.0).abs() < f32::EPSILON,
        "arc_speed should be 200.0, got {}",
        chain.arc_speed
    );

    assert_eq!(
        chain.source,
        Vec2::new(20.0, 0.0),
        "chain source should be the position of the first target"
    );
}

#[test]
fn fire_chain_entity_has_cleanup_on_node_exit() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 40.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 25.0, 1.0, 200.0, "", app.world_mut());

    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<ChainLightningChain>>();
    let chain_entity = query
        .iter(app.world())
        .next()
        .expect("chain entity should exist");

    assert!(
        app.world().get::<CleanupOnNodeExit>(chain_entity).is_some(),
        "ChainLightningChain entity should have CleanupOnNodeExit"
    );
}

#[test]
fn fire_chain_entity_has_effect_source_chip_none_for_empty_chip() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 40.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 25.0, 1.0, 200.0, "", app.world_mut());

    let mut query = app
        .world_mut()
        .query_filtered::<&EffectSourceChip, With<ChainLightningChain>>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one chain entity with EffectSourceChip"
    );
    assert_eq!(
        results[0].0, None,
        "empty source_chip should produce EffectSourceChip(None)"
    );
}

#[test]
fn fire_chain_entity_damage_includes_effective_damage_multiplier() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            crate::effect::EffectiveDamageMultiplier(2.0),
        ))
        .id();

    let _cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 40.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 25.0, 1.0, 200.0, "", app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningChain>();
    let chains: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(chains.len(), 1);

    // damage = BASE_BOLT_DAMAGE * 1.0 * 2.0 = 20.0
    let expected_damage = BASE_BOLT_DAMAGE * 1.0 * 2.0;
    assert!(
        (chains[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected chain damage {expected_damage}, got {}",
        chains[0].damage
    );
}

// ── Behavior 3: fire() with arcs=0 does nothing ──

#[test]
fn fire_with_arcs_zero_does_nothing() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell = spawn_test_cell(&mut app, 10.0, 0.0);

    tick(&mut app);

    fire(entity, 0, 50.0, 1.0, 200.0, "", app.world_mut());

    // No ChainLightningChain entity
    let mut chain_query = app.world_mut().query::<&ChainLightningChain>();
    let chains: Vec<_> = chain_query.iter(app.world()).collect();
    assert!(
        chains.is_empty(),
        "arcs=0 should not spawn any chain entity"
    );

    // No DamageCell message
    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();
    assert!(
        written.is_empty(),
        "arcs=0 should not write any DamageCell message"
    );
}

#[test]
fn fire_with_arcs_zero_and_multiple_cells_does_nothing() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell_a = spawn_test_cell(&mut app, 10.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 15.0, 0.0);
    let _cell_c = spawn_test_cell(&mut app, 20.0, 0.0);

    tick(&mut app);

    fire(entity, 0, 50.0, 1.0, 200.0, "", app.world_mut());

    let mut chain_query = app.world_mut().query::<&ChainLightningChain>();
    assert!(
        chain_query.iter(app.world()).next().is_none(),
        "arcs=0 with multiple cells should still do nothing"
    );
}

// ── Behavior 4: fire() with arcs=1 damages first target, spawns no chain entity ──

#[test]
fn fire_with_arcs_one_damages_first_target_and_spawns_no_chain() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let cell = spawn_test_cell(&mut app, 10.0, 0.0);

    tick(&mut app);

    fire(entity, 1, 50.0, 1.0, 200.0, "", app.world_mut());

    // DamageCell should be written
    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();
    assert_eq!(written.len(), 1, "arcs=1 should damage the first target");
    assert_eq!(written[0].cell, cell);

    // No chain entity (remaining_jumps would be 0)
    let mut chain_query = app.world_mut().query::<&ChainLightningChain>();
    assert!(
        chain_query.iter(app.world()).next().is_none(),
        "arcs=1 should not spawn a chain entity (remaining_jumps=0)"
    );
}

#[test]
fn fire_with_arcs_one_and_no_cells_in_range_does_nothing() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    // Cell far away
    let _cell = spawn_test_cell(&mut app, 500.0, 0.0);

    tick(&mut app);

    fire(entity, 1, 50.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();
    assert!(
        written.is_empty(),
        "arcs=1 with no cells in range should not damage anything"
    );

    let mut chain_query = app.world_mut().query::<&ChainLightningChain>();
    assert!(
        chain_query.iter(app.world()).next().is_none(),
        "arcs=1 with no cells in range should not spawn a chain"
    );
}

// ── Behavior 5: fire() with no valid targets in range ──

#[test]
fn fire_with_no_targets_in_range_damages_nothing_and_spawns_no_chain() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell = spawn_test_cell(&mut app, 500.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 50.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();
    assert!(
        written.is_empty(),
        "no targets in range should produce no DamageCell"
    );

    let mut chain_query = app.world_mut().query::<&ChainLightningChain>();
    assert!(
        chain_query.iter(app.world()).next().is_none(),
        "no targets in range should not spawn a chain"
    );
}

#[test]
fn fire_with_empty_quadtree_damages_nothing() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    // No cells at all
    tick(&mut app);

    fire(entity, 3, 50.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();
    assert!(
        written.is_empty(),
        "empty quadtree should produce no DamageCell"
    );

    let mut chain_query = app.world_mut().query::<&ChainLightningChain>();
    assert!(
        chain_query.iter(app.world()).next().is_none(),
        "empty quadtree should not spawn a chain"
    );
}

// ── Behavior 6: fire() with range=0 or negative range ──

#[test]
fn fire_with_zero_range_damages_nothing() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell = spawn_test_cell(&mut app, 0.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 0.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();
    assert!(written.is_empty(), "range=0.0 should produce no DamageCell");

    let mut chain_query = app.world_mut().query::<&ChainLightningChain>();
    assert!(
        chain_query.iter(app.world()).next().is_none(),
        "range=0.0 should not spawn a chain"
    );
}

#[test]
fn fire_with_negative_range_damages_nothing() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell = spawn_test_cell(&mut app, 0.0, 0.0);

    tick(&mut app);

    fire(entity, 3, -5.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();
    assert!(
        written.is_empty(),
        "negative range should produce no DamageCell"
    );

    let mut chain_query = app.world_mut().query::<&ChainLightningChain>();
    assert!(
        chain_query.iter(app.world()).next().is_none(),
        "negative range should not spawn a chain"
    );
}

// ── Behavior 7: fire() only targets cells on CELL_LAYER ──

#[test]
fn fire_only_targets_cells_on_cell_layer() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    // Cell on CELL_LAYER
    let cell = spawn_test_cell(&mut app, 10.0, 0.0);

    // Wall entity on WALL_LAYER (not a cell)
    let wall_pos = Vec2::new(5.0, 0.0);
    app.world_mut().spawn((
        Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
        CollisionLayers::new(WALL_LAYER, 0),
        Position2D(wall_pos),
        GlobalPosition2D(wall_pos),
        Spatial2D,
    ));

    // Bolt entity on BOLT_LAYER (not a cell)
    let bolt_pos = Vec2::new(8.0, 0.0);
    app.world_mut().spawn((
        Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
        CollisionLayers::new(BOLT_LAYER, 0),
        Position2D(bolt_pos),
        GlobalPosition2D(bolt_pos),
        Spatial2D,
    ));

    tick(&mut app);

    fire(entity, 3, 50.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();

    assert_eq!(
        written.len(),
        1,
        "only CELL_LAYER entity should be targeted"
    );
    assert_eq!(
        written[0].cell, cell,
        "DamageCell should target the CELL_LAYER entity"
    );
}

#[test]
fn fire_targets_entity_with_combined_cell_layer_membership() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    // Entity with CELL_LAYER | BOLT_LAYER
    let pos = Vec2::new(10.0, 0.0);
    let combined = app
        .world_mut()
        .spawn((
            crate::cells::components::Cell,
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
            CollisionLayers::new(CELL_LAYER | BOLT_LAYER, 0),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
        ))
        .id();

    tick(&mut app);

    fire(entity, 3, 50.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();

    assert_eq!(
        written.len(),
        1,
        "combined CELL_LAYER entity should be targeted"
    );
    assert_eq!(written[0].cell, combined);
}

// ── Behavior 8: fire() reads position from Position2D first, then Transform, then Vec2::ZERO ──

#[test]
fn fire_defaults_position_to_zero_with_no_position_or_transform() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn_empty().id();

    // Cell at (10, 0) — within range 50 from origin
    let cell = spawn_test_cell(&mut app, 10.0, 0.0);

    tick(&mut app);

    fire(entity, 1, 50.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();

    assert_eq!(
        written.len(),
        1,
        "cell at (10,0) should be in range from default (0,0)"
    );
    assert_eq!(written[0].cell, cell);
}

#[test]
fn fire_prefers_position2d_over_transform() {
    let mut app = chain_lightning_test_app();

    // Entity with Position2D(50, 50) and Transform at (100, 100)
    let entity = app
        .world_mut()
        .spawn((
            rantzsoft_spatial2d::components::Position2D(Vec2::new(50.0, 50.0)),
            Transform::from_xyz(100.0, 100.0, 0.0),
        ))
        .id();

    // Cell at (60, 50) — 10 units from Position2D but 50+ from Transform
    let cell = spawn_test_cell(&mut app, 60.0, 50.0);

    tick(&mut app);

    fire(entity, 1, 15.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();

    assert_eq!(
        written.len(),
        1,
        "should use Position2D (50,50), cell at (60,50) is 10 units away, within range 15"
    );
    assert_eq!(written[0].cell, cell);
}

// ── Behavior 9: fire() stores EffectSourceChip on chain entity ──

#[test]
fn fire_stores_effect_source_chip_with_non_empty_chip_name() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell_a = spawn_test_cell(&mut app, 10.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 20.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 50.0, 1.0, 200.0, "zapper", app.world_mut());

    let mut query = app
        .world_mut()
        .query_filtered::<&EffectSourceChip, With<ChainLightningChain>>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "expected one chain entity");
    assert_eq!(
        results[0].0,
        Some("zapper".to_string()),
        "spawned chain should have EffectSourceChip(Some(\"zapper\"))"
    );
}

#[test]
fn fire_stores_effect_source_chip_none_for_empty_chip_name() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell_a = spawn_test_cell(&mut app, 10.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 20.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 50.0, 1.0, 200.0, "", app.world_mut());

    let mut query = app
        .world_mut()
        .query_filtered::<&EffectSourceChip, With<ChainLightningChain>>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "expected one chain entity");
    assert_eq!(
        results[0].0, None,
        "empty source_chip should produce EffectSourceChip(None)"
    );
}

// ── Behavior 10: fire() uses GameRng deterministically ──

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
    fire(entity, 1, 50.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let first_written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();
    assert_eq!(first_written.len(), 1, "first fire should damage one cell");
    let first_target = first_written[0].cell;

    // Reset by creating a new app with same setup for clean message state
    let mut app2 = chain_lightning_test_app();

    let entity2 = app2
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell_a2 = spawn_test_cell(&mut app2, 10.0, 0.0);
    let _cell_b2 = spawn_test_cell(&mut app2, 0.0, 10.0);
    let _cell_c2 = spawn_test_cell(&mut app2, -10.0, 0.0);

    tick(&mut app2);

    // Same seed
    app2.world_mut().insert_resource(GameRng::from_seed(42));
    fire(entity2, 1, 50.0, 1.0, 200.0, "", app2.world_mut());

    let messages2 = app2.world().resource::<Messages<DamageCell>>();
    let second_written: Vec<&DamageCell> = messages2.iter_current_update_messages().collect();
    assert_eq!(
        second_written.len(),
        1,
        "second fire should damage one cell"
    );
    let second_target = second_written[0].cell;

    // Entity IDs may differ across apps, but with same entity spawn order they should be the same
    assert_eq!(
        first_target, second_target,
        "same RNG seed should produce same target selection"
    );
}

// ── Behavior 10b: fire() with damage_mult=0.0 ──

#[test]
fn fire_with_zero_damage_mult_sends_damage_cell_with_zero_damage() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let cell = spawn_test_cell(&mut app, 10.0, 0.0);

    tick(&mut app);

    fire(entity, 1, 50.0, 0.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();
    assert_eq!(
        written.len(),
        1,
        "should still write DamageCell with 0 damage"
    );
    assert_eq!(written[0].cell, cell);
    assert!(
        (written[0].damage - 0.0).abs() < f32::EPSILON,
        "damage should be 0.0, got {}",
        written[0].damage
    );
}

#[test]
fn fire_with_zero_damage_mult_and_multiple_arcs_spawns_chain_with_zero_damage() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell_a = spawn_test_cell(&mut app, 10.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 20.0, 0.0);
    let _cell_c = spawn_test_cell(&mut app, 30.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 50.0, 0.0, 200.0, "", app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningChain>();
    let chains: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(chains.len(), 1, "should spawn chain entity with damage=0.0");
    assert!(
        (chains[0].damage - 0.0).abs() < f32::EPSILON,
        "chain damage should be 0.0"
    );
}

// ── Behavior 21: DamageCell from fire() includes source_chip ──

#[test]
fn fire_damage_cell_includes_source_chip() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();

    let _cell = spawn_test_cell(&mut app, 10.0, 0.0);

    tick(&mut app);

    fire(entity, 1, 50.0, 1.5, 200.0, "zapper", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();
    assert_eq!(written.len(), 1);

    let expected_damage = BASE_BOLT_DAMAGE * 1.5;
    assert!(
        (written[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {expected_damage}"
    );
    assert_eq!(
        written[0].source_chip,
        Some("zapper".to_string()),
        "DamageCell should include source_chip"
    );
}
