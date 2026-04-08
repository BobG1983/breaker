//! Tests for lock resolution behavior driven by `NodeLayout.locks`.
//!
//! These tests verify the two-pass spawn approach: Pass 1 spawns non-locked
//! cells; Pass 2 spawns locked cells with resolved entity IDs from Pass 1.

use std::collections::HashMap;

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    cells::components::*,
    state::run::node::{
        NodeLayout,
        definition::{LockMap, NodePool},
    },
};

/// Helper to reduce verbosity of String grid construction.
fn s(val: &str) -> String {
    val.to_owned()
}

// =============================================================================
// Section B: Two-Pass Spawn
// =============================================================================

// -- Behavior 1: Non-locked cells spawn normally when locks map is present --

#[test]
fn non_locked_cells_spawn_normally_when_locks_present() {
    // Given: 3x2 grid all "S", locks: Some({(0,1): [(1,0)]})
    let layout = locked_layout_3x2_single();
    let mut app = test_app(layout);
    app.update();

    // Then: 6 Cell entities exist
    let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
    assert_eq!(cell_count, 6, "all 6 cells should spawn");

    // 5 non-locked cells should NOT have Locked component
    let locked_count = app
        .world_mut()
        .query::<(&Cell, &Locked)>()
        .iter(app.world())
        .count();
    assert_eq!(
        locked_count, 1,
        "only the cell at (0,1) should have Locked component"
    );

    // The locked cell at (0,1) should have Locks with exactly 1 entity
    let lock_adjacents_count = app
        .world_mut()
        .query::<(&Cell, &Locks)>()
        .iter(app.world())
        .count();
    assert_eq!(
        lock_adjacents_count, 1,
        "only the locked cell should have Locks"
    );
}

#[test]
fn empty_locks_map_spawns_all_cells_without_locked() {
    // Edge case: locks: Some({}) (empty map) — all 6 cells spawn without Locked
    let layout = NodeLayout {
        name: "empty_locks".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S"), s("S")], vec![s("S"), s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(HashMap::new()),
    };
    let mut app = test_app(layout);
    app.update();

    let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
    assert_eq!(cell_count, 6);

    let locked_count = app
        .world_mut()
        .query::<(&Cell, &Locked)>()
        .iter(app.world())
        .count();
    assert_eq!(
        locked_count, 0,
        "empty locks map should produce no Locked cells"
    );
}

// -- Behavior 2: Non-locked cells collected into position-to-entity map --

#[test]
fn locked_cell_lock_adjacents_references_correct_entity() {
    // Given: 3x1 grid ["S","T","S"], locks: {(0,2): [(0,0)]}
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 2), vec![(0, 0)]);
    let layout = NodeLayout {
        name: "pos_map_test".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("T"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
    };
    let mut app = test_app(layout.clone());
    app.update();

    let cells_by_pos = collect_cells_by_grid_position(&mut app, &layout);

    // Cell at (0,0) should have CellTypeAlias("S") and exist
    let entity_0_0 = cells_by_pos[&(0, 0)];
    let alias_0_0 = app.world().get::<CellTypeAlias>(entity_0_0).unwrap();
    assert_eq!(alias_0_0.0, "S");

    // Cell at (0,1) should have CellTypeAlias("T")
    let entity_0_1 = cells_by_pos[&(0, 1)];
    let alias_0_1 = app.world().get::<CellTypeAlias>(entity_0_1).unwrap();
    assert_eq!(alias_0_1.0, "T");

    // Cell at (0,2) should be locked with Locks containing entity at (0,0)
    let entity_0_2 = cells_by_pos[&(0, 2)];
    let adjacents = app
        .world()
        .get::<Locks>(entity_0_2)
        .expect("locked cell at (0,2) should have Locks");
    assert_eq!(
        adjacents.0.len(),
        1,
        "Locks should contain exactly 1 entity"
    );
    assert_eq!(
        adjacents.0[0], entity_0_0,
        "Locks should reference the entity at grid position (0,0)"
    );
}

