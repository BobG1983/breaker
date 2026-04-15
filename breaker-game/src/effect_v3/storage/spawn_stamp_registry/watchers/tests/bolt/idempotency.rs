//! Behaviors 8-11: no re-stamping, late arrivals, no-entities no-op, and registry immutability.

use bevy::prelude::*;

use super::super::helpers::{bolt_only_test_app, set_registry, speed_boost_tree};
use crate::{
    bolt::components::Bolt,
    effect_v3::{
        storage::{BoundEffects, SpawnStampRegistry},
        types::EntityKind,
    },
    shared::test_utils::tick,
};

// ── Behavior 8: no re-stamping on subsequent ticks ─────────────────────────

#[test]
fn bolt_watcher_does_not_restamp_on_subsequent_ticks() {
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
    tick(&mut app);
    tick(&mut app);

    let bound = app
        .world()
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");
    assert_eq!(
        bound.0.len(),
        1,
        "watcher must stamp exactly once (Added<T>), not once per tick"
    );
}

#[test]
fn bolt_watcher_does_not_stamp_late_added_entries_onto_existing_entity() {
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
    tick(&mut app);

    // Inject a new registry entry AFTER the entity has already been stamped.
    app.world_mut()
        .resource_mut::<SpawnStampRegistry>()
        .entries
        .push((
            EntityKind::Bolt,
            "chip_late".to_string(),
            speed_boost_tree(3.0),
        ));
    tick(&mut app);

    let bound = app
        .world()
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");
    assert_eq!(
        bound.0.len(),
        1,
        "already-stamped entity must NOT receive late-added entries \
         (Added<T> is false on tick 3)"
    );
    assert_eq!(bound.0[0].0, "chip_a");
}

// ── Behavior 9: entities arriving on later ticks are also stamped ──────────

#[test]
fn bolt_watcher_stamps_entity_spawned_on_later_tick() {
    let mut app = bolt_only_test_app();
    set_registry(
        &mut app,
        vec![(
            EntityKind::Bolt,
            "chip_a".to_string(),
            speed_boost_tree(1.5),
        )],
    );

    let first = app.world_mut().spawn(Bolt).id();
    tick(&mut app);

    let second = app.world_mut().spawn(Bolt).id();
    tick(&mut app);

    let first_bound = app
        .world()
        .get::<BoundEffects>(first)
        .expect("first bolt should still have BoundEffects");
    assert_eq!(
        first_bound.0.len(),
        1,
        "first bolt must not be double-stamped"
    );
    assert_eq!(first_bound.0[0].0, "chip_a");

    let second_bound = app
        .world()
        .get::<BoundEffects>(second)
        .expect("second bolt should have BoundEffects");
    assert_eq!(second_bound.0.len(), 1);
    assert_eq!(second_bound.0[0].0, "chip_a");
    assert_eq!(second_bound.0[0].1, speed_boost_tree(1.5));
}

#[test]
fn bolt_watcher_picks_up_command_spawned_entities() {
    // Edge case for behavior 9: spawn a bolt via Commands in a system that
    // runs in FixedFirst (commands flush at the end of FixedFirst), then
    // FixedUpdate's watcher picks the bolt up via Added<Bolt>.
    #[derive(Resource, Default)]
    struct SpawnedBolt(Option<Entity>);

    fn spawn_bolt_via_commands(mut commands: Commands, mut store: ResMut<SpawnedBolt>) {
        if store.0.is_none() {
            let e = commands.spawn(Bolt).id();
            store.0 = Some(e);
        }
    }

    let mut app = bolt_only_test_app();
    set_registry(
        &mut app,
        vec![(
            EntityKind::Bolt,
            "chip_a".to_string(),
            speed_boost_tree(1.5),
        )],
    );

    app.world_mut().init_resource::<SpawnedBolt>();
    app.add_systems(FixedFirst, spawn_bolt_via_commands);
    tick(&mut app);

    let spawned_entity = app
        .world()
        .resource::<SpawnedBolt>()
        .0
        .expect("spawn_bolt_via_commands should have spawned a bolt");

    let bound = app
        .world()
        .get::<BoundEffects>(spawned_entity)
        .expect("command-flushed bolt should be picked up by Added<Bolt>");
    assert_eq!(bound.0.len(), 1);
    assert_eq!(bound.0[0].0, "chip_a");
    assert_eq!(bound.0[0].1, speed_boost_tree(1.5));
}

