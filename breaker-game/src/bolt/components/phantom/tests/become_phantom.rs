//! Tests for `Bolt::become_phantom`.

use std::collections::HashSet;

use bevy::prelude::*;

use super::{super::inner::*, helpers::test_app};
use crate::{
    bolt::components::{Bolt, BoltBaseDamage, ExtraBolt, PiercingRemaining, PrimaryBolt},
    prelude::*,
    shared::{Lifespan, PhantomFlicker},
};

// ── Behavior 1: become_phantom inserts PhantomBolt ──────────

#[test]
fn become_phantom_inserts_phantom_bolt_marker() {
    let mut app = test_app();
    let e = app.world_mut().spawn(Bolt).id();
    app.add_systems(Update, move |mut commands: Commands| {
        Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
    });
    app.update();
    assert!(
        app.world().get::<PhantomBolt>(e).is_some(),
        "PhantomBolt marker should be present after become_phantom"
    );
}

#[test]
fn become_phantom_twice_is_idempotent_phantom_bolt_still_present() {
    let mut app = test_app();
    let e = app.world_mut().spawn(Bolt).id();
    app.add_systems(Update, move |mut commands: Commands| {
        Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
        Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
    });
    app.update();
    assert!(
        app.world().get::<PhantomBolt>(e).is_some(),
        "PhantomBolt should still be present after double become_phantom"
    );
}

// ── Behavior 2: become_phantom inserts PhantomDedupKey::Bolt ─

#[test]
fn become_phantom_inserts_dedup_key_bolt_variant() {
    let mut app = test_app();
    let e = app.world_mut().spawn(Bolt).id();
    app.add_systems(Update, move |mut commands: Commands| {
        Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
    });
    app.update();
    let key = app
        .world()
        .get::<PhantomDedupKey>(e)
        .expect("PhantomDedupKey should be present after become_phantom");
    assert_eq!(
        *key,
        PhantomDedupKey::Bolt(e),
        "PhantomDedupKey should be Bolt(e), got {key:?}"
    );
}

#[test]
fn become_phantom_preserves_different_entity_in_bolt_key() {
    let mut app = test_app();
    let e = app.world_mut().spawn(Bolt).id();
    let other = app.world_mut().spawn_empty().id();
    app.add_systems(Update, move |mut commands: Commands| {
        Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(other));
    });
    app.update();
    let key = app
        .world()
        .get::<PhantomDedupKey>(e)
        .expect("PhantomDedupKey should be present");
    assert_eq!(
        *key,
        PhantomDedupKey::Bolt(other),
        "become_phantom must not rewrite the supplied entity in the key"
    );
}

// ── Behavior 3: become_phantom inserts PhantomDedupKey::Chip ──

#[test]
fn become_phantom_inserts_dedup_key_chip_variant() {
    let mut app = test_app();
    let e = app.world_mut().spawn(Bolt).id();
    let src = app.world_mut().spawn_empty().id();
    app.add_systems(Update, move |mut commands: Commands| {
        Bolt::become_phantom(
            &mut commands,
            e,
            PhantomDedupKey::Chip {
                chip:       "phantom_bolt".to_string(),
                fired_from: src,
            },
        );
    });
    app.update();
    let key = app
        .world()
        .get::<PhantomDedupKey>(e)
        .expect("PhantomDedupKey should be present");
    assert_eq!(
        *key,
        PhantomDedupKey::Chip {
            chip:       "phantom_bolt".to_string(),
            fired_from: src,
        },
        "PhantomDedupKey should be Chip variant with correct fields"
    );
}

#[test]
fn become_phantom_preserves_empty_chip_string() {
    let mut app = test_app();
    let e = app.world_mut().spawn(Bolt).id();
    let src = app.world_mut().spawn_empty().id();
    app.add_systems(Update, move |mut commands: Commands| {
        Bolt::become_phantom(
            &mut commands,
            e,
            PhantomDedupKey::Chip {
                chip:       String::new(),
                fired_from: src,
            },
        );
    });
    app.update();
    let key = app
        .world()
        .get::<PhantomDedupKey>(e)
        .expect("PhantomDedupKey should be present");
    assert_eq!(
        *key,
        PhantomDedupKey::Chip {
            chip:       String::new(),
            fired_from: src,
        },
        "empty chip string should be preserved verbatim"
    );
}

// ── Behavior 4: become_phantom inserts empty PhantomDamagedCells ──

#[test]
fn become_phantom_inserts_empty_damaged_cells() {
    let mut app = test_app();
    let e = app.world_mut().spawn(Bolt).id();
    app.add_systems(Update, move |mut commands: Commands| {
        Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
    });
    app.update();
    let cells = app
        .world()
        .get::<PhantomDamagedCells>(e)
        .expect("PhantomDamagedCells should be present after become_phantom");
    assert!(
        cells.0.is_empty(),
        "PhantomDamagedCells should be an empty HashSet, got {} entries",
        cells.0.len()
    );
}

#[test]
fn become_phantom_overwrites_existing_damaged_cells_with_empty_set() {
    let mut app = test_app();
    let cell_a = app.world_mut().spawn_empty().id();
    let cell_b = app.world_mut().spawn_empty().id();
    let mut pre_populated = HashSet::new();
    pre_populated.insert(cell_a);
    pre_populated.insert(cell_b);
    let e = app
        .world_mut()
        .spawn((Bolt, PhantomDamagedCells(pre_populated)))
        .id();
    app.add_systems(Update, move |mut commands: Commands| {
        Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
    });
    app.update();
    let cells = app
        .world()
        .get::<PhantomDamagedCells>(e)
        .expect("PhantomDamagedCells should be present after become_phantom");
    assert!(
        cells.0.is_empty(),
        "become_phantom should overwrite pre-existing PhantomDamagedCells with empty set"
    );
}

