//! Tests for `tick_chain_lightning` -- `ArcTraveling` state arc movement.

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::*;
use crate::{effect::core::EffectSourceChip, shared::CleanupOnNodeExit};

// -- Behavior 16: tick advances arc position toward target --

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
    let arc_position = app.world().get::<Position2D>(arc).unwrap();
    assert!(
        (arc_position.0.x - 23.125).abs() < 0.1,
        "arc should advance by ~3.125 units per tick, expected ~23.125, got {}",
        arc_position.0.x
    );
    assert!(
        (arc_position.0.y - 0.0).abs() < 0.01,
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