// ── Behavior 10: running with no matching entities is a no-op ──────────────

#[test]
fn bolt_watcher_is_noop_when_world_has_no_bolts() {
    let mut app = bolt_only_test_app();
    set_registry(
        &mut app,
        vec![(
            EntityKind::Bolt,
            "chip_a".to_string(),
            speed_boost_tree(1.5),
        )],
    );

    tick(&mut app);

    // No BoundEffects inserted anywhere in the world.
    let count = app
        .world_mut()
        .query::<&BoundEffects>()
        .iter(app.world())
        .count();
    assert_eq!(count, 0);

    // Positive anchor: spawning a bolt after the no-bolt tick must cause
    // the watcher to stamp it. Against the stub this fails and gives RED
    // signal; combined with the above assertion it locks in the no-op
    // semantics when there are no matching entities.
    let entity = app.world_mut().spawn(Bolt).id();
    tick(&mut app);

    let bound = app
        .world()
        .get::<BoundEffects>(entity)
        .expect("bolt spawned after no-op tick must be stamped");
    assert_eq!(bound.0.len(), 1);
    assert_eq!(bound.0[0].0, "chip_a");
    assert_eq!(bound.0[0].1, speed_boost_tree(1.5));

    // And now there is exactly one BoundEffects in the world.
    let count_after = app
        .world_mut()
        .query::<&BoundEffects>()
        .iter(app.world())
        .count();
    assert_eq!(count_after, 1);
}

// ── Behavior 11: registry entries are not consumed or mutated ──────────────

#[test]
fn bolt_watcher_does_not_mutate_registry() {
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

    let registry = app.world().resource::<SpawnStampRegistry>();
    assert_eq!(registry.entries.len(), 1);
    assert_eq!(registry.entries[0].0, EntityKind::Bolt);
    assert_eq!(registry.entries[0].1, "chip_a");
    assert_eq!(registry.entries[0].2, speed_boost_tree(1.5));

    // The entity still got its stamp.
    let bound = app
        .world()
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");
    assert_eq!(bound.0[0].1, speed_boost_tree(1.5));
}

#[test]
fn bolt_watcher_registry_stable_across_multiple_ticks_and_entities() {
    let mut app = bolt_only_test_app();
    set_registry(
        &mut app,
        vec![(
            EntityKind::Bolt,
            "chip_a".to_string(),
            speed_boost_tree(1.5),
        )],
    );

    let entity_a = app.world_mut().spawn(Bolt).id();
    tick(&mut app);

    // Positive anchor: entity_a must have been stamped on its spawn tick.
    // Against the stub this fails and gives RED signal.
    assert!(
        app.world().get::<BoundEffects>(entity_a).is_some(),
        "first bolt must be stamped on its spawn tick"
    );

    let entity_b = app.world_mut().spawn(Bolt).id();
    tick(&mut app);

    // Positive anchor: entity_b must have been stamped on its spawn tick.
    assert!(
        app.world().get::<BoundEffects>(entity_b).is_some(),
        "second bolt must be stamped on its spawn tick"
    );

    let registry = app.world().resource::<SpawnStampRegistry>();
    assert_eq!(
        registry.entries.len(),
        1,
        "registry must not grow or shrink as a result of stamping"
    );
    assert_eq!(registry.entries[0].1, "chip_a");
    assert_eq!(registry.entries[0].2, speed_boost_tree(1.5));
}
