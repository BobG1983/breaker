use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::plugin::RantzPhysics2dPlugin;

use super::*;
use crate::{
    cells::messages::DamageCell,
    effect::core::EffectSourceChip,
    shared::{CleanupOnNodeExit, GameRng, GameState, PlayingState},
};

// ── Behavior 18: tick damages target and despawns chain when final jump completes ──

#[test]
fn tick_final_jump_damages_target_and_despawns_chain() {
    let mut app = chain_lightning_damage_test_app();

    let cell_c = spawn_test_cell(&mut app, 60.0, 0.0);
    tick(&mut app);

    // Arc close to target so it arrives this tick
    let arc = spawn_arc(&mut app, Vec2::new(59.0, 0.0));

    let chain_entity = app
        .world_mut()
        .spawn((
            ChainLightningChain {
                source: Vec2::new(40.0, 0.0),
                remaining_jumps: 1, // final jump
                damage: 15.0,
                hit_set: HashSet::new(),
                state: ChainState::ArcTraveling {
                    target: cell_c,
                    target_pos: Vec2::new(60.0, 0.0),
                    arc_entity: arc,
                    arc_pos: Vec2::new(59.0, 0.0),
                },
                range: 25.0,
                arc_speed: 200.0,
            },
            EffectSourceChip(None),
            CleanupOnNodeExit,
        ))
        .id();

    tick(&mut app);

    // DamageCell should be written
    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "final jump should still damage target"
    );
    assert_eq!(collector.0[0].cell, cell_c);

    // Chain entity should be despawned (remaining_jumps goes to 0)
    assert!(
        app.world().get_entity(chain_entity).is_err(),
        "chain should be despawned after final jump"
    );

    // Arc entity should also be despawned
    assert!(
        app.world().get_entity(arc).is_err(),
        "arc should be despawned after final jump"
    );
}

#[test]
fn tick_final_jump_arc_starts_close_to_target_damages_and_despawns_same_tick() {
    let mut app = chain_lightning_damage_test_app();

    let cell_c = spawn_test_cell(&mut app, 60.0, 0.0);
    tick(&mut app);

    // Arc starts very close, will arrive on first tick
    let arc = spawn_arc(&mut app, Vec2::new(59.99, 0.0));

    let chain_entity = app
        .world_mut()
        .spawn((
            ChainLightningChain {
                source: Vec2::new(40.0, 0.0),
                remaining_jumps: 1,
                damage: 10.0,
                hit_set: HashSet::new(),
                state: ChainState::ArcTraveling {
                    target: cell_c,
                    target_pos: Vec2::new(60.0, 0.0),
                    arc_entity: arc,
                    arc_pos: Vec2::new(59.99, 0.0),
                },
                range: 25.0,
                arc_speed: 200.0,
            },
            EffectSourceChip(None),
            CleanupOnNodeExit,
        ))
        .id();

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1, "should damage on first tick");

    assert!(
        app.world().get_entity(chain_entity).is_err(),
        "chain despawned on same tick as damage"
    );
    assert!(
        app.world().get_entity(arc).is_err(),
        "arc despawned on same tick"
    );
}

// ── Section E: Multiple Simultaneous Chains ──────────────────────

// ── Behavior 22: Multiple chains tick independently ──

#[test]
fn multiple_chains_tick_independently() {
    let mut app = chain_lightning_damage_test_app();

    // Cells near chain_1
    let first_near = spawn_test_cell(&mut app, 10.0, 0.0);
    let first_far = spawn_test_cell(&mut app, 20.0, 0.0);

    // Cells near chain_2
    let second_near = spawn_test_cell(&mut app, 210.0, 0.0);
    let second_far = spawn_test_cell(&mut app, 220.0, 0.0);

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

// ── Behavior 23: Chains do not share hit sets ──

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

// ── Section F: reverse() and register() ──────────────────────────

// ── Behavior 24: reverse() is a no-op ──

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

// ── Behavior 25: register() wires tick_chain_lightning in FixedUpdate ──

#[test]
fn register_wires_tick_chain_lightning_system() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.add_plugins(RantzPhysics2dPlugin);
    app.init_state::<GameState>();
    app.add_sub_state::<PlayingState>();
    app.add_message::<DamageCell>();
    app.insert_resource(DamageCellCollector::default());
    app.add_systems(Update, collect_damage_cells);
    app.insert_resource(GameRng::from_seed(42));

    register(&mut app);

    // Transition to PlayingState::Active
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    // Spawn a chain in Idle with a valid target -- if register() wires the system, tick will process it
    let cell = spawn_test_cell(&mut app, 10.0, 0.0);
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

// ── Section G: Lifecycle and Cleanup ─────────────────────────────

// ── Behavior 26: ChainLightningChain has CleanupOnNodeExit (covered in fire tests) ──
// Already tested in fire_chain_entity_has_cleanup_on_node_exit

// ── Behavior 27: ChainLightningArc has CleanupOnNodeExit ──

#[test]
fn arc_entity_has_cleanup_on_node_exit() {
    let mut app = chain_lightning_damage_test_app();

    let cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let cell_b = spawn_test_cell(&mut app, 40.0, 0.0);

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
            app.world().get::<CleanupOnNodeExit>(arc_entity).is_some(),
            "ChainLightningArc entity should have CleanupOnNodeExit"
        );
    } else {
        panic!("expected an arc entity to be spawned");
    }
}

// ── Behavior 28: Arc entity despawned after reaching target ──
// Already tested in tick_arc_arrival_damages_target_and_transitions_to_idle

// ── Behavior 29: Chain despawns when no valid targets remain mid-chain ──

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
