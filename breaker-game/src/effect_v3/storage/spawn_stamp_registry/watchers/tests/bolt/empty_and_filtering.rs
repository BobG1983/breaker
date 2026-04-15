//! Behaviors 3-4: empty registry no-op and non-matching `EntityKind` filtering.

use bevy::prelude::*;

use super::super::helpers::{bolt_only_test_app, set_registry, speed_boost_tree};
use crate::{
    bolt::components::Bolt,
    effect_v3::{storage::BoundEffects, types::EntityKind},
    shared::test_utils::tick,
};

// ── Behavior 3: empty registry is a no-op ──────────────────────────────────

#[test]
fn bolt_watcher_is_noop_when_registry_is_empty() {
    let mut app = bolt_only_test_app();
    // Registry starts empty (Default).

    let first = app.world_mut().spawn(Bolt).id();
    tick(&mut app);

    assert!(
        app.world().get::<BoundEffects>(first).is_none(),
        "empty registry must not insert BoundEffects"
    );

    // Positive anchor: after populating the registry, a NEWLY spawned bolt
    // must be stamped. This distinguishes the real implementation from a
    // no-op stub — against the stub this assertion fails, giving RED signal.
    set_registry(
        &mut app,
        vec![(
            EntityKind::Bolt,
            "chip_a".to_string(),
            speed_boost_tree(1.5),
        )],
    );
    let second = app.world_mut().spawn(Bolt).id();
    tick(&mut app);

    let bound = app
        .world()
        .get::<BoundEffects>(second)
        .expect("bolt spawned after registry populated must be stamped");
    assert_eq!(bound.0.len(), 1);
    assert_eq!(bound.0[0].0, "chip_a");
    assert_eq!(bound.0[0].1, speed_boost_tree(1.5));

    // The first bolt, spawned when the registry was empty, must still not
    // have been stamped.
    assert!(
        app.world().get::<BoundEffects>(first).is_none(),
        "first bolt (spawned pre-populate) must remain unstamped"
    );
}

#[test]
fn bolt_watcher_is_noop_when_registry_is_empty_across_two_ticks() {
    let mut app = bolt_only_test_app();

    let first = app.world_mut().spawn(Bolt).id();
    tick(&mut app);
    tick(&mut app);

    assert!(
        app.world().get::<BoundEffects>(first).is_none(),
        "empty registry must remain a no-op after a second tick"
    );

    // Positive anchor: populate the registry, spawn a new bolt, tick twice,
    // and confirm the new bolt is stamped exactly once. Against the stub
    // this positive assertion fails, giving the test real RED signal.
    set_registry(
        &mut app,
        vec![(
            EntityKind::Bolt,
            "chip_a".to_string(),
            speed_boost_tree(1.5),
        )],
    );
    let second = app.world_mut().spawn(Bolt).id();
    tick(&mut app);
    tick(&mut app);

    let bound = app
        .world()
        .get::<BoundEffects>(second)
        .expect("bolt spawned after registry populated must be stamped");
    assert_eq!(bound.0.len(), 1);
    assert_eq!(bound.0[0].0, "chip_a");
    assert_eq!(bound.0[0].1, speed_boost_tree(1.5));

    // The first bolt, spawned when the registry was empty, must still not
    // have been stamped even after four ticks total.
    assert!(
        app.world().get::<BoundEffects>(first).is_none(),
        "first bolt (spawned pre-populate) must remain unstamped across ticks"
    );
}

// ── Behavior 4: non-matching EntityKind is ignored ─────────────────────────

#[test]
fn bolt_watcher_ignores_cell_kind_entries() {
    let mut app = bolt_only_test_app();
    set_registry(
        &mut app,
        vec![
            (
                EntityKind::Cell,
                "cell_chip".to_string(),
                speed_boost_tree(1.5),
            ),
            (
                EntityKind::Bolt,
                "bolt_chip".to_string(),
                speed_boost_tree(2.0),
            ),
        ],
    );

    let entity = app.world_mut().spawn(Bolt).id();
    tick(&mut app);

    let bound = app
        .world()
        .get::<BoundEffects>(entity)
        .expect("bolt must receive the Bolt-kind entry");
    assert_eq!(
        bound.0.len(),
        1,
        "bolt watcher must apply ONLY the Bolt-kind entry, not the Cell-kind entry"
    );
    assert_eq!(bound.0[0].0, "bolt_chip");
    assert_eq!(bound.0[0].1, speed_boost_tree(2.0));
    assert!(
        bound.0.iter().all(|(name, _)| name != "cell_chip"),
        "bolt watcher must not stamp entries of kind Cell"
    );
}

#[test]
fn bolt_watcher_ignores_wall_kind_entries() {
    let mut app = bolt_only_test_app();
    set_registry(
        &mut app,
        vec![
            (
                EntityKind::Wall,
                "wall_chip".to_string(),
                speed_boost_tree(1.5),
            ),
            (
                EntityKind::Bolt,
                "bolt_chip".to_string(),
                speed_boost_tree(2.0),
            ),
        ],
    );

    let entity = app.world_mut().spawn(Bolt).id();
    tick(&mut app);

    let bound = app
        .world()
        .get::<BoundEffects>(entity)
        .expect("bolt must receive the Bolt-kind entry");
    assert_eq!(
        bound.0.len(),
        1,
        "bolt watcher must apply ONLY the Bolt-kind entry, not the Wall-kind entry"
    );
    assert_eq!(bound.0[0].0, "bolt_chip");
    assert_eq!(bound.0[0].1, speed_boost_tree(2.0));
    assert!(
        bound.0.iter().all(|(name, _)| name != "wall_chip"),
        "bolt watcher must not stamp entries of kind Wall"
    );
}

#[test]
fn bolt_watcher_ignores_breaker_kind_entries() {
    let mut app = bolt_only_test_app();
    set_registry(
        &mut app,
        vec![
            (
                EntityKind::Breaker,
                "breaker_chip".to_string(),
                speed_boost_tree(1.5),
            ),
            (
                EntityKind::Bolt,
                "bolt_chip".to_string(),
                speed_boost_tree(2.0),
            ),
        ],
    );

    let entity = app.world_mut().spawn(Bolt).id();
    tick(&mut app);

    let bound = app
        .world()
        .get::<BoundEffects>(entity)
        .expect("bolt must receive the Bolt-kind entry");
    assert_eq!(
        bound.0.len(),
        1,
        "bolt watcher must apply ONLY the Bolt-kind entry, not the Breaker-kind entry"
    );
    assert_eq!(bound.0[0].0, "bolt_chip");
    assert_eq!(bound.0[0].1, speed_boost_tree(2.0));
    assert!(
        bound.0.iter().all(|(name, _)| name != "breaker_chip"),
        "bolt watcher must not stamp entries of kind Breaker"
    );
}

#[test]
fn bolt_watcher_ignores_any_kind_entries() {
    let mut app = bolt_only_test_app();
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

    let entity = app.world_mut().spawn(Bolt).id();
    tick(&mut app);

    let bound = app
        .world()
        .get::<BoundEffects>(entity)
        .expect("bolt must receive the Bolt-kind entry");
    assert_eq!(
        bound.0.len(),
        1,
        "bolt watcher must apply ONLY the Bolt-kind entry, not the Any-kind entry"
    );
    assert_eq!(bound.0[0].0, "bolt_chip");
    assert_eq!(bound.0[0].1, speed_boost_tree(2.0));
    assert!(
        bound.0.iter().all(|(name, _)| name != "any_chip"),
        "bolt watcher must not stamp entries of kind Any (out of scope for this wave)"
    );
}
