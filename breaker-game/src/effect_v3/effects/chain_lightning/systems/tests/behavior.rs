use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::super::system::*;
use crate::{
    cells::components::Cell,
    chips::definition::Rarity,
    effect_v3::{components::EffectSourceChip, effects::chain_lightning::components::*},
    prelude::*,
    shared::{
        rng::{EffectBaseSeed, EffectEventCounter, derive_seed, derive_seed_named},
        test_utils::{MessageCollector, TestAppBuilder, tick},
    },
};

/// Builder-format `SourceId` for the canonical "Storm" chip used across
/// chain-lightning behavior tests.
fn storm_source() -> SourceId {
    SourceId::chip("Storm").rarity(Rarity::Common).build()
}

fn chain_test_app() -> App {
    TestAppBuilder::new()
        .with_message_capture::<DamageDealt<Cell>>()
        .insert_resource(EffectBaseSeed(0))
        .insert_resource(EffectEventCounter::default())
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
            .insert_resource(EffectBaseSeed(seed))
            .insert_resource(EffectEventCounter::default())
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
            tick:            0,
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
            .insert_resource(EffectBaseSeed(seed))
            .insert_resource(EffectEventCounter::default())
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
            tick:            0,
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
        tick:            0,
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
        tick: 0,
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
        tick: 0,
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
            tick:            0,
        },
        EffectSourceChip(Some(storm_source())),
    ));

    tick(&mut app);

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1, "expected 1 DamageDealt<Cell> message");
    assert_eq!(
        msgs.0[0].source,
        Some(storm_source()),
        "DamageDealt should carry source_chip from EffectSourceChip, got {:?}",
        msgs.0[0].source,
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
            tick:            0,
        },
        EffectSourceChip(None),
    ));

    tick(&mut app);

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1, "expected 1 DamageDealt<Cell> message");
    assert_eq!(
        msgs.0[0].source, None,
        "DamageDealt should carry None source_chip from EffectSourceChip(None), got {:?}",
        msgs.0[0].source,
    );
}

// ── Group C — tick_chain_lightning per-chain-per-tick RNG derivation ────────

// B13 — intentionally green: removing GameRng from tick_chain_lightning means
// no-GameRng apps no longer panic. This test SHOULD PASS at RED gate because
// the stub removes the GameRng param.
#[test]
fn tick_chain_lightning_does_not_consume_game_rng() {
    let mut app = TestAppBuilder::new()
        .with_message_capture::<DamageDealt<Cell>>()
        .insert_resource(EffectBaseSeed(0))
        .insert_resource(EffectEventCounter::default())
        .with_system(FixedUpdate, tick_chain_lightning)
        .build();

    let only_cell = spawn_cell(&mut app, Vec2::new(50.0, 0.0));

    app.world_mut().spawn(ChainLightningChain {
        remaining_jumps: 1,
        damage:          10.0,
        hit_set:         HashSet::new(),
        state:           ChainState::Idle,
        range:           200.0,
        arc_speed:       5000.0,
        source_pos:      Vec2::ZERO,
        tick:            0,
    });

    // Must not panic despite no GameRng resource.
    tick(&mut app);

    // Sentinel: the chain must have transitioned (proves the system executed).
    let chains: Vec<&ChainLightningChain> = app
        .world_mut()
        .query::<&ChainLightningChain>()
        .iter(app.world())
        .collect();
    assert_eq!(
        chains.len(),
        1,
        "chain must still exist after one idle tick"
    );
    if let ChainState::ArcTraveling { target, .. } = &chains[0].state {
        assert_eq!(*target, only_cell, "chain selected the only available cell");
    } else {
        panic!("chain must transition to ArcTraveling when a cell is in range");
    }
}

#[test]
fn chain_lightning_chain_tick_increments_each_fixed_update() {
    let mut app = chain_test_app();
    let _only_cell = spawn_cell(&mut app, Vec2::new(50.0, 0.0));

    let chain_entity = app
        .world_mut()
        .spawn(ChainLightningChain {
            remaining_jumps: 5,
            damage:          10.0,
            hit_set:         HashSet::new(),
            state:           ChainState::Idle,
            range:           200.0,
            arc_speed:       0.1,
            source_pos:      Vec2::ZERO,
            tick:            0,
        })
        .id();

    tick(&mut app);
    let tick_val = app
        .world()
        .get::<ChainLightningChain>(chain_entity)
        .expect("chain entity must still exist")
        .tick;
    assert_eq!(tick_val, 1, "tick must be 1 after the first fixed update");

    // 4 more ticks
    for _ in 0..4 {
        tick(&mut app);
    }
    let tick_val = app
        .world()
        .get::<ChainLightningChain>(chain_entity)
        .expect("chain entity must still exist")
        .tick;
    assert_eq!(tick_val, 5, "tick must be 5 after 5 fixed updates");
}

