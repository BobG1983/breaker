//! Behaviors 1-11 — canonical coverage of the bolt watcher.
//!
//! The bolt watcher is the reference implementation. Cells/walls/breakers
//! share the same code path and are exercised in `cells_walls_breakers.rs`.

use bevy::prelude::*;

use super::helpers::{bolt_only_test_app, set_registry, speed_boost_tree};
use crate::{
    bolt::components::Bolt,
    effect_v3::{
        storage::{BoundEffects, SpawnStampRegistry},
        types::{EntityKind, Tree},
    },
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
    // Mixed registry: a wrong-kind (Cell) entry AND a correct-kind (Bolt)
    // entry. The positive anchor (bolt stamp) fails against the stub,
    // while the negative anchor (no cell stamp) guards the behavior.
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
    // Mixed registry: wrong-kind (Wall) AND correct-kind (Bolt).
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
    // Mixed registry: wrong-kind (Breaker) AND correct-kind (Bolt).
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
    // Mixed registry: wrong-kind (Any) AND correct-kind (Bolt).
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
