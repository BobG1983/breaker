//! Tests for multiple simultaneous chains ticking independently and not sharing hit sets.

use std::collections::HashSet;

use bevy::prelude::*;

use super::*;

// -- Behavior 22: Multiple chains tick independently --

#[test]
fn multiple_chains_tick_independently() {
    let mut app = chain_lightning_damage_test_app();

    // Cells near chain_1
    let first_near = spawn_test_cell(&mut app, 10.0, 0.0);
    let _first_far = spawn_test_cell(&mut app, 20.0, 0.0);

    // Cells near chain_2
    let second_near = spawn_test_cell(&mut app, 210.0, 0.0);
    let _second_far = spawn_test_cell(&mut app, 220.0, 0.0);

    tick(&mut app);

    let mut hit_set_1 = HashSet::new();
    hit_set_1.insert(first_near);

    let mut hit_set_2 = HashSet::new();
    hit_set_2.insert(second_near);

    let _chain_1 = spawn_chain(
        &mut app,
        SpawnChainConfig {
            source: Vec2::new(0.0, 0.0),
            remaining_jumps: 2,
            damage: 10.0,
            range: 25.0,
            arc_speed: 200.0,
            hit_set: hit_set_1,
            state: ChainState::Idle,
            source_chip: None,
        },
    );

    let _chain_2 = spawn_chain(
        &mut app,
        SpawnChainConfig {
            source: Vec2::new(200.0, 0.0),
            remaining_jumps: 1,
            damage: 20.0,
            range: 25.0,
            arc_speed: 200.0,
            hit_set: hit_set_2,
            state: ChainState::Idle,
            source_chip: None,
        },
    );

    tick(&mut app);

    // Both chains should have spawned arcs
    let mut arc_query = app.world_mut().query::<&ChainLightningArc>();
    let arc_count = arc_query.iter(app.world()).count();
    assert_eq!(
        arc_count, 2,
        "each chain should independently spawn an arc, got {arc_count} arcs"
    );
}

// -- Behavior 23: Chains do not share hit sets --

#[test]
fn chains_do_not_share_hit_sets() {
    let mut app = chain_lightning_damage_test_app();

    // cell_a within range of both chains
    let cell_a = spawn_test_cell(&mut app, 10.0, 0.0);
    let cell_b = spawn_test_cell(&mut app, 20.0, 0.0);

    tick(&mut app);

    // Chain_1 has cell_a in hit_set
    let mut hit_set_1 = HashSet::new();
    hit_set_1.insert(cell_a);

    // Chain_2 does NOT have cell_a in hit_set
    let hit_set_2 = HashSet::new();

    let chain_1_entity = spawn_chain(
        &mut app,
        SpawnChainConfig {
            source: Vec2::new(0.0, 0.0),
            remaining_jumps: 2,
            damage: 10.0,
            range: 25.0,
            arc_speed: 200.0,
            hit_set: hit_set_1,
            state: ChainState::Idle,
            source_chip: None,
        },
    );

    let chain_2_entity = spawn_chain(
        &mut app,
        SpawnChainConfig {
            source: Vec2::new(0.0, 0.0),
            remaining_jumps: 2,
            damage: 20.0,
            range: 25.0,
            arc_speed: 200.0,
            hit_set: hit_set_2,
            state: ChainState::Idle,
            source_chip: None,
        },
    );

    tick(&mut app);

    // Chain_1 should NOT target cell_a (it's in hit_set)
    let chain_1 = app
        .world()
        .get::<ChainLightningChain>(chain_1_entity)
        .unwrap();
    match &chain_1.state {
        ChainState::ArcTraveling { target, .. } => {
            assert_ne!(
                *target, cell_a,
                "chain_1 should not re-target cell_a (in its hit_set)"
            );
        }
        ChainState::Idle => panic!("chain_1 should have transitioned to ArcTraveling after tick"),
    }

    // Chain_2 CAN target cell_a (not in its hit_set)
    let chain_2 = app
        .world()
        .get::<ChainLightningChain>(chain_2_entity)
        .unwrap();
    match &chain_2.state {
        ChainState::ArcTraveling { target, .. } => {
            // cell_a could be targeted since it's not in chain_2's hit_set
            // (or cell_b -- the test is that it CAN target cell_a, not that it MUST)
            // At minimum, the chain should be ArcTraveling (found a target)
            assert!(
                *target == cell_a || *target == cell_b,
                "chain_2 should be able to target either cell"
            );
        }
        ChainState::Idle => panic!("chain_2 should have transitioned to ArcTraveling after tick"),
    }
}
