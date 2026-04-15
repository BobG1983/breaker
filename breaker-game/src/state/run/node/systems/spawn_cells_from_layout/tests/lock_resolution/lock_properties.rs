//! Lock component properties (hp, `required_to_clear`) — Section E + G

use std::collections::HashMap;

use bevy::prelude::*;

use super::{super::helpers::*, helpers::*};
use crate::{
    cells::components::*,
    shared::death_pipeline::hp::Hp,
    state::run::node::{
        NodeLayout,
        definition::{LockMap, NodePool},
    },
};

#[test]
fn locked_cell_with_required_to_clear_has_both_components() {
    // Edge case: locked cell that is also required_to_clear
    // "S" in test_registry has required_to_clear=true
    let layout = simple_locked_layout();
    let mut app = test_app(layout.clone());
    app.update();

    let cells_by_pos = collect_cells_by_grid_position(&mut app, &layout);
    let entity_locked = cells_by_pos[&(0, 0)];

    assert!(
        app.world().get::<RequiredToClear>(entity_locked).is_some(),
        "locked cell should have RequiredToClear"
    );
    assert!(
        app.world().get::<Locked>(entity_locked).is_some(),
        "locked cell should have Locked"
    );
}

// -- Behavior 10: Locked cell uses builder's .definition() and .override_hp() --

#[test]
fn locked_cell_hp_scaled_by_hp_mult() {
    // Given: 2x1 grid ["T","S"], locks: {(0,0): [(0,1)]}
    // "T" is Tough toughness — without ToughnessConfig, fallback = 30.0
    // "S" is Standard toughness — without ToughnessConfig, fallback = 20.0
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(0, 1)]);
    let layout = NodeLayout {
        name:            "hp_mult_locked".to_owned(),
        timer_secs:      60.0,
        cols:            2,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("T"), s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some(locks),
    };
    let mut app = test_app_with_sequence(layout.clone());
    app.update();

    let cells_by_pos = collect_cells_by_grid_position(&mut app, &layout);

    // Locked cell at (0,0) — "T" Tough toughness, fallback base = 30.0
    let entity_t = cells_by_pos[&(0, 0)];
    let health_t = app.world().get::<Hp>(entity_t).unwrap();
    assert!(
        (health_t.current - 30.0).abs() < f32::EPSILON,
        "locked cell 'T' current HP should be Tough base 30.0, got {}",
        health_t.current
    );
    assert!(
        (health_t.starting - 30.0).abs() < f32::EPSILON,
        "locked cell 'T' max HP should be Tough base 30.0, got {}",
        health_t.starting
    );

    // Locked cell should have Locked + Locks
    assert!(app.world().get::<Locked>(entity_t).is_some());
    let adj = app.world().get::<Locks>(entity_t).unwrap();
    assert_eq!(adj.0.len(), 1);
    assert_eq!(adj.0[0], cells_by_pos[&(0, 1)]);

    // Non-locked cell at (0,1) — "S" Standard toughness, fallback base = 20.0
    let entity_s = cells_by_pos[&(0, 1)];
    let health_s = app.world().get::<Hp>(entity_s).unwrap();
    assert!(
        (health_s.current - 20.0).abs() < f32::EPSILON,
        "non-locked cell 'S' current HP should be Standard base 20.0, got {}",
        health_s.current
    );
    assert!(app.world().get::<Locked>(entity_s).is_none());
}

#[test]
fn locked_cell_hp_unscaled_when_hp_mult_is_one() {
    // Edge case: locked cell "T" (Tough) — without ToughnessConfig, fallback = 30.0
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(0, 1)]);
    let layout = NodeLayout {
        name:            "hp_mult_one_locked".to_owned(),
        timer_secs:      60.0,
        cols:            2,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("T"), s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some(locks),
    };
    let mut app = test_app_with_sequence(layout.clone());
    app.update();

    let cells_by_pos = collect_cells_by_grid_position(&mut app, &layout);
    let entity_t = cells_by_pos[&(0, 0)];
    let health_t = app.world().get::<Hp>(entity_t).unwrap();
    assert!(
        (health_t.current - 30.0).abs() < f32::EPSILON,
        "locked cell 'T' HP should be Tough base 30.0, got {}",
        health_t.current
    );
    assert!(
        (health_t.starting - 30.0).abs() < f32::EPSILON,
        "locked cell 'T' max HP should be Tough base 30.0, got {}",
        health_t.starting
    );
}

// =============================================================================
// Section G: Required-to-Clear Count
// =============================================================================

// -- Behavior 13: Required-to-clear count correct with mixed locked/non-locked --

#[test]
fn required_to_clear_count_includes_locked_cells() {
    // Given: 3x1 grid ["S","T","S"], all have required_to_clear=true
    // locks: {(0,0): [(0,1)]}
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(0, 1)]);
    let layout = NodeLayout {
        name:            "rtc_mixed".to_owned(),
        timer_secs:      60.0,
        cols:            3,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("S"), s("T"), s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some(locks),
    };
    let mut app = test_app(layout);
    app.update();

    // All 3 cells should have RequiredToClear regardless of lock status
    let rtc_count = app
        .world_mut()
        .query::<(&Cell, &RequiredToClear)>()
        .iter(app.world())
        .count();
    assert_eq!(
        rtc_count, 3,
        "all 3 cells should have RequiredToClear (locked status does not affect it)"
    );
}

#[test]
fn required_to_clear_excludes_cells_with_false_flag() {
    // Edge case: locked cell with required_to_clear=false
    // We need a cell type with required_to_clear=false
    // Create a custom registry with an "N" type that has required_to_clear=false
    let layout = NodeLayout {
        name:            "rtc_false".to_owned(),
        timer_secs:      60.0,
        cols:            3,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("F"), s("S"), s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some({
            let mut locks: LockMap = HashMap::new();
            locks.insert((0, 0), vec![(0, 1)]);
            locks
        }),
    };
    let mut app = test_app_with_non_required_cell(layout);
    app.update();

    // "F" has required_to_clear=false, "S" has required_to_clear=true
    // So only 2 cells (the two "S" cells) should have RequiredToClear
    let rtc_count = app
        .world_mut()
        .query::<(&Cell, &RequiredToClear)>()
        .iter(app.world())
        .count();
    assert_eq!(
        rtc_count, 2,
        "only the 2 cells with required_to_clear=true should have RequiredToClear"
    );
}
