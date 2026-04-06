//! Tests for final jump: damage target and despawn chain when remaining jumps reach zero.

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_lifecycle::CleanupOnExit;

use super::*;
use crate::{effect::core::EffectSourceChip, state::types::NodeState};

// -- Behavior 18: tick damages target and despawns chain when final jump completes --

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
            CleanupOnExit::<NodeState>::default(),
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
            CleanupOnExit::<NodeState>::default(),
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
