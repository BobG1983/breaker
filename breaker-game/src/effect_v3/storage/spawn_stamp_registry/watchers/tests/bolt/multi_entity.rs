//! Behaviors 5-7: multiple entries, multiple entities, and cartesian stamping.

use bevy::prelude::*;

use super::super::helpers::{bolt_only_test_app, set_registry, speed_boost_tree};
use crate::{
    bolt::components::Bolt,
    effect_v3::{
        storage::BoundEffects,
        types::{EntityKind, Tree},
    },
    shared::test_utils::tick,
};

// ── Behavior 5: multiple matching entries are all stamped in order ─────────

#[test]
fn bolt_watcher_stamps_multiple_matching_entries_in_registry_order() {
    let mut app = bolt_only_test_app();
    set_registry(
        &mut app,
        vec![
            (
                EntityKind::Bolt,
                "chip_a".to_string(),
                speed_boost_tree(1.5),
            ),
            (
                EntityKind::Bolt,
                "chip_b".to_string(),
                speed_boost_tree(2.0),
            ),
            (
                EntityKind::Bolt,
                "chip_c".to_string(),
                speed_boost_tree(2.5),
            ),
        ],
    );

    let entity = app.world_mut().spawn(Bolt).id();
    tick(&mut app);

    let bound = app
        .world()
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should be inserted");
    assert_eq!(bound.0.len(), 3);
    assert_eq!(bound.0[0].0, "chip_a");
    assert_eq!(bound.0[0].1, speed_boost_tree(1.5));
    assert_eq!(bound.0[1].0, "chip_b");
    assert_eq!(bound.0[1].1, speed_boost_tree(2.0));
    assert_eq!(bound.0[2].0, "chip_c");
    assert_eq!(bound.0[2].1, speed_boost_tree(2.5));
}

#[test]
fn bolt_watcher_stamps_ten_matching_entries_in_order() {
    let mut app = bolt_only_test_app();
    let entries: Vec<(EntityKind, String, Tree)> = (0..10)
        .map(|i| {
            (
                EntityKind::Bolt,
                format!("chip_{i}"),
                speed_boost_tree((i as f32).mul_add(0.1, 1.0)),
            )
        })
        .collect();
    set_registry(&mut app, entries);

    let entity = app.world_mut().spawn(Bolt).id();
    tick(&mut app);

    let bound = app
        .world()
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should be inserted");
    assert_eq!(bound.0.len(), 10);
    for i in 0..10 {
        assert_eq!(bound.0[i].0, format!("chip_{i}"));
        assert_eq!(bound.0[i].1, speed_boost_tree((i as f32).mul_add(0.1, 1.0)));
    }
}

// ── Behavior 6: multiple entities in the same frame each get stamps ────────

#[test]
fn bolt_watcher_stamps_three_entities_spawned_same_frame() {
    let mut app = bolt_only_test_app();
    set_registry(
        &mut app,
        vec![(
            EntityKind::Bolt,
            "chip_a".to_string(),
            speed_boost_tree(1.5),
        )],
    );

    let e1 = app.world_mut().spawn(Bolt).id();
    let e2 = app.world_mut().spawn(Bolt).id();
    let e3 = app.world_mut().spawn(Bolt).id();
    tick(&mut app);

    for entity in [e1, e2, e3] {
        let bound = app
            .world()
            .get::<BoundEffects>(entity)
            .expect("BoundEffects should exist on every new bolt");
        assert_eq!(bound.0.len(), 1);
        assert_eq!(bound.0[0].0, "chip_a");
        assert_eq!(bound.0[0].1, speed_boost_tree(1.5));
    }
}

#[test]
fn bolt_watcher_ignores_non_bolt_entities_in_same_frame() {
    let mut app = bolt_only_test_app();
    set_registry(
        &mut app,
        vec![(
            EntityKind::Bolt,
            "chip_a".to_string(),
            speed_boost_tree(1.5),
        )],
    );

    let bolt_a = app.world_mut().spawn(Bolt).id();
    let bolt_b = app.world_mut().spawn(Bolt).id();
    let bare = app.world_mut().spawn_empty().id();
    tick(&mut app);

    assert!(app.world().get::<BoundEffects>(bolt_a).is_some());
    assert!(app.world().get::<BoundEffects>(bolt_b).is_some());
    assert!(
        app.world().get::<BoundEffects>(bare).is_none(),
        "bare entity with no Bolt marker must not receive BoundEffects"
    );
}

// ── Behavior 7: cartesian — multiple entities × multiple entries ───────────

#[test]
fn bolt_watcher_cartesian_two_entities_two_entries() {
    let mut app = bolt_only_test_app();
    set_registry(
        &mut app,
        vec![
            (
                EntityKind::Bolt,
                "chip_a".to_string(),
                speed_boost_tree(1.5),
            ),
            (
                EntityKind::Bolt,
                "chip_b".to_string(),
                speed_boost_tree(2.0),
            ),
        ],
    );

    let e1 = app.world_mut().spawn(Bolt).id();
    let e2 = app.world_mut().spawn(Bolt).id();
    tick(&mut app);

    for entity in [e1, e2] {
        let bound = app
            .world()
            .get::<BoundEffects>(entity)
            .expect("BoundEffects should exist");
        assert_eq!(bound.0.len(), 2);
        assert_eq!(bound.0[0].0, "chip_a");
        assert_eq!(bound.0[0].1, speed_boost_tree(1.5));
        assert_eq!(bound.0[1].0, "chip_b");
        assert_eq!(bound.0[1].1, speed_boost_tree(2.0));
    }
}