// -- Behavior 3: Locked cells receive Locked + Locks with correct entity IDs --

#[test]
fn locked_cell_has_locked_and_lock_adjacents_with_two_targets() {
    // Given: 3x2 grid all "S", locks: {(0,0): [(1,0), (1,1)]}
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(1, 0), (1, 1)]);
    let layout = NodeLayout {
        name: "two_targets".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S"), s("S")], vec![s("S"), s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
    };
    let mut app = test_app(layout.clone());
    app.update();

    let cells_by_pos = collect_cells_by_grid_position(&mut app, &layout);

    // Cell at (0,0) should have Locked component
    let entity_0_0 = cells_by_pos[&(0, 0)];
    assert!(
        app.world().get::<Locked>(entity_0_0).is_some(),
        "cell at (0,0) should have Locked component"
    );

    // Cell at (0,0) should have Locks with 2 entities
    let adjacents = app
        .world()
        .get::<Locks>(entity_0_0)
        .expect("cell at (0,0) should have Locks");
    assert_eq!(adjacents.0.len(), 2, "Locks should contain 2 entities");

    // Entities in Locks should be the cells at (1,0) and (1,1)
    let entity_1_0 = cells_by_pos[&(1, 0)];
    let entity_1_1 = cells_by_pos[&(1, 1)];
    assert!(
        adjacents.0.contains(&entity_1_0),
        "Locks should contain entity at (1,0)"
    );
    assert!(
        adjacents.0.contains(&entity_1_1),
        "Locks should contain entity at (1,1)"
    );

    // Target cells should NOT have Locked
    assert!(
        app.world().get::<Locked>(entity_1_0).is_none(),
        "cell at (1,0) should NOT have Locked"
    );
    assert!(
        app.world().get::<Locked>(entity_1_1).is_none(),
        "cell at (1,1) should NOT have Locked"
    );
}

#[test]
fn locked_cell_with_single_dependency() {
    // Edge case: locks: {(0,0): [(0,1)]} — single dependency
    let layout = simple_locked_layout();
    let mut app = test_app(layout.clone());
    app.update();

    let cells_by_pos = collect_cells_by_grid_position(&mut app, &layout);

    let entity_0_0 = cells_by_pos[&(0, 0)];
    let adjacents = app
        .world()
        .get::<Locks>(entity_0_0)
        .expect("locked cell should have Locks");
    assert_eq!(
        adjacents.0.len(),
        1,
        "Locks should contain exactly 1 entity"
    );
}

// =============================================================================
// Section C: Topological Sort — Chained Lock Dependencies
// =============================================================================

// -- Behavior 4: Simple chain A locked by B locked by C --

#[test]
fn chain_lock_resolves_in_dependency_order() {
    // Given: 3x1 grid ["S","S","S"], locks: {(0,0): [(0,1)], (0,1): [(0,2)]}
    let layout = chain_locked_layout();
    let mut app = test_app(layout.clone());
    app.update();

    let cells_by_pos = collect_cells_by_grid_position(&mut app, &layout);

    // All 3 cells should exist
    assert_eq!(cells_by_pos.len(), 3);

    let entity_a = cells_by_pos[&(0, 0)];
    let entity_b = cells_by_pos[&(0, 1)];
    let entity_c = cells_by_pos[&(0, 2)];

    // Cell C at (0,2) — no Locked (spawned in Pass 1)
    assert!(
        app.world().get::<Locked>(entity_c).is_none(),
        "cell C at (0,2) should NOT have Locked"
    );

    // Cell B at (0,1) — Locked + Locks([entity_C])
    assert!(
        app.world().get::<Locked>(entity_b).is_some(),
        "cell B at (0,1) should have Locked"
    );
    let b_adjacents = app
        .world()
        .get::<Locks>(entity_b)
        .expect("cell B should have Locks");
    assert_eq!(b_adjacents.0.len(), 1);
    assert_eq!(b_adjacents.0[0], entity_c);

    // Cell A at (0,0) — Locked + Locks([entity_B])
    assert!(
        app.world().get::<Locked>(entity_a).is_some(),
        "cell A at (0,0) should have Locked"
    );
    let a_adjacents = app
        .world()
        .get::<Locks>(entity_a)
        .expect("cell A should have Locks");
    assert_eq!(a_adjacents.0.len(), 1);
    assert_eq!(a_adjacents.0[0], entity_b);

    // entity_B in A's Locks is valid and has Locked
    assert!(
        app.world().get::<Locked>(a_adjacents.0[0]).is_some(),
        "entity_B referenced by A's Locks should have Locked component"
    );

    // entity_C in B's Locks does NOT have Locked
    assert!(
        app.world().get::<Locked>(b_adjacents.0[0]).is_none(),
        "entity_C referenced by B's Locks should NOT have Locked component"
    );
}

