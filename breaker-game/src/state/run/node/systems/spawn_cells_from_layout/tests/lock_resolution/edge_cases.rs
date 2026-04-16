//! Edge cases for lock map (Section H)

use std::collections::HashMap;

use bevy::prelude::*;

use super::{super::helpers::*, helpers::*};
use crate::{
    cells::components::*,
    prelude::*,
    state::run::node::definition::{LockMap, NodePool},
};

// -- Behavior 14: Lock target points to "." (empty) grid position --

#[test]
fn lock_target_pointing_to_empty_spawns_without_locked() {
    // Given: 3x1 grid ["S",".","S"], locks: {(0,0): [(0,1)]}
    // (0,1) is "." — no entity spawned there
    // Construct directly (no validate()) to bypass validation
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(0, 1)]);
    let layout = NodeLayout {
        name:            "lock_target_empty".to_owned(),
        timer_secs:      60.0,
        cols:            3,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("S"), s("."), s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some(locks),
        sequences:       None,
    };
    let mut app = test_app(layout);
    app.update();

    // Cell at (0,0) should spawn without Locked (graceful degradation)
    let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
    assert_eq!(cell_count, 2, "only non-dot cells should spawn");

    let locked_count = app
        .world_mut()
        .query::<(&Cell, &Locked)>()
        .iter(app.world())
        .count();
    assert_eq!(
        locked_count, 0,
        "cell should spawn without Locked when lock target is empty"
    );

    // Cell at (0,0) should still have Hp
    let health_count = app
        .world_mut()
        .query::<(&Cell, &Hp)>()
        .iter(app.world())
        .count();
    assert_eq!(health_count, 2, "all spawned cells should have Hp");
}

// -- Behavior 15: Lock target points to out-of-bounds grid position --

#[test]
fn lock_target_out_of_bounds_spawns_without_locked() {
    // Given: 2x1 grid ["S","S"], locks: {(0,0): [(5,5)]} — (5,5) out of bounds
    // Construct directly (no validate()) to bypass validation
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(5, 5)]);
    let layout = NodeLayout {
        name:            "lock_target_oob".to_owned(),
        timer_secs:      60.0,
        cols:            2,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("S"), s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some(locks),
        sequences:       None,
    };
    let mut app = test_app(layout);
    app.update();

    let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
    assert_eq!(cell_count, 2, "both cells should spawn");

    let locked_count = app
        .world_mut()
        .query::<(&Cell, &Locked)>()
        .iter(app.world())
        .count();
    assert_eq!(
        locked_count, 0,
        "cell should spawn without Locked when lock target is out of bounds"
    );
}

#[test]
fn lock_key_out_of_bounds_is_ignored() {
    // Edge case: lock key itself is out of bounds — spawn system ignores it
    let mut locks: LockMap = HashMap::new();
    locks.insert((5, 5), vec![(0, 0)]);
    let layout = NodeLayout {
        name:            "lock_key_oob".to_owned(),
        timer_secs:      60.0,
        cols:            2,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("S"), s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some(locks),
        sequences:       None,
    };
    let mut app = test_app(layout);
    app.update();

    let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
    assert_eq!(cell_count, 2, "both cells should spawn normally");

    let locked_count = app
        .world_mut()
        .query::<(&Cell, &Locked)>()
        .iter(app.world())
        .count();
    assert_eq!(
        locked_count, 0,
        "no cells should be locked when the lock key is out of bounds"
    );
}

// -- Behavior 16: Empty grid with locks map --

#[test]
fn empty_grid_with_locks_map_spawns_no_cells() {
    // Given: 2x1 grid [[".","."], locks: {(0,0): [(0,1)]}
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(0, 1)]);
    let layout = NodeLayout {
        name:            "empty_grid_with_locks".to_owned(),
        timer_secs:      60.0,
        cols:            2,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("."), s(".")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some(locks),
        sequences:       None,
    };
    let mut app = test_app(layout);
    app.update();

    let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
    assert_eq!(cell_count, 0, "no cells should spawn in all-dot grid");
}

#[test]
fn empty_locks_with_all_dot_grid_spawns_no_cells() {
    // Edge case: locks: Some({}) with all-dot grid
    let layout = NodeLayout {
        name:            "empty_both".to_owned(),
        timer_secs:      60.0,
        cols:            2,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("."), s(".")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some(HashMap::new()),
        sequences:       None,
    };
    let mut app = test_app(layout);
    app.update();

    let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
    assert_eq!(cell_count, 0, "no cells should spawn");
}
