//! Tests for `PhantomBolt::become_normal`.

use std::collections::HashSet;

use bevy::prelude::*;

use super::{super::inner::*, helpers::test_app};
use crate::{
    bolt::components::{Bolt, BoltBaseDamage, ExtraBolt, PiercingRemaining, PrimaryBolt},
    prelude::*,
    shared::{Lifespan, PhantomFlicker},
};

// ── Behavior 7: become_normal strips PhantomBolt ─────────────

#[test]
fn become_normal_strips_phantom_bolt_marker() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            Bolt,
            PhantomBolt,
            PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
            PhantomDamagedCells::default(),
        ))
        .id();
    // must patch key with real entity after spawn
    let key = PhantomDedupKey::Bolt(e);
    app.world_mut().entity_mut(e).insert(key);
    app.add_systems(Update, move |mut commands: Commands| {
        PhantomBolt::become_normal(&mut commands, e);
    });
    app.update();
    assert!(
        app.world().get::<PhantomBolt>(e).is_none(),
        "PhantomBolt should be removed after become_normal"
    );
}

#[test]
fn become_normal_on_non_phantom_bolt_does_not_panic() {
    let mut app = test_app();
    let e = app.world_mut().spawn(Bolt).id();
    app.add_systems(Update, move |mut commands: Commands| {
        PhantomBolt::become_normal(&mut commands, e);
    });
    app.update();
    assert!(
        app.world().get::<Bolt>(e).is_some(),
        "Bolt marker should still be present"
    );
    assert!(
        app.world().get::<PhantomBolt>(e).is_none(),
        "PhantomBolt should remain absent"
    );
}

// ── Behavior 8: become_normal strips PhantomDedupKey ─────────

#[test]
fn become_normal_strips_dedup_key_bolt_variant() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            Bolt,
            PhantomBolt,
            PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
            PhantomDamagedCells::default(),
        ))
        .id();
    let key = PhantomDedupKey::Bolt(e);
    app.world_mut().entity_mut(e).insert(key);
    app.add_systems(Update, move |mut commands: Commands| {
        PhantomBolt::become_normal(&mut commands, e);
    });
    app.update();
    assert!(
        app.world().get::<PhantomDedupKey>(e).is_none(),
        "PhantomDedupKey should be removed after become_normal"
    );
}

#[test]
fn become_normal_strips_dedup_key_chip_variant() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            Bolt,
            PhantomBolt,
            PhantomDedupKey::Chip {
                chip:       "phantom_bolt".to_string(),
                fired_from: Entity::PLACEHOLDER,
            },
            PhantomDamagedCells::default(),
        ))
        .id();
    let src = e;
    app.world_mut().entity_mut(e).insert(PhantomDedupKey::Chip {
        chip:       "phantom_bolt".to_string(),
        fired_from: src,
    });
    app.add_systems(Update, move |mut commands: Commands| {
        PhantomBolt::become_normal(&mut commands, e);
    });
    app.update();
    assert!(
        app.world().get::<PhantomDedupKey>(e).is_none(),
        "PhantomDedupKey Chip variant should be removed after become_normal"
    );
}

// ── Behavior 9: become_normal strips PhantomDamagedCells ─────

#[test]
fn become_normal_strips_empty_damaged_cells() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            Bolt,
            PhantomBolt,
            PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
            PhantomDamagedCells::default(),
        ))
        .id();
    app.world_mut()
        .entity_mut(e)
        .insert(PhantomDedupKey::Bolt(e));
    app.add_systems(Update, move |mut commands: Commands| {
        PhantomBolt::become_normal(&mut commands, e);
    });
    app.update();
    assert!(
        app.world().get::<PhantomDamagedCells>(e).is_none(),
        "PhantomDamagedCells should be removed after become_normal"
    );
}

#[test]
fn become_normal_strips_populated_damaged_cells() {
    let mut app = test_app();
    let cell_a = app.world_mut().spawn_empty().id();
    let cell_b = app.world_mut().spawn_empty().id();
    let cell_c = app.world_mut().spawn_empty().id();
    let mut cells = HashSet::new();
    cells.insert(cell_a);
    cells.insert(cell_b);
    cells.insert(cell_c);
    let e = app
        .world_mut()
        .spawn((
            Bolt,
            PhantomBolt,
            PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
            PhantomDamagedCells(cells),
        ))
        .id();
    app.world_mut()
        .entity_mut(e)
        .insert(PhantomDedupKey::Bolt(e));
    app.add_systems(Update, move |mut commands: Commands| {
        PhantomBolt::become_normal(&mut commands, e);
    });
    app.update();
    assert!(
        app.world().get::<PhantomDamagedCells>(e).is_none(),
        "Populated PhantomDamagedCells should be fully removed after become_normal"
    );
}

// ── Behavior 10: become_normal strips PhantomFlicker ─────────

#[test]
fn become_normal_strips_phantom_flicker() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            Bolt,
            PhantomBolt,
            PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
            PhantomDamagedCells::default(),
            PhantomFlicker {
                frequency: 4.0,
                min_alpha: 0.3,
            },
        ))
        .id();
    app.world_mut()
        .entity_mut(e)
        .insert(PhantomDedupKey::Bolt(e));
    app.add_systems(Update, move |mut commands: Commands| {
        PhantomBolt::become_normal(&mut commands, e);
    });
    app.update();
    assert!(
        app.world().get::<PhantomFlicker>(e).is_none(),
        "PhantomFlicker should be removed after become_normal"
    );
}

