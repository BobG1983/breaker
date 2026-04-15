use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::system::*;
use crate::{
    cells::components::Cell,
    effect_v3::{components::EffectSourceChip, effects::chain_lightning::components::*},
    shared::{
        death_pipeline::DamageDealt,
        rng::GameRng,
        test_utils::{MessageCollector, TestAppBuilder, tick},
    },
};

fn chain_test_app() -> App {
    TestAppBuilder::new()
        .with_message_capture::<DamageDealt<Cell>>()
        .insert_resource(GameRng::from_seed(42))
        .with_system(FixedUpdate, tick_chain_lightning)
        .build()
}

/// Spawns a minimal Cell entity at the given position.
fn spawn_cell(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut().spawn((Cell, Position2D(pos))).id()
}

// ── C3: ChainLightning random targeting ────────────────────────────────

#[test]
fn chain_lightning_selects_random_cell_not_always_nearest() {
    // Run with two different seeds and verify that at least one seed
    // picks a non-nearest cell.
    let seeds = [42_u64, 99, 7, 123, 255, 1000, 31337, 9999];
    let mut targets_by_seed: Vec<Entity> = Vec::new();

    for &seed in &seeds {
        let mut app = TestAppBuilder::new()
            .with_message_capture::<DamageDealt<Cell>>()
            .insert_resource(GameRng::from_seed(seed))
            .with_system(FixedUpdate, tick_chain_lightning)
            .build();

        let _closest = spawn_cell(&mut app, Vec2::new(50.0, 0.0));
        let _mid = spawn_cell(&mut app, Vec2::new(100.0, 0.0));
        let _farthest = spawn_cell(&mut app, Vec2::new(150.0, 0.0));

        // Spawn chain in Idle state.
        app.world_mut().spawn(ChainLightningChain {
            remaining_jumps: 1,
            damage:          10.0,
            hit_set:         HashSet::new(),
            state:           ChainState::Idle,
            range:           200.0,
            arc_speed:       5000.0,
            source_pos:      Vec2::new(0.0, 0.0),
        });

        tick(&mut app);

        // After one tick, the chain should be in ArcTraveling toward some cell.
        let chains: Vec<&ChainLightningChain> = app
            .world_mut()
            .query::<&ChainLightningChain>()
            .iter(app.world())
            .collect();

        if let Some(chain) = chains.first()
            && let ChainState::ArcTraveling { target, .. } = &chain.state
        {
            targets_by_seed.push(*target);
        }
    }

    assert_eq!(
        targets_by_seed.len(),
        seeds.len(),
        "expected all seeds to produce a target"
    );

    // The key assertion: across many seeds, at least one target should NOT be
    // the closest cell. With 8 seeds and 3 candidates, it's statistically
    // near-certain that at least one seed picks a non-nearest cell.
    let mut selected_non_nearest = false;
    for &seed in &seeds {
        let mut app = TestAppBuilder::new()
            .with_message_capture::<DamageDealt<Cell>>()
            .insert_resource(GameRng::from_seed(seed))
            .with_system(FixedUpdate, tick_chain_lightning)
            .build();

        let _closest = spawn_cell(&mut app, Vec2::new(50.0, 0.0));
        let _mid = spawn_cell(&mut app, Vec2::new(100.0, 0.0));
        let _farthest = spawn_cell(&mut app, Vec2::new(150.0, 0.0));

        app.world_mut().spawn(ChainLightningChain {
            remaining_jumps: 1,
            damage:          10.0,
            hit_set:         HashSet::new(),
            state:           ChainState::Idle,
            range:           200.0,
            arc_speed:       5000.0,
            source_pos:      Vec2::new(0.0, 0.0),
        });

        tick(&mut app);

        let chains: Vec<&ChainLightningChain> = app
            .world_mut()
            .query::<&ChainLightningChain>()
            .iter(app.world())
            .collect();

        if let Some(chain) = chains.first()
            && let ChainState::ArcTraveling { target_pos, .. } = &chain.state
            && (target_pos.x - 50.0).abs() > f32::EPSILON
        {
            selected_non_nearest = true;
        }
    }

    assert!(
        selected_non_nearest,
        "chain lightning should randomly select targets, not always nearest. \
         Both seeds selected the nearest cell at (50, 0).",
    );
}

#[test]
fn chain_lightning_selects_only_cell_when_one_in_range() {
    let mut app = chain_test_app();

    let only_cell = spawn_cell(&mut app, Vec2::new(50.0, 0.0));

    app.world_mut().spawn(ChainLightningChain {
        remaining_jumps: 1,
        damage:          10.0,
        hit_set:         HashSet::new(),
        state:           ChainState::Idle,
        range:           200.0,
        arc_speed:       5000.0,
        source_pos:      Vec2::new(0.0, 0.0),
    });

    tick(&mut app);

    let chains: Vec<&ChainLightningChain> = app
        .world_mut()
        .query::<&ChainLightningChain>()
        .iter(app.world())
        .collect();
    assert_eq!(chains.len(), 1);
    if let ChainState::ArcTraveling { target, .. } = &chains[0].state {
        assert_eq!(
            *target, only_cell,
            "with only one cell in range, it must be selected"
        );
    } else {
        panic!("expected chain to be in ArcTraveling state");
    }
}

