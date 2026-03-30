use std::collections::HashSet;

use bevy::prelude::*;

use super::*;
use crate::{effect::core::EffectSourceChip, shared::CleanupOnNodeExit};

// ── Behavior 15: Arc spawned with marker and chain transitions to ArcTraveling ──

#[test]
fn arc_entity_has_chain_lightning_arc_marker_and_no_extra_fields() {
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
        .query::<(&ChainLightningArc, &Transform, &CleanupOnNodeExit)>();
    let arcs: Vec<_> = arc_query.iter(app.world()).collect();
    assert_eq!(arcs.len(), 1, "expected one arc entity");

    let (_, transform, _) = arcs[0];
    assert!(
        (transform.translation.x - 20.0).abs() < 0.01,
        "arc Transform should be at source position"
    );
}

// ── Section C: tick_chain_lightning -- ArcTraveling State ──────────────

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
