//! Phantom-bolt lifespan-end behaviors:
//! Beh 1 — `LifetimeEndBehavior::Despawn` (T14)
//! Beh 2 — `LifetimeEndBehavior::RevertToNormalBolt` (T15)

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::components::{
        BoltBaseDamage, ExtraBolt, LifetimeEndBehavior, PhantomBolt, PhantomDamagedCells,
        PhantomDedupKey, PiercingRemaining, PrimaryBolt,
    },
    prelude::*,
    shared::{Lifespan, PhantomFlicker},
    state::types::{NodeState, RunState},
};

// ── Behavior 1 — T14: Despawn on LifetimeEndBehavior::Despawn ───────────────

/// T14 primary: phantom bolt with `Lifespan { remaining: 0.01 }` +
/// `LifetimeEndBehavior::Despawn` emits exactly one `DespawnEntity` per tick.
#[test]
fn phantom_bolt_despawn_behavior_emits_despawn_entity_on_expiry() {
    let mut app = lifespan_emit_app();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            PhantomBolt,
            PhantomDamagedCells::default(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, 0.0)),
            Lifespan { remaining: 0.01 },
            LifetimeEndBehavior::Despawn,
        ))
        .id();
    app.world_mut()
        .entity_mut(bolt)
        .insert(PhantomDedupKey::Bolt(bolt));

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        collector.0.len(),
        1,
        "exactly one DespawnEntity expected on expiry; got {}",
        collector.0.len()
    );
    assert_eq!(
        collector.0[0].entity, bolt,
        "DespawnEntity.entity must match the bolt"
    );

    let ky = app.world().resource::<CapturedKillYourselfBolt>();
    assert!(
        ky.0.is_empty(),
        "zero KillYourself<Bolt> must be emitted — legacy writer must be gone"
    );

    let lifespan = app
        .world()
        .get::<Lifespan>(bolt)
        .expect("Lifespan still on entity after tick");
    let expected = 0.01_f32 - FIXED_DT;
    assert!(
        (lifespan.remaining - expected).abs() < 1e-4,
        "Lifespan.remaining should be ≈ {expected} after tick, got {}",
        lifespan.remaining
    );

    // Phantom components still present — system does NOT strip on Despawn branch.
    assert!(app.world().get::<PhantomBolt>(bolt).is_some());
    assert!(app.world().get::<PhantomDedupKey>(bolt).is_some());
    assert!(app.world().get::<PhantomDamagedCells>(bolt).is_some());
    assert!(app.world().get::<Lifespan>(bolt).is_some());
    assert_eq!(
        *app.world().get::<LifetimeEndBehavior>(bolt).unwrap(),
        LifetimeEndBehavior::Despawn
    );
}

/// T14 edge 1a: `Lifespan { remaining: 0.0 }` boundary emits exactly one
/// `DespawnEntity` and decrements to ≈ `-FIXED_DT`.
#[test]
fn phantom_bolt_despawn_behavior_at_zero_boundary_emits_despawn_entity() {
    let mut app = lifespan_emit_app();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            PhantomBolt,
            PhantomDamagedCells::default(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, 0.0)),
            Lifespan { remaining: 0.0 },
            LifetimeEndBehavior::Despawn,
        ))
        .id();
    app.world_mut()
        .entity_mut(bolt)
        .insert(PhantomDedupKey::Bolt(bolt));

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        collector.0.len(),
        1,
        "remaining=0.0 boundary must emit exactly one DespawnEntity"
    );
    assert_eq!(collector.0[0].entity, bolt);

    let lifespan = app
        .world()
        .get::<Lifespan>(bolt)
        .expect("Lifespan still present");
    assert!(
        (lifespan.remaining - (-FIXED_DT)).abs() < 1e-4,
        "remaining should be ≈ -FIXED_DT after tick, got {}",
        lifespan.remaining
    );
}

/// T14 edge 1b: without `RantzDmgPlugin`, entity is still alive after tick
/// (system emits but does NOT despawn directly).
#[test]
fn phantom_bolt_despawn_behavior_system_only_emits_does_not_despawn_directly() {
    let mut app = lifespan_emit_app();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            PhantomBolt,
            PhantomDamagedCells::default(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, 0.0)),
            Lifespan { remaining: 0.01 },
            LifetimeEndBehavior::Despawn,
        ))
        .id();
    app.world_mut()
        .entity_mut(bolt)
        .insert(PhantomDedupKey::Bolt(bolt));

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(collector.0.len(), 1, "DespawnEntity must be emitted");
    assert_eq!(collector.0[0].entity, bolt);

    assert!(
        app.world().get_entity(bolt).is_ok(),
        "without RantzDmgPlugin the entity must NOT be despawned directly"
    );
}