#[test]
fn chain_lightning_excludes_already_hit_cells() {
    let mut app = chain_test_app();

    let cell_a = spawn_cell(&mut app, Vec2::new(50.0, 0.0));
    let _cell_b = spawn_cell(&mut app, Vec2::new(100.0, 0.0));
    let _cell_c = spawn_cell(&mut app, Vec2::new(150.0, 0.0));

    let mut hit_set = HashSet::new();
    hit_set.insert(cell_a);

    app.world_mut().spawn(ChainLightningChain {
        remaining_jumps: 1,
        damage: 10.0,
        hit_set,
        state: ChainState::Idle,
        range: 200.0,
        arc_speed: 5000.0,
        source_pos: Vec2::new(0.0, 0.0),
    });

    tick(&mut app);

    let chains: Vec<&ChainLightningChain> = app
        .world_mut()
        .query::<&ChainLightningChain>()
        .iter(app.world())
        .collect();

    if let Some(chain) = chains.first()
        && let ChainState::ArcTraveling { target, .. } = &chain.state
    {
        assert_ne!(
            *target, cell_a,
            "chain should not target already-hit cell A"
        );
    }
}

#[test]
fn chain_lightning_despawns_when_all_cells_in_hit_set() {
    let mut app = chain_test_app();

    let cell_a = spawn_cell(&mut app, Vec2::new(50.0, 0.0));
    let cell_b = spawn_cell(&mut app, Vec2::new(100.0, 0.0));

    let mut hit_set = HashSet::new();
    hit_set.insert(cell_a);
    hit_set.insert(cell_b);

    app.world_mut().spawn(ChainLightningChain {
        remaining_jumps: 1,
        damage: 10.0,
        hit_set,
        state: ChainState::Idle,
        range: 200.0,
        arc_speed: 5000.0,
        source_pos: Vec2::new(0.0, 0.0),
    });

    tick(&mut app);

    let chain_count = app
        .world_mut()
        .query::<&ChainLightningChain>()
        .iter(app.world())
        .count();
    assert_eq!(
        chain_count, 0,
        "chain should despawn when no valid targets remain"
    );
}

// ── C4: ChainLightning source_chip propagation ─────────────────────────

#[test]
fn chain_lightning_propagates_source_chip_in_damage_dealt() {
    let mut app = chain_test_app();

    let cell = spawn_cell(&mut app, Vec2::new(50.0, 0.0));

    // Spawn an arc entity for the arc visual.
    let arc_entity = app
        .world_mut()
        .spawn((ChainLightningArc, Position2D(Vec2::new(49.0, 0.0))))
        .id();

    // Spawn chain in ArcTraveling state, very close to target so it arrives this tick.
    app.world_mut().spawn((
        ChainLightningChain {
            remaining_jumps: 1,
            damage:          15.0,
            hit_set:         HashSet::new(),
            state:           ChainState::ArcTraveling {
                target: cell,
                target_pos: Vec2::new(50.0, 0.0),
                arc_entity,
                arc_pos: Vec2::new(49.0, 0.0),
            },
            range:           200.0,
            arc_speed:       5000.0,
            source_pos:      Vec2::ZERO,
        },
        EffectSourceChip(Some("storm_chip".to_string())),
    ));

    tick(&mut app);

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1, "expected 1 DamageDealt<Cell> message");
    assert_eq!(
        msgs.0[0].source_chip,
        Some("storm_chip".to_string()),
        "DamageDealt should carry source_chip from EffectSourceChip, got {:?}",
        msgs.0[0].source_chip,
    );
}

#[test]
fn chain_lightning_propagates_none_source_chip() {
    let mut app = chain_test_app();

    let cell = spawn_cell(&mut app, Vec2::new(50.0, 0.0));

    let arc_entity = app
        .world_mut()
        .spawn((ChainLightningArc, Position2D(Vec2::new(49.0, 0.0))))
        .id();

    app.world_mut().spawn((
        ChainLightningChain {
            remaining_jumps: 1,
            damage:          15.0,
            hit_set:         HashSet::new(),
            state:           ChainState::ArcTraveling {
                target: cell,
                target_pos: Vec2::new(50.0, 0.0),
                arc_entity,
                arc_pos: Vec2::new(49.0, 0.0),
            },
            range:           200.0,
            arc_speed:       5000.0,
            source_pos:      Vec2::ZERO,
        },
        EffectSourceChip(None),
    ));

    tick(&mut app);

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1, "expected 1 DamageDealt<Cell> message");
    assert_eq!(
        msgs.0[0].source_chip, None,
        "DamageDealt should carry None source_chip from EffectSourceChip(None), got {:?}",
        msgs.0[0].source_chip,
    );
}