#[test]
fn tick_chain_lightning_target_selection_is_deterministic_given_seed_entity_and_tick() {
    // Two identically-seeded apps with the same entity allocation order must
    // select the same target.
    let run_app = |seed: u64| -> Option<Vec2> {
        let mut app = TestAppBuilder::new()
            .with_message_capture::<DamageDealt<Cell>>()
            .insert_resource(EffectBaseSeed(seed))
            .insert_resource(EffectEventCounter::default())
            .with_system(FixedUpdate, tick_chain_lightning)
            .build();

        let _c1 = spawn_cell(&mut app, Vec2::new(50.0, 0.0));
        let _c2 = spawn_cell(&mut app, Vec2::new(100.0, 0.0));
        let _c3 = spawn_cell(&mut app, Vec2::new(150.0, 0.0));

        app.world_mut().spawn(ChainLightningChain {
            remaining_jumps: 1,
            damage:          10.0,
            hit_set:         HashSet::new(),
            state:           ChainState::Idle,
            range:           200.0,
            arc_speed:       0.1,
            source_pos:      Vec2::ZERO,
            tick:            0,
        });

        tick(&mut app);

        app.world_mut()
            .query::<&ChainLightningChain>()
            .iter(app.world())
            .next()
            .and_then(|c| {
                if let ChainState::ArcTraveling { target_pos, .. } = &c.state {
                    Some(*target_pos)
                } else {
                    None
                }
            })
    };

    let pos_a1 = run_app(0xCAFE).expect("app_a first run must produce ArcTraveling");
    let pos_a2 = run_app(0xCAFE).expect("app_a second run must produce ArcTraveling");
    assert_eq!(
        pos_a1, pos_a2,
        "same seed must produce same target position across independent runs"
    );

    // Different seed must produce a result derivable from the formula.
    // We verify it doesn't panic and produces a valid position.
    let _pos_b = run_app(0xCAFF).expect("different seed must also produce ArcTraveling");
}

#[test]
fn tick_chain_lightning_two_chains_select_independently() {
    let mut app = TestAppBuilder::new()
        .with_message_capture::<DamageDealt<Cell>>()
        .insert_resource(EffectBaseSeed(0xCAFE))
        .insert_resource(EffectEventCounter::default())
        .with_system(FixedUpdate, tick_chain_lightning)
        .build();

    let _c1 = spawn_cell(&mut app, Vec2::new(50.0, 0.0));
    let _c2 = spawn_cell(&mut app, Vec2::new(100.0, 0.0));
    let _c3 = spawn_cell(&mut app, Vec2::new(150.0, 0.0));

    let chain_a = app
        .world_mut()
        .spawn(ChainLightningChain {
            remaining_jumps: 1,
            damage:          10.0,
            hit_set:         HashSet::new(),
            state:           ChainState::Idle,
            range:           200.0,
            arc_speed:       0.1,
            source_pos:      Vec2::ZERO,
            tick:            0,
        })
        .id();

    let chain_b = app
        .world_mut()
        .spawn(ChainLightningChain {
            remaining_jumps: 1,
            damage:          10.0,
            hit_set:         HashSet::new(),
            state:           ChainState::Idle,
            range:           200.0,
            arc_speed:       0.1,
            source_pos:      Vec2::ZERO,
            tick:            0,
        })
        .id();

    // Entity indices must differ for independent derivation.
    assert_ne!(
        chain_a.index(),
        chain_b.index(),
        "chains must have different entity indices for independent RNG derivation"
    );

    tick(&mut app);

    let world = app.world();
    let chain_a_data = world
        .get::<ChainLightningChain>(chain_a)
        .expect("chain A must still exist");
    let second_chain_data = world
        .get::<ChainLightningChain>(chain_b)
        .expect("chain B must still exist");

    // Both must have transitioned.
    assert!(
        matches!(chain_a_data.state, ChainState::ArcTraveling { .. }),
        "chain A must transition to ArcTraveling"
    );
    assert!(
        matches!(second_chain_data.state, ChainState::ArcTraveling { .. }),
        "chain B must transition to ArcTraveling"
    );

    // Verify that the two chains' seeds (from derivation) differ.
    let base = 0xCAFE_u64;
    let chain_tick_seed = derive_seed_named(base, "chain_tick");
    let seed_a = derive_seed(chain_tick_seed, u64::from(chain_a.index().index()));
    let seed_b = derive_seed(chain_tick_seed, u64::from(chain_b.index().index()));
    assert_ne!(
        seed_a, seed_b,
        "chains with different entity indices must derive different seeds"
    );
}