/// T14 edge 1c: with `RantzDmgPlugin` (`lifespan_despawn_app`), entity is
/// despawned within the same tick.
#[test]
fn phantom_bolt_despawn_behavior_entity_despawned_via_dmg_pipeline() {
    let mut app = lifespan_despawn_app();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            PhantomBolt,
            PhantomDamagedCells::default(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, 0.0)),
            Lifespan { remaining: 0.01 },
            LifetimeEndBehavior::Despawn,
        ))
        .id();
    app.world_mut()
        .entity_mut(bolt)
        .insert(PhantomDedupKey::Bolt(bolt));

    tick(&mut app);

    assert!(
        app.world().get_entity(bolt).is_err(),
        "entity must be despawned via process_despawn_requests within the same tick"
    );

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(collector.0.len(), 1, "exactly one DespawnEntity recorded");
    assert_eq!(collector.0[0].entity, bolt);
}

// ── Behavior 2 — T15: RevertToNormalBolt strips phantom set ─────────────────

/// T15 primary: `LifetimeEndBehavior::RevertToNormalBolt` calls
/// `PhantomBolt::become_normal` on expiry — phantom set stripped, bolt alive,
/// no `DespawnEntity`.
#[test]
fn phantom_bolt_revert_behavior_strips_phantom_components_on_expiry() {
    let mut app = lifespan_emit_app();
    let bolt = app
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
            PhantomDamagedCells::default(),
            PhantomFlicker {
                frequency: 4.0,
                min_alpha: 0.3,
            },
            Lifespan { remaining: 0.01 },
            LifetimeEndBehavior::RevertToNormalBolt,
        ))
        .id();
    app.world_mut()
        .entity_mut(bolt)
        .insert(PhantomDedupKey::Bolt(bolt));

    tick(&mut app);

    // Entity still alive.
    assert!(
        app.world().get_entity(bolt).is_ok(),
        "bolt must remain alive after RevertToNormalBolt"
    );

    // No DespawnEntity.
    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(
        collector.0.is_empty(),
        "zero DespawnEntity must be emitted for RevertToNormalBolt"
    );

    // No KillYourself<Bolt>.
    let ky = app.world().resource::<CapturedKillYourselfBolt>();
    assert!(ky.0.is_empty(), "zero KillYourself<Bolt>");

    // Phantom components GONE.
    assert!(
        app.world().get::<PhantomBolt>(bolt).is_none(),
        "PhantomBolt must be stripped by become_normal"
    );
    assert!(
        app.world().get::<PhantomDedupKey>(bolt).is_none(),
        "PhantomDedupKey must be stripped"
    );
    assert!(
        app.world().get::<PhantomDamagedCells>(bolt).is_none(),
        "PhantomDamagedCells must be stripped"
    );
    assert!(
        app.world().get::<PhantomFlicker>(bolt).is_none(),
        "PhantomFlicker must be stripped"
    );
    assert!(
        app.world().get::<Lifespan>(bolt).is_none(),
        "Lifespan must be stripped by become_normal"
    );
    assert!(
        app.world().get::<LifetimeEndBehavior>(bolt).is_none(),
        "LifetimeEndBehavior must be stripped by become_normal"
    );

    // Normal bolt components PRESENT and UNCHANGED.
    assert!(
        app.world().get::<Bolt>(bolt).is_some(),
        "Bolt marker present"
    );
    assert!(
        app.world().get::<PrimaryBolt>(bolt).is_some(),
        "PrimaryBolt present"
    );
    let vel = app
        .world()
        .get::<Velocity2D>(bolt)
        .expect("Velocity2D present");
    assert_eq!(vel.0, Vec2::new(0.0, 400.0), "Velocity2D unchanged");
    let pos = app
        .world()
        .get::<Position2D>(bolt)
        .expect("Position2D present");
    assert_eq!(pos.0, Vec2::new(10.0, 20.0), "Position2D unchanged");
    assert!(
        app.world().get::<CleanupOnExit<RunState>>(bolt).is_some(),
        "CleanupOnExit<RunState> present"
    );
    let dmg = app
        .world()
        .get::<BoltBaseDamage>(bolt)
        .expect("BoltBaseDamage present");
    assert!(
        (dmg.0 - 10.0).abs() < f32::EPSILON,
        "BoltBaseDamage unchanged"
    );
    let piercing = app
        .world()
        .get::<PiercingRemaining>(bolt)
        .expect("PiercingRemaining present");
    assert_eq!(piercing.0, 3, "PiercingRemaining unchanged");
}