// ── Behavior 5: become_phantom does NOT insert PhantomFlicker/Lifespan/LifetimeEndBehavior ──

#[test]
fn become_phantom_does_not_insert_phantom_flicker() {
    let mut app = test_app();
    let e = app.world_mut().spawn(Bolt).id();
    app.add_systems(Update, move |mut commands: Commands| {
        Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
    });
    app.update();
    assert!(
        app.world().get::<PhantomBolt>(e).is_some(),
        "become_phantom did not execute — PhantomBolt not present"
    );
    assert!(
        app.world().get::<PhantomFlicker>(e).is_none(),
        "become_phantom must not insert PhantomFlicker"
    );
}

#[test]
fn become_phantom_does_not_insert_lifespan() {
    let mut app = test_app();
    let e = app.world_mut().spawn(Bolt).id();
    app.add_systems(Update, move |mut commands: Commands| {
        Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
    });
    app.update();
    assert!(
        app.world().get::<PhantomBolt>(e).is_some(),
        "become_phantom did not execute — PhantomBolt not present"
    );
    assert!(
        app.world().get::<Lifespan>(e).is_none(),
        "become_phantom must not insert Lifespan"
    );
}

#[test]
fn become_phantom_does_not_insert_lifetime_end_behavior() {
    let mut app = test_app();
    let e = app.world_mut().spawn(Bolt).id();
    app.add_systems(Update, move |mut commands: Commands| {
        Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
    });
    app.update();
    assert!(
        app.world().get::<PhantomBolt>(e).is_some(),
        "become_phantom did not execute — PhantomBolt not present"
    );
    assert!(
        app.world().get::<LifetimeEndBehavior>(e).is_none(),
        "become_phantom must not insert LifetimeEndBehavior"
    );
}

#[test]
fn become_phantom_does_not_remove_preexisting_lifespan() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            Bolt,
            Lifespan { remaining: 2.5 },
            LifetimeEndBehavior::Despawn,
        ))
        .id();
    app.add_systems(Update, move |mut commands: Commands| {
        Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
    });
    app.update();
    assert!(
        app.world().get::<PhantomBolt>(e).is_some(),
        "become_phantom did not execute — PhantomBolt not present"
    );
    let lifespan = app
        .world()
        .get::<Lifespan>(e)
        .expect("pre-existing Lifespan should remain after become_phantom");
    assert!(
        (lifespan.remaining - 2.5).abs() < f32::EPSILON,
        "Lifespan.remaining should still be 2.5"
    );
    let leb = app
        .world()
        .get::<LifetimeEndBehavior>(e)
        .expect("pre-existing LifetimeEndBehavior should remain after become_phantom");
    assert_eq!(
        *leb,
        LifetimeEndBehavior::Despawn,
        "LifetimeEndBehavior should still be Despawn"
    );
}

// ── Behavior 6: become_phantom does NOT mutate role/velocity/position/cleanup ──

#[test]
fn become_phantom_preserves_primary_bolt_and_components() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            Bolt,
            PrimaryBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(10.0, 20.0)),
            CleanupOnExit::<RunState>::default(),
            BoltBaseDamage(10.0),
            PiercingRemaining(3),
        ))
        .id();
    app.add_systems(Update, move |mut commands: Commands| {
        Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
    });
    app.update();
    assert!(
        app.world().get::<PhantomBolt>(e).is_some(),
        "become_phantom did not execute — PhantomBolt not present"
    );
    assert!(
        app.world().get::<PrimaryBolt>(e).is_some(),
        "PrimaryBolt should still be present"
    );
    let vel = app
        .world()
        .get::<Velocity2D>(e)
        .expect("Velocity2D should still be present");
    assert_eq!(
        vel.0,
        Vec2::new(0.0, 400.0),
        "Velocity2D should be unchanged"
    );
    let pos = app
        .world()
        .get::<Position2D>(e)
        .expect("Position2D should still be present");
    assert_eq!(
        pos.0,
        Vec2::new(10.0, 20.0),
        "Position2D should be unchanged"
    );
    assert!(
        app.world().get::<CleanupOnExit<RunState>>(e).is_some(),
        "CleanupOnExit<RunState> should still be present"
    );
    let dmg = app
        .world()
        .get::<BoltBaseDamage>(e)
        .expect("BoltBaseDamage should still be present");
    assert!(
        (dmg.0 - 10.0).abs() < f32::EPSILON,
        "BoltBaseDamage should still be 10.0"
    );
    let piercing = app
        .world()
        .get::<PiercingRemaining>(e)
        .expect("PiercingRemaining should still be present");
    assert_eq!(piercing.0, 3, "PiercingRemaining should still be 3");
}

#[test]
fn become_phantom_preserves_extra_bolt_and_node_cleanup() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(10.0, 20.0)),
            CleanupOnExit::<NodeState>::default(),
            BoltBaseDamage(10.0),
            PiercingRemaining(3),
        ))
        .id();
    app.add_systems(Update, move |mut commands: Commands| {
        Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
    });
    app.update();
    assert!(
        app.world().get::<PhantomBolt>(e).is_some(),
        "become_phantom did not execute — PhantomBolt not present"
    );
    assert!(
        app.world().get::<ExtraBolt>(e).is_some(),
        "ExtraBolt should still be present"
    );
    assert!(
        app.world().get::<CleanupOnExit<NodeState>>(e).is_some(),
        "CleanupOnExit<NodeState> should still be present"
    );
}