#[test]
fn three_deep_chain_all_cells_spawn_correctly() {
    // Edge case: A locked by B, B locked by C, C locked by D (4x1 grid)
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(0, 1)]);
    locks.insert((0, 1), vec![(0, 2)]);
    locks.insert((0, 2), vec![(0, 3)]);
    let layout = NodeLayout {
        name: "three_deep_chain".to_owned(),
        timer_secs: 60.0,
        cols: 4,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S"), s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
    };
    let mut app = test_app(layout.clone());
    app.update();

    let cells_by_pos = collect_cells_by_grid_position(&mut app, &layout);
    assert_eq!(cells_by_pos.len(), 4, "all 4 cells should spawn");

    // D at (0,3) — no Locked (spawned in Pass 1)
    assert!(app.world().get::<Locked>(cells_by_pos[&(0, 3)]).is_none());
    // C at (0,2) — Locked with Locks pointing to D
    assert!(app.world().get::<Locked>(cells_by_pos[&(0, 2)]).is_some());
    // B at (0,1) — Locked with Locks pointing to C
    assert!(app.world().get::<Locked>(cells_by_pos[&(0, 1)]).is_some());
    // A at (0,0) — Locked with Locks pointing to B
    assert!(app.world().get::<Locked>(cells_by_pos[&(0, 0)]).is_some());
}

// -- Behavior 5: Diamond dependency --

#[test]
fn diamond_lock_resolves_all_four_cells() {
    // Given: 2x2 grid all "S", diamond locks:
    // A=(0,0) locked by B=(0,1) and C=(1,0)
    // B=(0,1) locked by D=(1,1)
    // C=(1,0) locked by D=(1,1)
    let layout = diamond_locked_layout();
    let mut app = test_app(layout.clone());
    app.update();

    let cells_by_pos = collect_cells_by_grid_position(&mut app, &layout);
    assert_eq!(cells_by_pos.len(), 4, "all 4 cells should spawn");

    let entity_a = cells_by_pos[&(0, 0)];
    let entity_b = cells_by_pos[&(0, 1)];
    let entity_c = cells_by_pos[&(1, 0)];
    let entity_d = cells_by_pos[&(1, 1)];

    // D spawned in Pass 1 — no Locked
    assert!(
        app.world().get::<Locked>(entity_d).is_none(),
        "D at (1,1) should not have Locked"
    );

    // B has Locked + Locks containing entity_D
    assert!(app.world().get::<Locked>(entity_b).is_some());
    let b_adj = app.world().get::<Locks>(entity_b).unwrap();
    assert_eq!(b_adj.0.len(), 1);
    assert_eq!(b_adj.0[0], entity_d);

    // C has Locked + Locks containing entity_D
    assert!(app.world().get::<Locked>(entity_c).is_some());
    let c_adj = app.world().get::<Locks>(entity_c).unwrap();
    assert_eq!(c_adj.0.len(), 1);
    assert_eq!(c_adj.0[0], entity_d);

    // A has Locked + Locks containing entity_B and entity_C
    assert!(app.world().get::<Locked>(entity_a).is_some());
    let a_adj = app.world().get::<Locks>(entity_a).unwrap();
    assert_eq!(a_adj.0.len(), 2);
    assert!(a_adj.0.contains(&entity_b));
    assert!(a_adj.0.contains(&entity_c));

    // All have CellTypeAlias("S")
    for &pos in &[(0, 0), (0, 1), (1, 0), (1, 1)] {
        let entity = cells_by_pos[&pos];
        let alias = app.world().get::<CellTypeAlias>(entity).unwrap();
        assert_eq!(alias.0, "S", "cell at {pos:?} should have alias 'S'");
    }
}