#[test]
fn become_normal_without_phantom_flicker_does_not_panic() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            Bolt,
            PhantomBolt,
            PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
            PhantomDamagedCells::default(),
        ))
        .id();
    app.world_mut()
        .entity_mut(e)
        .insert(PhantomDedupKey::Bolt(e));
    app.add_systems(Update, move |mut commands: Commands| {
        PhantomBolt::become_normal(&mut commands, e);
    });
    app.update();
    assert!(
        app.world().get::<PhantomFlicker>(e).is_none(),
        "PhantomFlicker should remain absent when not present"
    );
}

// ── Behavior 11: become_normal strips Lifespan ───────────────

#[test]
fn become_normal_strips_lifespan() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            Bolt,
            PhantomBolt,
            PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
            PhantomDamagedCells::default(),
            Lifespan { remaining: 2.5 },
        ))
        .id();
    app.world_mut()
        .entity_mut(e)
        .insert(PhantomDedupKey::Bolt(e));
    app.add_systems(Update, move |mut commands: Commands| {
        PhantomBolt::become_normal(&mut commands, e);
    });
    app.update();
    assert!(
        app.world().get::<Lifespan>(e).is_none(),
        "Lifespan should be removed after become_normal"
    );
}

#[test]
fn become_normal_strips_lifespan_with_zero_remaining() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            Bolt,
            PhantomBolt,
            PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
            PhantomDamagedCells::default(),
            Lifespan { remaining: 0.0 },
        ))
        .id();
    app.world_mut()
        .entity_mut(e)
        .insert(PhantomDedupKey::Bolt(e));
    app.add_systems(Update, move |mut commands: Commands| {
        PhantomBolt::become_normal(&mut commands, e);
    });
    app.update();
    assert!(
        app.world().get::<Lifespan>(e).is_none(),
        "Lifespan with remaining=0.0 should also be removed"
    );
}

// ── Behavior 12: become_normal strips LifetimeEndBehavior ────

#[test]
fn become_normal_strips_lifetime_end_behavior_revert() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            Bolt,
            PhantomBolt,
            PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
            PhantomDamagedCells::default(),
            LifetimeEndBehavior::RevertToNormalBolt,
        ))
        .id();
    app.world_mut()
        .entity_mut(e)
        .insert(PhantomDedupKey::Bolt(e));
    app.add_systems(Update, move |mut commands: Commands| {
        PhantomBolt::become_normal(&mut commands, e);
    });
    app.update();
    assert!(
        app.world().get::<LifetimeEndBehavior>(e).is_none(),
        "LifetimeEndBehavior::RevertToNormalBolt should be removed"
    );
}

#[test]
fn become_normal_strips_lifetime_end_behavior_despawn() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            Bolt,
            PhantomBolt,
            PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
            PhantomDamagedCells::default(),
            LifetimeEndBehavior::Despawn,
        ))
        .id();
    app.world_mut()
        .entity_mut(e)
        .insert(PhantomDedupKey::Bolt(e));
    app.add_systems(Update, move |mut commands: Commands| {
        PhantomBolt::become_normal(&mut commands, e);
    });
    app.update();
    assert!(
        app.world().get::<LifetimeEndBehavior>(e).is_none(),
        "LifetimeEndBehavior::Despawn should be removed"
    );
}

// ── Behavior 13: become_normal preserves role/velocity/position/cleanup ──

#[test]
fn become_normal_preserves_primary_bolt_and_components() {
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
            PhantomBolt,
            PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
            PhantomDamagedCells::default(),
            PhantomFlicker {
                frequency: 4.0,
                min_alpha: 0.3,
            },
            Lifespan { remaining: 2.5 },
            LifetimeEndBehavior::RevertToNormalBolt,
        ))
        .id();
    app.world_mut()
        .entity_mut(e)
        .insert(PhantomDedupKey::Bolt(e));
    app.add_systems(Update, move |mut commands: Commands| {
        PhantomBolt::become_normal(&mut commands, e);
    });
    app.update();
    assert!(
        app.world().get::<PhantomBolt>(e).is_none(),
        "become_normal did not execute — PhantomBolt still present"
    );
    assert!(
        app.world().get::<Bolt>(e).is_some(),
        "Bolt should still be present"
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
fn become_normal_preserves_extra_bolt_and_node_cleanup() {
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
            PhantomBolt,
            PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
            PhantomDamagedCells::default(),
        ))
        .id();
    app.world_mut()
        .entity_mut(e)
        .insert(PhantomDedupKey::Bolt(e));
    app.add_systems(Update, move |mut commands: Commands| {
        PhantomBolt::become_normal(&mut commands, e);
    });
    app.update();
    assert!(
        app.world().get::<PhantomBolt>(e).is_none(),
        "become_normal did not execute — PhantomBolt still present"
    );
    assert!(
        app.world().get::<ExtraBolt>(e).is_some(),
        "ExtraBolt should still be present"
    );
    assert!(
        app.world().get::<CleanupOnExit<NodeState>>(e).is_some(),
        "CleanupOnExit<NodeState> should still be present after become_normal"
    );
}
