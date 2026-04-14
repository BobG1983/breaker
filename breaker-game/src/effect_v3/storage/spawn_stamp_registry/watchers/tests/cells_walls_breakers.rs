//! Behaviors 12-14 — per-kind sanity checks for cells, walls, breakers.
//!
//! Each watcher is exercised in isolation (only its own system registered)
//! to confirm that it filters on its marker component and not the others.

use bevy::prelude::*;

use super::helpers::{
    breaker_only_test_app, cell_only_test_app, set_registry, speed_boost_tree, wall_only_test_app,
};
use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    cells::components::Cell,
    effect_v3::{storage::BoundEffects, types::EntityKind},
    shared::test_utils::tick,
    walls::components::Wall,
};

// ── Behavior 12: cells watcher ─────────────────────────────────────────────

#[test]
fn cell_watcher_stamps_only_cell_marked_entities() {
    let mut app = cell_only_test_app();
    set_registry(
        &mut app,
        vec![(
            EntityKind::Cell,
            "cell_chip".to_string(),
            speed_boost_tree(1.5),
        )],
    );

    let bolt = app.world_mut().spawn(Bolt).id();
    let cell = app.world_mut().spawn(Cell).id();
    let wall = app.world_mut().spawn(Wall).id();
    let breaker = app.world_mut().spawn(Breaker).id();
    tick(&mut app);

    let cell_bound = app
        .world()
        .get::<BoundEffects>(cell)
        .expect("cell should have BoundEffects");
    assert_eq!(cell_bound.0.len(), 1);
    assert_eq!(cell_bound.0[0].0, "cell_chip");
    assert_eq!(cell_bound.0[0].1, speed_boost_tree(1.5));

    assert!(app.world().get::<BoundEffects>(bolt).is_none());
    assert!(app.world().get::<BoundEffects>(wall).is_none());
    assert!(app.world().get::<BoundEffects>(breaker).is_none());
}

#[test]
fn cell_watcher_stamps_two_cells_independently() {
    let mut app = cell_only_test_app();
    set_registry(
        &mut app,
        vec![(
            EntityKind::Cell,
            "cell_chip".to_string(),
            speed_boost_tree(1.5),
        )],
    );

    let c1 = app.world_mut().spawn(Cell).id();
    let c2 = app.world_mut().spawn(Cell).id();
    tick(&mut app);

    for cell in [c1, c2] {
        let bound = app
            .world()
            .get::<BoundEffects>(cell)
            .expect("each cell should have BoundEffects");
        assert_eq!(bound.0.len(), 1);
        assert_eq!(bound.0[0].0, "cell_chip");
    }
}

// ── Behavior 13: walls watcher ─────────────────────────────────────────────

#[test]
fn wall_watcher_stamps_only_wall_marked_entities() {
    let mut app = wall_only_test_app();
    set_registry(
        &mut app,
        vec![(
            EntityKind::Wall,
            "wall_chip".to_string(),
            speed_boost_tree(1.5),
        )],
    );

    let bolt = app.world_mut().spawn(Bolt).id();
    let cell = app.world_mut().spawn(Cell).id();
    let wall = app.world_mut().spawn(Wall).id();
    let breaker = app.world_mut().spawn(Breaker).id();
    tick(&mut app);

    let wall_bound = app
        .world()
        .get::<BoundEffects>(wall)
        .expect("wall should have BoundEffects");
    assert_eq!(wall_bound.0.len(), 1);
    assert_eq!(wall_bound.0[0].0, "wall_chip");
    assert_eq!(wall_bound.0[0].1, speed_boost_tree(1.5));

    assert!(app.world().get::<BoundEffects>(bolt).is_none());
    assert!(app.world().get::<BoundEffects>(cell).is_none());
    assert!(app.world().get::<BoundEffects>(breaker).is_none());
}

#[test]
fn wall_watcher_stamps_three_walls_independently() {
    let mut app = wall_only_test_app();
    set_registry(
        &mut app,
        vec![(
            EntityKind::Wall,
            "wall_chip".to_string(),
            speed_boost_tree(1.5),
        )],
    );

    let left = app.world_mut().spawn(Wall).id();
    let right = app.world_mut().spawn(Wall).id();
    let ceiling = app.world_mut().spawn(Wall).id();
    tick(&mut app);

    for wall in [left, right, ceiling] {
        let bound = app
            .world()
            .get::<BoundEffects>(wall)
            .expect("each wall should have BoundEffects");
        assert_eq!(bound.0.len(), 1);
        assert_eq!(bound.0[0].0, "wall_chip");
    }
}

// ── Behavior 14: breakers watcher ──────────────────────────────────────────

#[test]
fn breaker_watcher_stamps_only_breaker_marked_entities() {
    let mut app = breaker_only_test_app();
    set_registry(
        &mut app,
        vec![(
            EntityKind::Breaker,
            "breaker_chip".to_string(),
            speed_boost_tree(1.5),
        )],
    );

    let bolt = app.world_mut().spawn(Bolt).id();
    let cell = app.world_mut().spawn(Cell).id();
    let wall = app.world_mut().spawn(Wall).id();
    let breaker = app.world_mut().spawn(Breaker).id();
    tick(&mut app);

    let breaker_bound = app
        .world()
        .get::<BoundEffects>(breaker)
        .expect("breaker should have BoundEffects");
    assert_eq!(breaker_bound.0.len(), 1);
    assert_eq!(breaker_bound.0[0].0, "breaker_chip");
    assert_eq!(breaker_bound.0[0].1, speed_boost_tree(1.5));

    assert!(app.world().get::<BoundEffects>(bolt).is_none());
    assert!(app.world().get::<BoundEffects>(cell).is_none());
    assert!(app.world().get::<BoundEffects>(wall).is_none());
}

#[test]
fn breaker_watcher_stamps_two_breakers_independently() {
    let mut app = breaker_only_test_app();
    set_registry(
        &mut app,
        vec![(
            EntityKind::Breaker,
            "breaker_chip".to_string(),
            speed_boost_tree(1.5),
        )],
    );

    let primary = app.world_mut().spawn(Breaker).id();
    let extra = app.world_mut().spawn(Breaker).id();
    tick(&mut app);

    for breaker in [primary, extra] {
        let bound = app
            .world()
            .get::<BoundEffects>(breaker)
            .expect("each breaker should have BoundEffects");
        assert_eq!(bound.0.len(), 1);
        assert_eq!(bound.0[0].0, "breaker_chip");
    }
}
