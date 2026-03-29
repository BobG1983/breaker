use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::plugin::RantzPhysics2dPlugin;

use super::helpers::*;
use crate::{
    cells::messages::DamageCell,
    effect::core::EffectSourceChip,
    shared::{CleanupOnNodeExit, GameRng, GameState, PlayingState},
};

// ── Section B: tick_chain_lightning — Idle State ──────────────────

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

// ── Behavior 15: Arc spawned with marker and chain transitions to ArcTraveling ──

#[test]
fn arc_entity_has_chain_lightning_arc_marker_and_no_extra_fields() {
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
        .query::<(&ChainLightningArc, &Transform, &CleanupOnNodeExit)>();
    let arcs: Vec<_> = arc_query.iter(app.world()).collect();
    assert_eq!(arcs.len(), 1, "expected one arc entity");

    let (_, transform, _) = arcs[0];
    assert!(
        (transform.translation.x - 20.0).abs() < 0.01,
        "arc Transform should be at source position"
    );
}

// ── Section C: tick_chain_lightning — ArcTraveling State ──────────────

// ── Behavior 16: tick advances arc position toward target ──

#[test]
fn tick_arc_traveling_advances_arc_toward_target() {
    let mut app = chain_lightning_damage_test_app();

    let cell = spawn_test_cell(&mut app, 120.0, 0.0);
    tick(&mut app);

    // Spawn arc entity at (20, 0)
    let arc = spawn_arc(&mut app, Vec2::new(20.0, 0.0));

    // Spawn chain in ArcTraveling state pointing at (120, 0)
    let chain_entity = app
        .world_mut()
        .spawn((
            ChainLightningChain {
                source: Vec2::new(20.0, 0.0),
                remaining_jumps: 2,
                damage: 15.0,
                hit_set: HashSet::new(),
                state: ChainState::ArcTraveling {
                    target: cell,
                    target_pos: Vec2::new(120.0, 0.0),
                    arc_entity: arc,
                    arc_pos: Vec2::new(20.0, 0.0),
                },
                range: 25.0,
                arc_speed: 200.0,
            },
            EffectSourceChip(None),
            CleanupOnNodeExit,
        ))
        .id();

    tick(&mut app);

    // Arc should move toward (120, 0) by arc_speed * dt
    // dt = 1/64 = 0.015625, distance_per_tick = 200.0 * 0.015625 = 3.125
    // Direction is (1, 0), so new position should be (23.125, 0, 0)
    let arc_transform = app.world().get::<Transform>(arc).unwrap();
    assert!(
        (arc_transform.translation.x - 23.125).abs() < 0.1,
        "arc should advance by ~3.125 units per tick, expected ~23.125, got {}",
        arc_transform.translation.x
    );
    assert!(
        (arc_transform.translation.y - 0.0).abs() < 0.01,
        "arc y should remain ~0.0"
    );

    // Chain's arc_pos should also be updated
    let chain = app
        .world()
        .get::<ChainLightningChain>(chain_entity)
        .unwrap();
    match &chain.state {
        ChainState::ArcTraveling { arc_pos, .. } => {
            assert!(
                (arc_pos.x - 23.125).abs() < 0.1,
                "chain arc_pos should be updated to ~23.125"
            );
        }
        ChainState::Idle => {
            panic!("chain should still be in ArcTraveling state");
        }
    }
}

// ── Behavior 17: tick damages target and updates chain when arc arrives ──

#[test]
fn tick_arc_arrival_damages_target_and_transitions_to_idle() {
    let mut app = chain_lightning_damage_test_app();

    let cell_b = spawn_test_cell(&mut app, 40.0, 0.0);
    // Another cell to ensure chain doesn't despawn immediately after going idle
    let _cell_c = spawn_test_cell(&mut app, 60.0, 0.0);
    tick(&mut app);

    // Place arc very close to target so it arrives this tick
    // Arc at (39.0, 0), target at (40, 0), distance=1.0
    // arc_speed=200, dt=1/64=0.015625, move=3.125 > 1.0, so arc arrives
    let arc = spawn_arc(&mut app, Vec2::new(39.0, 0.0));

    let chain_entity = app
        .world_mut()
        .spawn((
            ChainLightningChain {
                source: Vec2::new(20.0, 0.0),
                remaining_jumps: 2,
                damage: 15.0,
                hit_set: HashSet::new(),
                state: ChainState::ArcTraveling {
                    target: cell_b,
                    target_pos: Vec2::new(40.0, 0.0),
                    arc_entity: arc,
                    arc_pos: Vec2::new(39.0, 0.0),
                },
                range: 25.0,
                arc_speed: 200.0,
            },
            EffectSourceChip(None),
            CleanupOnNodeExit,
        ))
        .id();

    tick(&mut app);

    // DamageCell should be written for cell_b
    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected 1 DamageCell for cell_b on arc arrival"
    );
    assert_eq!(collector.0[0].cell, cell_b);
    assert!(
        (collector.0[0].damage - 15.0).abs() < f32::EPSILON,
        "damage should be 15.0"
    );

    // remaining_jumps should be decremented
    let chain = app
        .world()
        .get::<ChainLightningChain>(chain_entity)
        .unwrap();
    assert_eq!(
        chain.remaining_jumps, 1,
        "remaining_jumps should be decremented to 1"
    );

    // source should be updated to cell_b's position
    assert!(
        (chain.source.x - 40.0).abs() < 0.01,
        "source should be updated to cell_b position"
    );

    // cell_b should be in hit_set
    assert!(
        chain.hit_set.contains(&cell_b),
        "cell_b should be added to hit_set"
    );

    // Arc entity should be despawned
    assert!(
        app.world().get_entity(arc).is_err(),
        "arc entity should be despawned after arrival"
    );

    // Chain should transition back to Idle
    assert!(
        matches!(chain.state, ChainState::Idle),
        "chain should transition back to Idle after arc arrival"
    );
}

