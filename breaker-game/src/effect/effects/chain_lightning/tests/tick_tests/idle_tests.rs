use std::collections::HashSet;

use bevy::prelude::*;

use super::*;
use crate::shared::CleanupOnNodeExit;

// ── Section B: tick_chain_lightning -- Idle State ──────────────────

// ── Behavior 11: tick picks a random valid target and spawns a ChainLightningArc ──

#[test]
fn tick_idle_chain_spawns_arc_and_transitions_to_arc_traveling() {
    let mut app = chain_lightning_damage_test_app();

    let cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let cell_b = spawn_test_cell(&mut app, 40.0, 0.0);

    // Populate quadtree
    tick(&mut app);

    let mut hit_set = HashSet::new();
    hit_set.insert(cell_a);

    let chain_entity = spawn_chain(
        &mut app,
        SpawnChainConfig {
            source: Vec2::new(20.0, 0.0), // source at cell_a position
            remaining_jumps: 2,
            damage: 15.0,
            range: 25.0,
            arc_speed: 200.0,
            hit_set,
            state: ChainState::Idle,
            source_chip: None,
        },
    );

    // Tick to run tick_chain_lightning
    tick(&mut app);

    // Arc entity should exist
    let mut arc_query = app.world_mut().query::<(Entity, &ChainLightningArc)>();
    let arcs: Vec<_> = arc_query.iter(app.world()).collect();
    assert_eq!(
        arcs.len(),
        1,
        "expected one ChainLightningArc entity to be spawned"
    );
    let (arc_entity, _) = arcs[0];

    // Arc should have Transform at source position (20, 0)
    let arc_transform = app.world().get::<Transform>(arc_entity).unwrap();
    assert!(
        (arc_transform.translation.x - 20.0).abs() < 0.01,
        "arc should spawn at source x=20.0, got {}",
        arc_transform.translation.x
    );
    assert!(
        (arc_transform.translation.y - 0.0).abs() < 0.01,
        "arc should spawn at source y=0.0, got {}",
        arc_transform.translation.y
    );

    // Arc should have CleanupOnNodeExit
    assert!(
        app.world().get::<CleanupOnNodeExit>(arc_entity).is_some(),
        "arc entity should have CleanupOnNodeExit"
    );

    // Chain should be in ArcTraveling state
    let chain = app
        .world()
        .get::<ChainLightningChain>(chain_entity)
        .unwrap();
    match &chain.state {
        ChainState::ArcTraveling {
            target,
            target_pos,
            arc_entity: state_arc,
            arc_pos,
        } => {
            assert_eq!(
                *target, cell_b,
                "target should be cell_b (the only valid non-hit cell)"
            );
            assert!(
                (target_pos.x - 40.0).abs() < 0.01,
                "target_pos should be cell_b position"
            );
            assert_eq!(
                *state_arc, arc_entity,
                "arc_entity in state should match spawned arc"
            );
            assert!(
                (arc_pos.x - 20.0).abs() < 0.01,
                "arc_pos should start at source position"
            );
        }
        ChainState::Idle => {
            panic!("chain should transition from Idle to ArcTraveling");
        }
    }

    // remaining_jumps should still be 2 (only decremented on arrival)
    assert_eq!(
        chain.remaining_jumps, 2,
        "remaining_jumps should not decrement until arc arrives"
    );
}

// ── Behavior 12: tick in Idle with no valid targets despawns chain ──