// -- Behavior 6: Multiple independent lock groups --

#[test]
fn multiple_independent_lock_groups_resolve_correctly() {
    // Given: 4x1 grid ["S","S","T","T"], locks: {(0,0): [(0,1)], (0,2): [(0,3)]}
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(0, 1)]);
    locks.insert((0, 2), vec![(0, 3)]);
    let layout = NodeLayout {
        name: "independent_groups".to_owned(),
        timer_secs: 60.0,
        cols: 4,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S"), s("T"), s("T")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
    };
    let mut app = test_app(layout.clone());
    app.update();

    let cells_by_pos = collect_cells_by_grid_position(&mut app, &layout);
    assert_eq!(cells_by_pos.len(), 4);

    // (0,1) and (0,3) are unlocked (not in locks keys)
    assert!(
        app.world().get::<Locked>(cells_by_pos[&(0, 1)]).is_none(),
        "cell at (0,1) should NOT have Locked"
    );
    assert!(
        app.world().get::<Locked>(cells_by_pos[&(0, 3)]).is_none(),
        "cell at (0,3) should NOT have Locked"
    );

    // (0,0) locked by (0,1)
    let adj_0_0 = app
        .world()
        .get::<Locks>(cells_by_pos[&(0, 0)])
        .expect("cell at (0,0) should have Locks");
    assert_eq!(adj_0_0.0.len(), 1);
    assert_eq!(adj_0_0.0[0], cells_by_pos[&(0, 1)]);

    // (0,2) locked by (0,3)
    let adj_0_2 = app
        .world()
        .get::<Locks>(cells_by_pos[&(0, 2)])
        .expect("cell at (0,2) should have Locks");
    assert_eq!(adj_0_2.0.len(), 1);
    assert_eq!(adj_0_2.0[0], cells_by_pos[&(0, 3)]);
}

// =============================================================================
// Section D: Circular Lock Handling
// =============================================================================

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
        name: "self_ref".to_owned(),
        timer_secs: 60.0,
        cols: 1,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
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
        name: "partial_cycle".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
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

    // All cells should still have CellHealth (they spawn as unlocked fallbacks)
    let health_count = app
        .world_mut()
        .query::<(&Cell, &CellHealth)>()
        .iter(app.world())
        .count();
    assert_eq!(health_count, 3, "all cells should have CellHealth");
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
        name: "four_node_cycle".to_owned(),
        timer_secs: 60.0,
        cols: 4,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S"), s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
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

// =============================================================================
// Section E: Builder Integration with Lock Resolution
// =============================================================================

// -- Behavior 9: Locked cell's Locks entity IDs match Pass 1 entities --