/// T15 edge 2a: `Lifespan { remaining: 0.0 }` boundary with
/// `RevertToNormalBolt` — same revert outcome, entity alive, phantom set gone,
/// no `DespawnEntity`.
#[test]
fn phantom_bolt_revert_behavior_at_zero_boundary_strips_phantom_set() {
    let mut app = lifespan_emit_app();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            PrimaryBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(10.0, 20.0)),
            PhantomBolt,
            PhantomDamagedCells::default(),
            Lifespan { remaining: 0.0 },
            LifetimeEndBehavior::RevertToNormalBolt,
        ))
        .id();
    app.world_mut()
        .entity_mut(bolt)
        .insert(PhantomDedupKey::Bolt(bolt));

    tick(&mut app);

    assert!(
        app.world().get_entity(bolt).is_ok(),
        "bolt alive after revert at boundary"
    );
    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(
        collector.0.is_empty(),
        "zero DespawnEntity on RevertToNormalBolt at 0.0 boundary"
    );
    assert!(
        app.world().get::<PhantomBolt>(bolt).is_none(),
        "PhantomBolt stripped at 0.0 boundary"
    );
    assert!(
        app.world().get::<Lifespan>(bolt).is_none(),
        "Lifespan stripped by become_normal"
    );
}

/// T15 edge 2b: `RevertToNormalBolt` on an `ExtraBolt` — phantom set stripped,
/// `ExtraBolt` and `CleanupOnExit<NodeState>` preserved.
#[test]
fn extra_bolt_revert_behavior_strips_phantom_and_preserves_extra_bolt_components() {
    let mut app = lifespan_emit_app();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(5.0, 10.0)),
            CleanupOnExit::<NodeState>::default(),
            PhantomBolt,
            PhantomDamagedCells::default(),
            Lifespan { remaining: 0.01 },
            LifetimeEndBehavior::RevertToNormalBolt,
        ))
        .id();
    app.world_mut()
        .entity_mut(bolt)
        .insert(PhantomDedupKey::Bolt(bolt));

    tick(&mut app);

    assert!(
        app.world().get_entity(bolt).is_ok(),
        "ExtraBolt alive after revert"
    );
    assert!(
        app.world().get::<PhantomBolt>(bolt).is_none(),
        "PhantomBolt stripped"
    );
    assert!(
        app.world().get::<ExtraBolt>(bolt).is_some(),
        "ExtraBolt preserved"
    );
    assert!(
        app.world().get::<CleanupOnExit<NodeState>>(bolt).is_some(),
        "CleanupOnExit<NodeState> preserved"
    );
    let vel = app
        .world()
        .get::<Velocity2D>(bolt)
        .expect("Velocity2D present");
    assert_eq!(vel.0, Vec2::new(0.0, 400.0));
    let pos = app
        .world()
        .get::<Position2D>(bolt)
        .expect("Position2D present");
    assert_eq!(pos.0, Vec2::new(5.0, 10.0));
}

/// T15 edge 2c: `Lifespan { remaining: 1.0 }` — pre-expiry tick is a no-op for
/// revert. `PhantomBolt` STILL present; `Lifespan.remaining` decremented by
/// exactly `FIXED_DT`; zero `DespawnEntity`.
#[test]
fn phantom_bolt_revert_behavior_pre_expiry_does_not_revert() {
    let mut app = lifespan_emit_app();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            PrimaryBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(10.0, 20.0)),
            PhantomBolt,
            PhantomDamagedCells::default(),
            Lifespan { remaining: 1.0 },
            LifetimeEndBehavior::RevertToNormalBolt,
        ))
        .id();
    app.world_mut()
        .entity_mut(bolt)
        .insert(PhantomDedupKey::Bolt(bolt));

    tick(&mut app);

    assert!(
        app.world().get_entity(bolt).is_ok(),
        "bolt alive before expiry"
    );
    assert!(
        app.world().get::<PhantomBolt>(bolt).is_some(),
        "PhantomBolt still present — revert must not fire pre-expiry"
    );

    let lifespan = app
        .world()
        .get::<Lifespan>(bolt)
        .expect("Lifespan present pre-expiry");
    let expected = 1.0_f32 - FIXED_DT;
    assert!(
        (lifespan.remaining - expected).abs() < 1e-4,
        "Lifespan.remaining should be ≈ {expected} (decremented by FIXED_DT), got {}",
        lifespan.remaining
    );

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(collector.0.is_empty(), "zero DespawnEntity before expiry");
}
