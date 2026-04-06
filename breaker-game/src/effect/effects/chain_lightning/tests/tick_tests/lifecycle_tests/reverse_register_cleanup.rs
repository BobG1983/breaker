//! Tests for `reverse()`, `register()`, `CleanupOnExit<NodeState>`, and chain despawn on no valid targets.

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_lifecycle::CleanupOnExit;
use rantzsoft_physics2d::plugin::RantzPhysics2dPlugin;

use super::*;
use crate::{
    cells::messages::DamageCell,
    shared::GameRng,
    state::types::{AppState, GameState, NodeState, RunState},
};

// -- Behavior 24: reverse() is a no-op --

#[test]
fn reverse_is_noop() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(10.0, 20.0, 0.0)).id();

    reverse(entity, "", &mut world);

    assert!(
        world.get_entity(entity).is_ok(),
        "entity should still exist after no-op reverse"
    );
}

#[test]
fn reverse_on_empty_entity_is_noop() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    reverse(entity, "", &mut world);

    assert!(
        world.get_entity(entity).is_ok(),
        "empty entity should still exist after no-op reverse"
    );
}

// -- Behavior 25: register() wires tick_chain_lightning in FixedUpdate --

#[test]
fn register_wires_tick_chain_lightning_system() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.add_plugins(RantzPhysics2dPlugin);
    app.init_state::<AppState>();
    app.add_sub_state::<GameState>();
    app.add_sub_state::<RunState>();
    app.add_sub_state::<NodeState>();
    app.add_message::<DamageCell>();
    app.insert_resource(DamageCellCollector::default());
    app.add_systems(Update, collect_damage_cells);
    app.insert_resource(GameRng::from_seed(42));

    register(&mut app);

    // Navigate to NodeState::Playing so run_if(in_state(NodeState::Playing)) passes
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Game);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Run);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<RunState>>()
        .set(RunState::Node);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();

    // Spawn a chain in Idle with a valid target -- if register() wires the system, tick will process it
    let _cell = spawn_test_cell(&mut app, 10.0, 0.0);
    tick(&mut app);

    let chain_entity = spawn_chain(
        &mut app,
        SpawnChainConfig {
            source: Vec2::new(0.0, 0.0),
            remaining_jumps: 1,
            damage: 10.0,
            range: 25.0,
            arc_speed: 200.0,
            hit_set: HashSet::new(),
            state: ChainState::Idle,
            source_chip: None,
        },
    );

    // Tick to run tick_chain_lightning
    tick(&mut app);

    // The chain should have transitioned (arc spawned or chain despawned)
    // If the system runs, it should either spawn an arc or process the chain
    let chain_still_idle = app
        .world()
        .get::<ChainLightningChain>(chain_entity)
        .is_some_and(|c| matches!(c.state, ChainState::Idle));

    assert!(
        !chain_still_idle,
        "register() should wire tick_chain_lightning -- chain should not remain Idle after tick"
    );
}

// -- Behavior 27: ChainLightningArc has CleanupOnExit<NodeState> --

#[test]
fn arc_entity_has_cleanup_on_node_exit() {
    let mut app = chain_lightning_damage_test_app();

    let cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 40.0, 0.0);

    tick(&mut app);

    let mut hit_set = HashSet::new();
    hit_set.insert(cell_a);

    spawn_chain(
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

    let mut arc_query = app
        .world_mut()
        .query_filtered::<Entity, With<ChainLightningArc>>();
    if let Some(arc_entity) = arc_query.iter(app.world()).next() {
        assert!(
            app.world()
                .get::<CleanupOnExit<NodeState>>(arc_entity)
                .is_some(),
            "ChainLightningArc entity should have CleanupOnExit<NodeState>"
        );
    } else {
        panic!("expected an arc entity to be spawned");
    }
}

// -- Behavior 29: Chain despawns when no valid targets remain mid-chain --

#[test]
fn chain_despawns_when_all_cells_in_range_are_in_hit_set() {
    let mut app = chain_lightning_damage_test_app();

    let cell_a = spawn_test_cell(&mut app, 10.0, 0.0);
    let cell_b = spawn_test_cell(&mut app, 15.0, 0.0);

    tick(&mut app);

    let mut hit_set = HashSet::new();
    hit_set.insert(cell_a);
    hit_set.insert(cell_b);

    let chain_entity = spawn_chain(
        &mut app,
        SpawnChainConfig {
            source: Vec2::new(0.0, 0.0),
            remaining_jumps: 2, // has jumps remaining
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
        "chain should despawn when all cells in range are in hit_set"
    );

    let mut arc_query = app.world_mut().query::<&ChainLightningArc>();
    assert!(
        arc_query.iter(app.world()).next().is_none(),
        "no arc should be spawned"
    );
}