#[test]
fn locked_cell_lock_adjacents_matches_pass1_entity() {
    // Given: 2x1 grid ["S","S"], locks: {(0,0): [(0,1)]}
    let layout = simple_locked_layout();
    let mut app = test_app(layout.clone());
    app.update();

    let cells_by_pos = collect_cells_by_grid_position(&mut app, &layout);

    let entity_locked = cells_by_pos[&(0, 0)];
    let entity_target = cells_by_pos[&(0, 1)];

    let adjacents = app
        .world()
        .get::<Locks>(entity_locked)
        .expect("locked cell should have Locks");
    assert_eq!(adjacents.0.len(), 1);
    assert_eq!(
        adjacents.0[0], entity_target,
        "Locks entity should match the Pass 1 entity at (0,1)"
    );

    // Verify the referenced entity has CellTypeAlias("S") and exists
    let alias = app
        .world()
        .get::<CellTypeAlias>(adjacents.0[0])
        .expect("referenced entity should exist and have CellTypeAlias");
    assert_eq!(alias.0, "S");
}

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
    // Given: 2x1 grid ["T","S"], locks: {(0,0): [(0,1)]}, hp_mult=2.0
    // "T" has hp=3.0, so scaled_hp = 3.0 * 2.0 = 6.0
    // "S" has hp=1.0, so scaled_hp = 1.0 * 2.0 = 2.0
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(0, 1)]);
    let layout = NodeLayout {
        name: "hp_mult_locked".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("T"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
    };
    let mut app = test_app_with_hp_mult(layout.clone(), 2.0);
    app.update();

    let cells_by_pos = collect_cells_by_grid_position(&mut app, &layout);

    // Locked cell at (0,0) — "T" with hp 3.0 * 2.0 = 6.0
    let entity_t = cells_by_pos[&(0, 0)];
    let health_t = app.world().get::<CellHealth>(entity_t).unwrap();
    assert!(
        (health_t.current - 6.0).abs() < f32::EPSILON,
        "locked cell 'T' current HP should be 6.0, got {}",
        health_t.current
    );
    assert!(
        (health_t.max - 6.0).abs() < f32::EPSILON,
        "locked cell 'T' max HP should be 6.0, got {}",
        health_t.max
    );

    // Locked cell should have Locked + Locks
    assert!(app.world().get::<Locked>(entity_t).is_some());
    let adj = app.world().get::<Locks>(entity_t).unwrap();
    assert_eq!(adj.0.len(), 1);
    assert_eq!(adj.0[0], cells_by_pos[&(0, 1)]);

    // Non-locked cell at (0,1) — "S" with hp 1.0 * 2.0 = 2.0
    let entity_s = cells_by_pos[&(0, 1)];
    let health_s = app.world().get::<CellHealth>(entity_s).unwrap();
    assert!(
        (health_s.current - 2.0).abs() < f32::EPSILON,
        "non-locked cell 'S' current HP should be 2.0, got {}",
        health_s.current
    );
    assert!(app.world().get::<Locked>(entity_s).is_none());
}

#[test]
fn locked_cell_hp_unscaled_when_hp_mult_is_one() {
    // Edge case: hp_mult = 1.0 — locked cell "T" has CellHealth { current: 3.0, max: 3.0 }
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(0, 1)]);
    let layout = NodeLayout {
        name: "hp_mult_one_locked".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("T"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
    };
    let mut app = test_app_with_hp_mult(layout.clone(), 1.0);
    app.update();

    let cells_by_pos = collect_cells_by_grid_position(&mut app, &layout);
    let entity_t = cells_by_pos[&(0, 0)];
    let health_t = app.world().get::<CellHealth>(entity_t).unwrap();
    assert!(
        (health_t.current - 3.0).abs() < f32::EPSILON,
        "locked cell 'T' HP should be 3.0, got {}",
        health_t.current
    );
    assert!(
        (health_t.max - 3.0).abs() < f32::EPSILON,
        "locked cell 'T' max HP should be 3.0, got {}",
        health_t.max
    );
}

// =============================================================================
// Section F: Shielded Cells Remain Unchanged
// =============================================================================

// -- Behavior 11: Shielded cells use existing orbit path --
// (Existing tests in shield_cells/ cover this fully.)

// -- Behavior 12: Shielded cell NOT affected by locks map --

