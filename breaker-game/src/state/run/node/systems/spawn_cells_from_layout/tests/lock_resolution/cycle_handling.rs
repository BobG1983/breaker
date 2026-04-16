//! Circular lock detection tests (Section D)

use std::collections::HashMap;

use bevy::prelude::*;

use super::{super::helpers::*, helpers::*};
use crate::{
    cells::components::*,
    prelude::*,
    state::run::node::definition::{LockMap, NodePool},
};

// -- Behavior 7: Direct circular lock A<->B --

#[test]
fn direct_circular_lock_drops_edges_both_spawn_unlocked() {
    // Given: 2x1 grid ["S","S"], locks: {(0,0): [(0,1)], (0,1): [(0,0)]}
    let layout = circular_locked_layout();
    let mut app = test_app(layout);
    app.update();

    let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
    assert_eq!(cell_count, 2, "both cells should still spawn");

    // Neither cell should have Locked or Locks (circular edges dropped)
    let locked_count = app
        .world_mut()
        .query::<(&Cell, &Locked)>()
        .iter(app.world())
        .count();
    assert_eq!(
        locked_count, 0,
        "circular lock edges should be dropped — no Locked components"
    );

    let lock_adj_count = app
        .world_mut()
        .query::<(&Cell, &Locks)>()
        .iter(app.world())
        .count();
    assert_eq!(
        lock_adj_count, 0,
        "circular lock edges should be dropped — no Locks components"
    );
}

#[test]
fn self_referencing_lock_spawns_without_locked() {
    // Edge case: self-reference {(0,0): [(0,0)]}
    // Normally caught by validate(), but if it reaches spawn, cell spawns without Locked
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(0, 0)]);
    let layout = NodeLayout {
        name:            "self_ref".to_owned(),
        timer_secs:      60.0,
        cols:            1,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some(locks),
        sequences:       None,
    };
    let mut app = test_app(layout);
    app.update();

    let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
    assert_eq!(cell_count, 1);

    let locked_count = app
        .world_mut()
        .query::<(&Cell, &Locked)>()
        .iter(app.world())
        .count();
    assert_eq!(
        locked_count, 0,
        "self-referencing lock should be skipped, cell spawns without Locked"
    );
}

// -- Behavior 8: Partial circular lock in chain --

#[test]
fn partial_circular_chain_all_spawn_unlocked() {
    // Given: 3x1 grid ["S","S","S"], locks: {(0,0): [(0,1)], (0,1): [(0,2)], (0,2): [(0,1)]}
    // B and C form a cycle. A depends on B which is in the cycle.
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(0, 1)]);
    locks.insert((0, 1), vec![(0, 2)]);
    locks.insert((0, 2), vec![(0, 1)]);
    let layout = NodeLayout {
        name:            "partial_cycle".to_owned(),
        timer_secs:      60.0,
        cols:            3,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("S"), s("S"), s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some(locks),
        sequences:       None,
    };
    let mut app = test_app(layout);
    app.update();

    let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
    assert_eq!(cell_count, 3, "all 3 cells should spawn");

    // All three cells should have no Locked (cycle detected, fallback to unlocked)
    let locked_count = app
        .world_mut()
        .query::<(&Cell, &Locked)>()
        .iter(app.world())
        .count();
    assert_eq!(
        locked_count, 0,
        "all cells in or depending on cycle should spawn without Locked"
    );

    // All cells should still have Hp (they spawn as unlocked fallbacks)
    let health_count = app
        .world_mut()
        .query::<(&Cell, &Hp)>()
        .iter(app.world())
        .count();
    assert_eq!(health_count, 3, "all cells should have Hp");
}

#[test]
fn large_cycle_four_nodes_all_spawn_without_locks() {
    // Edge case: 4-node cycle: {(0,0):[(0,1)], (0,1):[(0,2)], (0,2):[(0,3)], (0,3):[(0,0)]}
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(0, 1)]);
    locks.insert((0, 1), vec![(0, 2)]);
    locks.insert((0, 2), vec![(0, 3)]);
    locks.insert((0, 3), vec![(0, 0)]);
    let layout = NodeLayout {
        name:            "four_node_cycle".to_owned(),
        timer_secs:      60.0,
        cols:            4,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("S"), s("S"), s("S"), s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some(locks),
        sequences:       None,
    };
    let mut app = test_app(layout);
    app.update();

    let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
    assert_eq!(cell_count, 4, "all 4 cells should spawn");

    let locked_count = app
        .world_mut()
        .query::<(&Cell, &Locked)>()
        .iter(app.world())
        .count();
    assert_eq!(
        locked_count, 0,
        "all cells in the 4-node cycle should spawn without Locked"
    );
}