#[test]
fn tick_idle_no_valid_targets_despawns_chain() {
    let mut app = chain_lightning_damage_test_app();

    let cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let cell_b = spawn_test_cell(&mut app, 25.0, 0.0);

    tick(&mut app);

    let mut hit_set = HashSet::new();
    hit_set.insert(cell_a);
    hit_set.insert(cell_b);

    let chain_entity = spawn_chain(
        &mut app,
        SpawnChainConfig {
            source: Vec2::new(20.0, 0.0),
            remaining_jumps: 2,
            damage: 15.0,
            range: 25.0,
            arc_speed: 200.0,
            hit_set,
            state: ChainState::Idle,
            source_chip: None,
        },
    );

    tick(&mut app);

    // Chain should be despawned
    assert!(
        app.world().get_entity(chain_entity).is_err(),
        "chain with no valid targets should be despawned"
    );

    // No arc entity should exist
    let mut arc_query = app.world_mut().query::<&ChainLightningArc>();
    assert!(
        arc_query.iter(app.world()).next().is_none(),
        "no arc should be spawned when no valid targets"
    );
}

// ── Behavior 13: tick in Idle with 0 remaining jumps despawns chain ──

#[test]
fn tick_idle_zero_remaining_jumps_despawns_chain() {
    let mut app = chain_lightning_damage_test_app();

    let _cell = spawn_test_cell(&mut app, 10.0, 0.0);

    tick(&mut app);

    let chain_entity = spawn_chain(
        &mut app,
        SpawnChainConfig {
            source: Vec2::new(0.0, 0.0),
            remaining_jumps: 0, // remaining_jumps = 0
            damage: 15.0,
            range: 25.0,
            arc_speed: 200.0,
            hit_set: HashSet::new(),
            state: ChainState::Idle,
            source_chip: None,
        },
    );

    tick(&mut app);

    assert!(
        app.world().get_entity(chain_entity).is_err(),
        "chain with 0 remaining jumps should be despawned"
    );

    // No arc should be spawned
    let mut arc_query = app.world_mut().query::<&ChainLightningArc>();
    assert!(
        arc_query.iter(app.world()).next().is_none(),
        "no arc should be spawned with 0 remaining jumps"
    );

    // No damage should be dealt
    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "no damage should be dealt with 0 remaining jumps"
    );
}

// ── Behavior 14: tick in Idle excludes hit_set cells from selection ──

#[test]
fn tick_idle_excludes_hit_set_cells() {
    let mut app = chain_lightning_damage_test_app();

    let cell_a = spawn_test_cell(&mut app, 10.0, 0.0);
    let cell_b = spawn_test_cell(&mut app, 15.0, 0.0);

    tick(&mut app);

    let mut hit_set = HashSet::new();
    hit_set.insert(cell_a); // cell_a already hit

    let chain_entity = spawn_chain(
        &mut app,
        SpawnChainConfig {
            source: Vec2::new(0.0, 0.0),
            remaining_jumps: 2,
            damage: 15.0,
            range: 25.0,
            arc_speed: 200.0,
            hit_set,
            state: ChainState::Idle,
            source_chip: None,
        },
    );

    tick(&mut app);

    // Chain should target cell_b (not cell_a which is in hit_set)
    let chain = app
        .world()
        .get::<ChainLightningChain>(chain_entity)
        .unwrap();
    match &chain.state {
        ChainState::ArcTraveling { target, .. } => {
            assert_eq!(
                *target, cell_b,
                "should target cell_b since cell_a is in hit_set"
            );
        }
        ChainState::Idle => {
            panic!("chain should transition to ArcTraveling");
        }
    }
}

#[test]
fn tick_idle_only_cell_in_range_is_in_hit_set_despawns_chain() {
    let mut app = chain_lightning_damage_test_app();

    let cell_a = spawn_test_cell(&mut app, 10.0, 0.0);

    tick(&mut app);

    let mut hit_set = HashSet::new();
    hit_set.insert(cell_a);

    let chain_entity = spawn_chain(
        &mut app,
        SpawnChainConfig {
            source: Vec2::new(0.0, 0.0),
            remaining_jumps: 2,
            damage: 15.0,
            range: 25.0,
            arc_speed: 200.0,
            hit_set,
            state: ChainState::Idle,
            source_chip: None,
        },
    );

    tick(&mut app);

    assert!(
        app.world().get_entity(chain_entity).is_err(),
        "chain should despawn when only cell in range is in hit_set"
    );
}
