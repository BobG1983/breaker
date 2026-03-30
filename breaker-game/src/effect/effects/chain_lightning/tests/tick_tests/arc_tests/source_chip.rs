//! Tests for `EffectSourceChip` propagation through arc arrival `DamageCell` messages.

use std::collections::HashSet;

use bevy::prelude::*;

use super::*;
use crate::{effect::core::EffectSourceChip, shared::CleanupOnNodeExit};

// -- Behavior 20: DamageCell from arc arrival includes source_chip --

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

// -- Behavior 21b: DamageCell from arc arrival when chain has no EffectSourceChip component --

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