#[test]
fn tick_arc_arrival_at_exact_target_position_triggers_damage() {
    let mut app = chain_lightning_damage_test_app();

    let cell_b = spawn_test_cell(&mut app, 40.0, 0.0);
    let _cell_c = spawn_test_cell(&mut app, 60.0, 0.0);
    tick(&mut app);

    // Arc at exact target position
    let arc = spawn_arc(&mut app, Vec2::new(40.0, 0.0));

    app.world_mut().spawn((
        ChainLightningChain {
            source: Vec2::new(20.0, 0.0),
            remaining_jumps: 2,
            damage: 15.0,
            hit_set: HashSet::new(),
            state: ChainState::ArcTraveling {
                target: cell_b,
                target_pos: Vec2::new(40.0, 0.0),
                arc_entity: arc,
                arc_pos: Vec2::new(40.0, 0.0), // exactly at target
            },
            range: 25.0,
            arc_speed: 200.0,
        },
        EffectSourceChip(None),
        CleanupOnNodeExit,
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "arc at exact target position should trigger damage immediately"
    );
    assert_eq!(collector.0[0].cell, cell_b);
}

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

// ── Behavior 19: Arc arrival uses GlobalPosition2D for next source ──

#[test]
fn tick_arc_arrival_updates_source_to_target_global_position() {
    let mut app = chain_lightning_damage_test_app();

    let cell_b = spawn_test_cell(&mut app, 42.5, 17.3);
    let _cell_c = spawn_test_cell(&mut app, 60.0, 17.3);
    tick(&mut app);

    let arc = spawn_arc(&mut app, Vec2::new(42.0, 17.3));

    let chain_entity = app
        .world_mut()
        .spawn((
            ChainLightningChain {
                source: Vec2::new(20.0, 0.0),
                remaining_jumps: 2,
                damage: 15.0,
                hit_set: HashSet::new(),
                state: ChainState::ArcTraveling {
                    target: cell_b,
                    target_pos: Vec2::new(42.5, 17.3),
                    arc_entity: arc,
                    arc_pos: Vec2::new(42.0, 17.3),
                },
                range: 25.0,
                arc_speed: 200.0,
            },
            EffectSourceChip(None),
            CleanupOnNodeExit,
        ))
        .id();

    tick(&mut app);

    let chain = app
        .world()
        .get::<ChainLightningChain>(chain_entity)
        .unwrap();
    assert!(
        (chain.source.x - 42.5).abs() < 0.01,
        "source should be updated to cell_b GlobalPosition2D x=42.5, got {}",
        chain.source.x
    );
    assert!(
        (chain.source.y - 17.3).abs() < 0.01,
        "source should be updated to cell_b GlobalPosition2D y=17.3, got {}",
        chain.source.y
    );
}

#[test]
fn tick_arc_arrival_target_despawned_chain_transitions_to_idle() {
    let mut app = chain_lightning_damage_test_app();

    let cell_b = spawn_test_cell(&mut app, 40.0, 0.0);
    let _cell_c = spawn_test_cell(&mut app, 60.0, 0.0);
    tick(&mut app);

    let arc = spawn_arc(&mut app, Vec2::new(39.0, 0.0));

    let chain_entity = app
        .world_mut()
        .spawn((
            ChainLightningChain {
                source: Vec2::new(20.0, 0.0),
                remaining_jumps: 2,
                damage: 15.0,
                hit_set: HashSet::new(),
                state: ChainState::ArcTraveling {
                    target: cell_b,
                    target_pos: Vec2::new(40.0, 0.0),
                    arc_entity: arc,
                    arc_pos: Vec2::new(39.0, 0.0),
                },
                range: 25.0,
                arc_speed: 200.0,
            },
            EffectSourceChip(None),
            CleanupOnNodeExit,
        ))
        .id();

    // Despawn the target cell before tick
    app.world_mut().despawn(cell_b);

    tick(&mut app);

    // Chain should not panic, should transition back to idle or despawn
    // Per spec: despawn the arc and transition back to idle
    let chain = app
        .world()
        .get::<ChainLightningChain>(chain_entity)
        .expect("chain entity should still exist after target despawned");
    assert!(
        matches!(chain.state, ChainState::Idle),
        "chain should transition to Idle when target despawned"
    );
    // Arc should be despawned regardless
    assert!(
        app.world().get_entity(arc).is_err(),
        "arc should be despawned when target is missing"
    );
}

// ── Section D: EffectSourceChip Propagation ──────────────────────

// ── Behavior 20: DamageCell from arc arrival includes source_chip ──

#[test]
fn tick_arc_arrival_damage_cell_includes_source_chip() {
    let mut app = chain_lightning_damage_test_app();

    let cell_b = spawn_test_cell(&mut app, 40.0, 0.0);
    let _cell_c = spawn_test_cell(&mut app, 60.0, 0.0);
    tick(&mut app);

    let arc = spawn_arc(&mut app, Vec2::new(39.0, 0.0));

    app.world_mut().spawn((
        ChainLightningChain {
            source: Vec2::new(20.0, 0.0),
            remaining_jumps: 2,
            damage: 15.0,
            hit_set: HashSet::new(),
            state: ChainState::ArcTraveling {
                target: cell_b,
                target_pos: Vec2::new(40.0, 0.0),
                arc_entity: arc,
                arc_pos: Vec2::new(39.0, 0.0),
            },
            range: 25.0,
            arc_speed: 200.0,
        },
        EffectSourceChip(Some("zapper".to_string())),
        CleanupOnNodeExit,
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(
        collector.0[0].source_chip,
        Some("zapper".to_string()),
        "DamageCell from arc arrival should include source_chip"
    );
}

#[test]
fn tick_arc_arrival_damage_cell_source_chip_none_when_effect_source_chip_none() {
    let mut app = chain_lightning_damage_test_app();

    let cell_b = spawn_test_cell(&mut app, 40.0, 0.0);
    let _cell_c = spawn_test_cell(&mut app, 60.0, 0.0);
    tick(&mut app);

    let arc = spawn_arc(&mut app, Vec2::new(39.0, 0.0));

    app.world_mut().spawn((
        ChainLightningChain {
            source: Vec2::new(20.0, 0.0),
            remaining_jumps: 2,
            damage: 15.0,
            hit_set: HashSet::new(),
            state: ChainState::ArcTraveling {
                target: cell_b,
                target_pos: Vec2::new(40.0, 0.0),
                arc_entity: arc,
                arc_pos: Vec2::new(39.0, 0.0),
            },
            range: 25.0,
            arc_speed: 200.0,
        },
        EffectSourceChip(None),
        CleanupOnNodeExit,
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(
        collector.0[0].source_chip, None,
        "DamageCell source_chip should be None when EffectSourceChip is None"
    );
}

// ── Behavior 21b: DamageCell from arc arrival when chain has no EffectSourceChip component ──

#[test]
fn tick_arc_arrival_no_effect_source_chip_component_defaults_to_none() {
    let mut app = chain_lightning_damage_test_app();

    let cell_b = spawn_test_cell(&mut app, 40.0, 0.0);
    let _cell_c = spawn_test_cell(&mut app, 60.0, 0.0);
    tick(&mut app);

    let arc = spawn_arc(&mut app, Vec2::new(39.0, 0.0));

    // Spawn chain WITHOUT EffectSourceChip component
    app.world_mut().spawn((
        ChainLightningChain {
            source: Vec2::new(20.0, 0.0),
            remaining_jumps: 2,
            damage: 15.0,
            hit_set: HashSet::new(),
            state: ChainState::ArcTraveling {
                target: cell_b,
                target_pos: Vec2::new(40.0, 0.0),
                arc_entity: arc,
                arc_pos: Vec2::new(39.0, 0.0),
            },
            range: 25.0,
            arc_speed: 200.0,
        },
        CleanupOnNodeExit,
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(
        collector.0[0].source_chip, None,
        "missing EffectSourceChip should default to source_chip None"
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
            // (or cell_b — the test is that it CAN target cell_a, not that it MUST)
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

    // Spawn a chain in Idle with a valid target — if register() wires the system, tick will process it
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
        "register() should wire tick_chain_lightning — chain should not remain Idle after tick"
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
