//! Chain/graph dependency resolution (Section C: Topological Sort)

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
