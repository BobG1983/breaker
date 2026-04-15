//! Behaviors 1-2: insert `BoundEffects` and append to existing.

use bevy::prelude::*;

use super::super::helpers::{bolt_only_test_app, set_registry, speed_boost_tree};
use crate::{
    bolt::components::Bolt,
    effect_v3::{storage::BoundEffects, types::EntityKind},
    shared::test_utils::tick,
};

// ── Behavior 1: insert BoundEffects with matching tree ─────────────────────

#[test]
fn bolt_watcher_inserts_bound_effects_when_missing() {
    let mut app = bolt_only_test_app();
    set_registry(
        &mut app,
        vec![(
            EntityKind::Bolt,
            "chip_a".to_string(),
            speed_boost_tree(1.5),
        )],
    );

    let entity = app.world_mut().spawn(Bolt).id();
    tick(&mut app);

    let bound = app
        .world()
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should be inserted after tick");
    assert_eq!(bound.0.len(), 1);
    assert_eq!(bound.0[0].0, "chip_a");
    assert_eq!(bound.0[0].1, speed_boost_tree(1.5));
}

#[test]
fn bolt_watcher_inserts_empty_string_name_entry() {
    let mut app = bolt_only_test_app();
    set_registry(
        &mut app,
        vec![(EntityKind::Bolt, String::new(), speed_boost_tree(1.5))],
    );

    let entity = app.world_mut().spawn(Bolt).id();
    tick(&mut app);

    let bound = app
        .world()
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should be inserted after tick");
    assert_eq!(bound.0.len(), 1);
    assert_eq!(bound.0[0].0, "", "empty-string name must be preserved");
}

// ── Behavior 2: append to existing BoundEffects ────────────────────────────

#[test]
fn bolt_watcher_appends_to_existing_bound_effects() {
    let mut app = bolt_only_test_app();
    set_registry(
        &mut app,
        vec![(
            EntityKind::Bolt,
            "chip_a".to_string(),
            speed_boost_tree(1.5),
        )],
    );

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            BoundEffects(vec![("pre_existing".to_string(), speed_boost_tree(2.0))]),
        ))
        .id();
    tick(&mut app);

    let bound = app
        .world()
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should still exist");
    assert_eq!(bound.0.len(), 2);
    assert_eq!(bound.0[0].0, "pre_existing");
    assert_eq!(bound.0[0].1, speed_boost_tree(2.0));
    assert_eq!(bound.0[1].0, "chip_a");
    assert_eq!(bound.0[1].1, speed_boost_tree(1.5));
}

#[test]
fn bolt_watcher_appends_to_empty_bound_effects_vec() {
    let mut app = bolt_only_test_app();
    set_registry(
        &mut app,
        vec![(
            EntityKind::Bolt,
            "chip_a".to_string(),
            speed_boost_tree(1.5),
        )],
    );

    let entity = app.world_mut().spawn((Bolt, BoundEffects(vec![]))).id();
    tick(&mut app);

    let bound = app
        .world()
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should still exist");
    assert_eq!(bound.0.len(), 1);
    assert_eq!(bound.0[0].0, "chip_a");
    assert_eq!(bound.0[0].1, speed_boost_tree(1.5));
}
