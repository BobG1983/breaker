//! Basic lock spawn tests (Section B: Two-Pass Spawn, Section E: Builder Integration)

use std::collections::HashMap;

use bevy::prelude::*;

use super::{super::helpers::*, helpers::*};
use crate::{
    cells::components::*,
    state::run::node::{
        NodeLayout,
        definition::{LockMap, NodePool},
    },
};

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
        name:            "empty_locks".to_owned(),
        timer_secs:      60.0,
        cols:            3,
        rows:            2,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("S"), s("S"), s("S")], vec![s("S"), s("S"), s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some(HashMap::new()),
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
        name:            "pos_map_test".to_owned(),
        timer_secs:      60.0,
        cols:            3,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("S"), s("T"), s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some(locks),
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
        name:            "two_targets".to_owned(),
        timer_secs:      60.0,
        cols:            3,
        rows:            2,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("S"), s("S"), s("S")], vec![s("S"), s("S"), s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some(locks),
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