#[test]
fn shielded_cell_not_affected_by_locks_map() {
    // Given: 3x1 grid ["H","S","S"], "H" has shield config
    // locks: {(0,1): [(0,2)]} — only (0,1) is in locks, NOT "H"
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 1), vec![(0, 2)]);
    let layout = NodeLayout {
        name: "shield_with_locks".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("H"), s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
    };
    let mut app = test_app_with_shield_registry(layout);
    app.update();

    // "H" at (0,0) should have ShieldParent + Locked + Locks (from orbit path)
    let shield_parents: Vec<(Entity, &Locks)> = app
        .world_mut()
        .query_filtered::<(Entity, &Locks), With<ShieldParent>>()
        .iter(app.world())
        .collect();
    assert_eq!(
        shield_parents.len(),
        1,
        "shield cell should have ShieldParent + Locks"
    );
    // Shield cell's Locks should contain orbit entities (3 of them)
    let (shield_entity, shield_adj) = shield_parents[0];
    assert_eq!(
        shield_adj.0.len(),
        3,
        "shield cell Locks should have 3 orbit entities, not lock targets"
    );
    // Verify orbit entities are OrbitCell, not regular cells
    for &orbit_entity in &shield_adj.0 {
        assert!(
            app.world().get::<OrbitCell>(orbit_entity).is_some(),
            "shield Locks should reference OrbitCell entities"
        );
    }

    // Verify shield entity has Locked (from shield path)
    assert!(
        app.world().get::<Locked>(shield_entity).is_some(),
        "shield cell should have Locked from orbit path"
    );

    // Cell at (0,1) should have Locked + Locks from lock resolution path
    // We need to find the non-shield locked cell
    let non_shield_locked: Vec<(Entity, &Locks)> = app
        .world_mut()
        .query_filtered::<(Entity, &Locks), (With<Locked>, Without<ShieldParent>)>()
        .iter(app.world())
        .collect();
    assert_eq!(
        non_shield_locked.len(),
        1,
        "cell at (0,1) should be the only non-shield locked cell"
    );

    // Cell at (0,2) should NOT have Locked
    // Count total cells with Locked: shield (1) + lock-resolved (1) = 2
    let total_locked = app
        .world_mut()
        .query::<(&Cell, &Locked)>()
        .iter(app.world())
        .count();
    // Shield parent cell counts as Locked too (from orbit path)
    // So: "H" locked (shield), "S" at (0,1) locked (from locks map) = 2 total
    assert_eq!(
        total_locked, 2,
        "should have 2 locked cells: shield + lock-resolved"
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
        name: "rtc_mixed".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("T"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
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
        name: "rtc_false".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("F"), s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some({
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

// =============================================================================
// Section H: Edge Cases for Lock Map
// =============================================================================

// -- Behavior 14: Lock target points to "." (empty) grid position --

#[test]
fn lock_target_pointing_to_empty_spawns_without_locked() {
    // Given: 3x1 grid ["S",".","S"], locks: {(0,0): [(0,1)]}
    // (0,1) is "." — no entity spawned there
    // Construct directly (no validate()) to bypass validation
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(0, 1)]);
    let layout = NodeLayout {
        name: "lock_target_empty".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("."), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
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

    // Cell at (0,0) should still have CellHealth
    let health_count = app
        .world_mut()
        .query::<(&Cell, &CellHealth)>()
        .iter(app.world())
        .count();
    assert_eq!(health_count, 2, "all spawned cells should have CellHealth");
}

// -- Behavior 15: Lock target points to out-of-bounds grid position --

#[test]
fn lock_target_out_of_bounds_spawns_without_locked() {
    // Given: 2x1 grid ["S","S"], locks: {(0,0): [(5,5)]} — (5,5) out of bounds
    // Construct directly (no validate()) to bypass validation
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(5, 5)]);
    let layout = NodeLayout {
        name: "lock_target_oob".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
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
        name: "lock_key_oob".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
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
        name: "empty_grid_with_locks".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("."), s(".")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
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
        name: "empty_both".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("."), s(".")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(HashMap::new()),
    };
    let mut app = test_app(layout);
    app.update();

    let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
    assert_eq!(cell_count, 0, "no cells should spawn");
}
