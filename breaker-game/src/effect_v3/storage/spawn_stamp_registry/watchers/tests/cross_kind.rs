//! Behaviors 15-16 — all four watchers registered together, verifying
//! kind-isolation and `EntityKind::Any` handling.

use bevy::prelude::*;

use super::helpers::{set_registry, speed_boost_tree, watcher_test_app};
use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    cells::components::Cell,
    effect_v3::{storage::BoundEffects, types::EntityKind},
    shared::test_utils::tick,
    walls::components::Wall,
};

// ── Behavior 15: all four watchers registered — only matching kinds ────────

#[test]
fn all_watchers_stamp_only_matching_kinds() {
    let mut app = watcher_test_app();
    set_registry(
        &mut app,
        vec![
            (
                EntityKind::Bolt,
                "bolt_chip".to_string(),
                speed_boost_tree(1.1),
            ),
            (
                EntityKind::Cell,
                "cell_chip".to_string(),
                speed_boost_tree(1.2),
            ),
            (
                EntityKind::Wall,
                "wall_chip".to_string(),
                speed_boost_tree(1.3),
            ),
            (
                EntityKind::Breaker,
                "breaker_chip".to_string(),
                speed_boost_tree(1.4),
            ),
        ],
    );

    let bolt = app.world_mut().spawn(Bolt).id();
    let cell = app.world_mut().spawn(Cell).id();
    let wall = app.world_mut().spawn(Wall).id();
    let breaker = app.world_mut().spawn(Breaker).id();
    tick(&mut app);

    let bolt_bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt BoundEffects");
    assert_eq!(bolt_bound.0.len(), 1);
    assert_eq!(bolt_bound.0[0].0, "bolt_chip");
    assert_eq!(bolt_bound.0[0].1, speed_boost_tree(1.1));

    let cell_bound = app
        .world()
        .get::<BoundEffects>(cell)
        .expect("cell BoundEffects");
    assert_eq!(cell_bound.0.len(), 1);
    assert_eq!(cell_bound.0[0].0, "cell_chip");
    assert_eq!(cell_bound.0[0].1, speed_boost_tree(1.2));

    let wall_bound = app
        .world()
        .get::<BoundEffects>(wall)
        .expect("wall BoundEffects");
    assert_eq!(wall_bound.0.len(), 1);
    assert_eq!(wall_bound.0[0].0, "wall_chip");
    assert_eq!(wall_bound.0[0].1, speed_boost_tree(1.3));

    let breaker_bound = app
        .world()
        .get::<BoundEffects>(breaker)
        .expect("breaker BoundEffects");
    assert_eq!(breaker_bound.0.len(), 1);
    assert_eq!(breaker_bound.0[0].0, "breaker_chip");
    assert_eq!(breaker_bound.0[0].1, speed_boost_tree(1.4));
}

#[test]
fn entity_with_two_kind_markers_receives_both_stamps_without_panic() {
    let mut app = watcher_test_app();
    set_registry(
        &mut app,
        vec![
            (
                EntityKind::Bolt,
                "bolt_chip".to_string(),
                speed_boost_tree(1.1),
            ),
            (
                EntityKind::Cell,
                "cell_chip".to_string(),
                speed_boost_tree(1.2),
            ),
        ],
    );

    // Pathological: entity carries both Bolt and Cell markers. Game code
    // never does this but the watchers must not panic.
    let entity = app.world_mut().spawn((Bolt, Cell)).id();
    tick(&mut app);

    let bound = app
        .world()
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist on dual-marker entity");
    assert_eq!(
        bound.0.len(),
        2,
        "entity with two kind markers must receive both stamps"
    );
    // Order between the two watchers is not asserted.
    let names: Vec<&str> = bound.0.iter().map(|(n, _)| n.as_str()).collect();
    assert!(names.contains(&"bolt_chip"));
    assert!(names.contains(&"cell_chip"));
}

// ── Behavior 16: EntityKind::Any is ignored by all watchers ───────────────

#[test]
fn any_kind_entry_is_ignored_by_all_watchers() {
    let mut app = watcher_test_app();
    // Mixed registry: an Any-kind entry AND a Bolt-kind entry. The Bolt
    // entry provides a positive anchor — the bolt entity MUST be stamped
    // with bolt_chip (fails against the stub, giving RED signal). The
    // Any-kind entry must be ignored across all four watchers.
    set_registry(
        &mut app,
        vec![
            (
                EntityKind::Any,
                "any_chip".to_string(),
                speed_boost_tree(1.5),
            ),
            (
                EntityKind::Bolt,
                "bolt_chip".to_string(),
                speed_boost_tree(2.0),
            ),
        ],
    );

    let bolt = app.world_mut().spawn(Bolt).id();
    let cell = app.world_mut().spawn(Cell).id();
    let wall = app.world_mut().spawn(Wall).id();
    let breaker = app.world_mut().spawn(Breaker).id();
    tick(&mut app);

    // Positive anchor: Bolt receives the Bolt-kind entry only.
    let bolt_bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt must receive the Bolt-kind entry");
    assert_eq!(
        bolt_bound.0.len(),
        1,
        "bolt must receive ONLY the Bolt-kind entry, not the Any-kind entry"
    );
    assert_eq!(bolt_bound.0[0].0, "bolt_chip");
    assert_eq!(bolt_bound.0[0].1, speed_boost_tree(2.0));
    assert!(
        bolt_bound.0.iter().all(|(name, _)| name != "any_chip"),
        "EntityKind::Any must not stamp onto Bolt entities"
    );

    // Cell/Wall/Breaker have no matching kind-specific entry, so they must
    // remain unstamped — the Any-kind entry must not apply to them.
    assert!(
        app.world().get::<BoundEffects>(cell).is_none(),
        "EntityKind::Any must not stamp onto Cell entities"
    );
    assert!(
        app.world().get::<BoundEffects>(wall).is_none(),
        "EntityKind::Any must not stamp onto Wall entities"
    );
    assert!(
        app.world().get::<BoundEffects>(breaker).is_none(),
        "EntityKind::Any must not stamp onto Breaker entities"
    );
}

#[test]
fn mixed_any_and_specific_entries_only_specific_is_applied() {
    let mut app = watcher_test_app();
    set_registry(
        &mut app,
        vec![
            (
                EntityKind::Any,
                "any_chip".to_string(),
                speed_boost_tree(1.5),
            ),
            (
                EntityKind::Bolt,
                "bolt_chip".to_string(),
                speed_boost_tree(2.0),
            ),
        ],
    );

    let bolt = app.world_mut().spawn(Bolt).id();
    tick(&mut app);

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects");
    assert_eq!(
        bound.0.len(),
        1,
        "only the EntityKind::Bolt entry should be stamped; Any is ignored"
    );
    assert_eq!(bound.0[0].0, "bolt_chip");
    assert_eq!(bound.0[0].1, speed_boost_tree(2.0));
}