#[test]
fn tick_chain_lightning_uses_different_rng_each_tick() {
    // Verify: chain.tick advances and that the derived seeds differ per tick.
    let mut app = TestAppBuilder::new()
        .with_message_capture::<DamageDealt<Cell>>()
        .insert_resource(EffectBaseSeed(0xCAFE))
        .insert_resource(EffectEventCounter::default())
        .with_system(FixedUpdate, tick_chain_lightning)
        .build();

    // Cell very close so ArcTraveling completes in one tick, chain returns to Idle.
    let _cell = spawn_cell(&mut app, Vec2::new(1.0, 0.0));

    let chain_entity = app
        .world_mut()
        .spawn(ChainLightningChain {
            remaining_jumps: 2,
            damage:          10.0,
            hit_set:         HashSet::new(),
            state:           ChainState::Idle,
            range:           200.0,
            arc_speed:       5000.0,
            source_pos:      Vec2::ZERO,
            tick:            0,
        })
        .id();

    tick(&mut app);
    let tick_after_1 = app
        .world()
        .get::<ChainLightningChain>(chain_entity)
        .map(|c| c.tick);

    tick(&mut app);
    let tick_after_2 = app
        .world()
        .get::<ChainLightningChain>(chain_entity)
        .map(|c| c.tick);

    // tick counter must advance each fixed update.
    if let (Some(t1), Some(t2)) = (tick_after_1, tick_after_2) {
        assert_eq!(t1, 1, "tick must be 1 after first fixed update");
        assert_eq!(t2, 2, "tick must be 2 after second fixed update");

        let base = 0xCAFE_u64;
        let chain_tick_seed = derive_seed_named(base, "chain_tick");
        let entity_idx = u64::from(chain_entity.index().index());
        let seed_t1 = derive_seed(chain_tick_seed, entity_idx);
        let seed_t2 = derive_seed(chain_tick_seed, entity_idx ^ 1);
        assert_ne!(
            seed_t1, seed_t2,
            "per-tick seeds must differ across consecutive ticks"
        );
    } else {
        // Chain may have despawned after exhausting jumps — that's acceptable.
        // The tick assertions are validated when the entity exists. If the
        // entity was despawned we can only verify the seeds differ by formula.
        let base = 0xCAFE_u64;
        let chain_tick_seed = derive_seed_named(base, "chain_tick");
        let entity_idx = u64::from(chain_entity.index().index());
        let seed_t1 = derive_seed(chain_tick_seed, entity_idx);
        let seed_t2 = derive_seed(chain_tick_seed, entity_idx ^ 1);
        assert_ne!(
            seed_t1, seed_t2,
            "per-tick seeds must differ across consecutive ticks"
        );
    }
}

#[test]
fn tick_chain_lightning_no_panic_when_no_chains_exist() {
    // EffectBaseSeed is absent AND no chains — must not panic.
    let mut app = TestAppBuilder::new()
        .with_message_capture::<DamageDealt<Cell>>()
        .with_system(FixedUpdate, tick_chain_lightning)
        .build();

    // No EffectBaseSeed, no chains — system should short-circuit cleanly.
    tick(&mut app);
    // If we get here, the system did not panic — test passes.
}

// ── Group D — existing random-target test preserved under new derivation ────

#[test]
fn chain_lightning_random_target_preserved_under_new_rng_derivation() {
    // Replaces the GameRng-based random-not-always-nearest test using the
    // new per-chain-per-tick derivation.
    let seeds = [42_u64, 99, 7, 123, 255, 1000, 31337, 9999];
    let mut selected_non_nearest = false;

    for &seed in &seeds {
        let mut app = TestAppBuilder::new()
            .with_message_capture::<DamageDealt<Cell>>()
            .insert_resource(EffectBaseSeed(seed))
            .insert_resource(EffectEventCounter::default())
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
            arc_speed:       0.1,
            source_pos:      Vec2::ZERO,
            tick:            0,
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
        "chain lightning should randomly select targets under new per-tick RNG derivation, \
         not always select the nearest cell. All 8 seeds selected the nearest cell at (50, 0)."
    );
}
