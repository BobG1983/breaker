//! Tests for `tick_bolt_lifespan` system — Wave 3A behavioral contract.
//!
//! All 17 tests MUST FAIL against the RED-phase stub. GREEN gate requires the
//! production implementation to make all 17 pass.

use bevy::prelude::*;

use super::system::tick_bolt_lifespan;
use crate::{
    bolt::{
        BoltPlugin,
        components::{
            BoltBaseDamage, ExtraBolt, LifetimeEndBehavior, PhantomBolt, PhantomDamagedCells,
            PhantomDedupKey, PiercingRemaining, PrimaryBolt,
        },
    },
    prelude::*,
    shared::{Lifespan, PhantomFlicker, birthing::BIRTHING_DURATION},
    state::{
        run::resources::NodeOutcome,
        types::{NodeState, RunState},
    },
};

// ── Constants ────────────────────────────────────────────────────────────────

const FIXED_DT: f32 = 1.0 / 64.0;

// ── KillYourself<Bolt> regression-guard capture ──────────────────────────────

#[derive(Resource, Default)]
struct CapturedKillYourselfBolt(Vec<KillYourself<Bolt>>);

fn capture_kill_yourself_bolt(
    mut reader: MessageReader<KillYourself<Bolt>>,
    mut captured: ResMut<CapturedKillYourselfBolt>,
) {
    for m in reader.read() {
        captured.0.push(m.clone());
    }
}

// ── Test app helpers ──────────────────────────────────────────────────────────

/// Emit-only app: `tick_bolt_lifespan` in `FixedUpdate` with `DespawnEntity` capture
/// and `KillYourself<Bolt>` capture (so regression-guard "zero" assertions work).
/// No `RantzDmgPlugin` — proves the system emits but does NOT despawn directly.
fn lifespan_emit_app() -> App {
    TestAppBuilder::new()
        .with_message_capture::<DespawnEntity>()
        .with_message::<KillYourself<Bolt>>()
        .with_resource::<CapturedKillYourselfBolt>()
        .with_system(
            FixedUpdate,
            (tick_bolt_lifespan, capture_kill_yourself_bolt).chain(),
        )
        .build()
}

/// End-to-end app: `with_dmg_pipeline()` and `tick_bolt_lifespan` in `FixedUpdate`
/// with `DespawnEntity` capture and `KillYourself<Bolt>` capture.
/// Proves expiry → `DespawnEntity` → `process_despawn_requests` within one tick.
fn lifespan_despawn_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_dmg_pipeline()
        .with_message::<KillYourself<Bolt>>()
        .with_resource::<CapturedKillYourselfBolt>()
        .build();
    app.add_systems(
        FixedUpdate,
        tick_bolt_lifespan.before(DmgSystems::ApplyKill),
    );
    app.add_systems(FixedUpdate, capture_kill_yourself_bolt);
    attach_message_capture::<DespawnEntity>(&mut app);
    app
}

/// Full plugin-wiring app for Behavior 6 — mirrors `scheduling_test_app()` at
/// `bolt/systems/bolt_cell_collision/tests/scheduling.rs:40-54`.
fn bolt_plugin_wiring_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_physics()
        .with_playfield()
        .with_bolt_registry()
        .with_breaker_registry()
        .with_cell_registry()
        .with_resource::<crate::input::resources::InputActions>()
        .with_resource::<NodeOutcome>()
        .with_effects_pipeline()
        .build();
    app.add_plugins(BoltPlugin);
    attach_message_capture::<DespawnEntity>(&mut app);
    app
}

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

// ── Behavior 3 — T16: default Despawn for non-phantom bolt ──────────────────

/// T16 primary: non-phantom bolt with `Lifespan` and NO `LifetimeEndBehavior`
/// defaults to the Despawn branch — emits one `DespawnEntity`.
#[test]
fn non_phantom_bolt_no_behavior_defaults_to_despawn_entity() {
    let mut app = lifespan_emit_app();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, 0.0)),
            Lifespan { remaining: 0.01 },
        ))
        .id();

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        collector.0.len(),
        1,
        "no LifetimeEndBehavior defaults to Despawn; expected one DespawnEntity, got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].entity, bolt);

    let ky = app.world().resource::<CapturedKillYourselfBolt>();
    assert!(ky.0.is_empty(), "zero KillYourself<Bolt>");

    let lifespan = app
        .world()
        .get::<Lifespan>(bolt)
        .expect("Lifespan still on entity");
    let expected = 0.01_f32 - FIXED_DT;
    assert!(
        (lifespan.remaining - expected).abs() < 1e-4,
        "Lifespan.remaining ≈ {expected}, got {}",
        lifespan.remaining
    );

    assert!(
        app.world().get::<Bolt>(bolt).is_some(),
        "Bolt marker still present (system does not strip)"
    );
}

/// T16 edge 3a: `Lifespan { remaining: 0.0 }` boundary, no `LifetimeEndBehavior`
/// — exactly one `DespawnEntity` emitted.
#[test]
fn non_phantom_bolt_at_zero_boundary_defaults_to_despawn_entity() {
    let mut app = lifespan_emit_app();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, 0.0)),
            Lifespan { remaining: 0.0 },
        ))
        .id();

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        collector.0.len(),
        1,
        "zero-boundary with no behavior must emit one DespawnEntity"
    );
    assert_eq!(collector.0[0].entity, bolt);
}

/// T16 edge 3b: `Lifespan { remaining: 1.0 }`, no `LifetimeEndBehavior` —
/// pre-expiry tick is a no-op; `Lifespan.remaining` decremented, zero
/// `DespawnEntity`.
#[test]
fn non_phantom_bolt_pre_expiry_no_behavior_is_noop() {
    let mut app = lifespan_emit_app();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, 0.0)),
            Lifespan { remaining: 1.0 },
        ))
        .id();

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(collector.0.is_empty(), "no DespawnEntity before expiry");
    assert!(app.world().get_entity(bolt).is_ok());

    let lifespan = app.world().get::<Lifespan>(bolt).expect("Lifespan present");
    let expected = 1.0_f32 - FIXED_DT;
    assert!(
        (lifespan.remaining - expected).abs() < 1e-4,
        "Lifespan.remaining ≈ {expected}, got {}",
        lifespan.remaining
    );
}

/// T16 edge 3c: two bolts — one expires, one does not. Only the short-lifespan
/// bolt emits `DespawnEntity`.
#[test]
fn two_bolts_mixed_expiry_only_short_lifespan_emits_despawn_entity() {
    let mut app = lifespan_emit_app();
    let short_bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, 0.0)),
            Lifespan { remaining: 0.01 },
        ))
        .id();
    let long_bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(10.0, 0.0)),
            Lifespan { remaining: 1.0 },
        ))
        .id();

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        collector.0.len(),
        1,
        "exactly one DespawnEntity for two bolts with different lifespans"
    );
    assert_eq!(
        collector.0[0].entity, short_bolt,
        "DespawnEntity must target the short-lifespan bolt"
    );

    assert!(
        app.world().get_entity(long_bolt).is_ok(),
        "long-lifespan bolt still alive"
    );
    let long_lifespan = app
        .world()
        .get::<Lifespan>(long_bolt)
        .expect("Lifespan present");
    let expected = 1.0_f32 - FIXED_DT;
    assert!(
        (long_lifespan.remaining - expected).abs() < 1e-4,
        "long bolt Lifespan.remaining ≈ {expected}, got {}",
        long_lifespan.remaining
    );
}

// ── Behavior 4 — Birthing filter preserved ───────────────────────────────────

fn test_birthing() -> Birthing {
    Birthing {
        timer:          Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
        target_scale:   Scale2D { x: 8.0, y: 8.0 },
        stashed_layers: CollisionLayers::default(),
    }
}

/// Behavior 4 primary: birthing bolt with expired `Lifespan` emits ZERO
/// `DespawnEntity`; `Lifespan.remaining` is UNCHANGED (system skips entirely).
#[test]
fn tick_bolt_lifespan_skips_birthing_bolt_entirely() {
    let mut app = lifespan_emit_app();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::ZERO),
            Lifespan { remaining: 0.01 },
            LifetimeEndBehavior::Despawn,
            test_birthing(),
        ))
        .id();

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(
        collector.0.is_empty(),
        "birthing bolt must emit zero DespawnEntity"
    );

    let lifespan = app.world().get::<Lifespan>(bolt).expect("Lifespan present");
    assert!(
        (lifespan.remaining - 0.01).abs() < f32::EPSILON,
        "Lifespan.remaining must be UNCHANGED for birthing bolt; got {}",
        lifespan.remaining
    );
}

/// Behavior 4 edge 4a: birthing + non-birthing coexist — only non-birthing bolt
/// emits `DespawnEntity`; birthing bolt's `Lifespan.remaining` unchanged.
#[test]
fn tick_bolt_lifespan_skips_birthing_processes_non_birthing() {
    let mut app = lifespan_emit_app();
    let birthing_bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::ZERO),
            Lifespan { remaining: 0.01 },
            LifetimeEndBehavior::Despawn,
            test_birthing(),
        ))
        .id();
    let normal_bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(5.0, 0.0)),
            Lifespan { remaining: 0.01 },
            LifetimeEndBehavior::Despawn,
        ))
        .id();

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        collector.0.len(),
        1,
        "exactly one DespawnEntity — the non-birthing bolt"
    );
    assert_eq!(
        collector.0[0].entity, normal_bolt,
        "DespawnEntity must be for the non-birthing bolt"
    );

    let birthing_lifespan = app
        .world()
        .get::<Lifespan>(birthing_bolt)
        .expect("Lifespan present on birthing bolt");
    assert!(
        (birthing_lifespan.remaining - 0.01).abs() < f32::EPSILON,
        "birthing bolt Lifespan.remaining must be UNCHANGED; got {}",
        birthing_lifespan.remaining
    );
}

// ── Behavior 5 — No Lifespan component is a no-op ────────────────────────────

/// Behavior 5: bolt with NO `Lifespan` emits zero `DespawnEntity`; entity
/// stays alive; system must NOT insert `Lifespan`.
#[test]
fn bolt_without_lifespan_system_is_noop() {
    let mut app = lifespan_emit_app();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::ZERO),
        ))
        .id();

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(
        collector.0.is_empty(),
        "no DespawnEntity for bolt without Lifespan"
    );
    assert!(app.world().get_entity(bolt).is_ok(), "entity still alive");
    assert!(
        app.world().get::<Lifespan>(bolt).is_none(),
        "system must NOT insert Lifespan on a bolt that had none"
    );
}

// ── Behavior 6 — Plugin wiring: BoltPlugin registers tick_bolt_lifespan ──────

/// Behavior 6 primary: `BoltPlugin` registers `tick_bolt_lifespan` in
/// `FixedUpdate`; a bolt with expired `Lifespan` is despawned end-to-end via
/// the unified pipeline within one tick.
#[test]
fn bolt_plugin_registers_tick_bolt_lifespan_in_fixed_update() {
    let mut app = bolt_plugin_wiring_app();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::ZERO),
            Lifespan { remaining: 0.01 },
            LifetimeEndBehavior::Despawn,
        ))
        .id();

    tick(&mut app);

    assert!(
        app.world().get_entity(bolt).is_err(),
        "BoltPlugin must register tick_bolt_lifespan; bolt with Lifespan 0.01 \
         must be despawned end-to-end after one tick in NodeState::Playing"
    );

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(collector.0.len(), 1, "exactly one DespawnEntity recorded");
    assert_eq!(collector.0[0].entity, bolt);
}

/// Behavior 6 edge 6a: two-phase state-gating test.
///
/// Phase 1 — tick while in `NodeState::Playing`: `Lifespan.remaining` must
/// decrease (RED-phase stub fails here — no-op stub leaves remaining unchanged).
/// Phase 2 — exit Playing, then tick again: `Lifespan.remaining` must not
/// change (gate respected).
///
/// Modeled verbatim on `tick_phantom_lifespan_runs_in_playing_not_outside` at
/// `breaker-game/src/breaker/systems/tick_phantom_breaker_lifespan/tests/wiring.rs`.
#[test]
fn tick_bolt_lifespan_runs_in_playing_not_outside() {
    let mut app = bolt_plugin_wiring_app();

    let persistent_bolt = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::ZERO),
            Lifespan { remaining: 1.0 },
            LifetimeEndBehavior::Despawn,
        ))
        .id();

    // Phase 1 — tick while in Playing.
    tick(&mut app);

    let after_playing = app
        .world()
        .get::<Lifespan>(persistent_bolt)
        .expect("Lifespan present after Playing tick")
        .remaining;
    assert!(
        after_playing < 1.0,
        "BoltPlugin must register tick_bolt_lifespan; \
         Lifespan.remaining must decrease during Playing tick; got {after_playing} \
         (no-op stub fails here)"
    );

    // Phase 2 — exit Playing then tick.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Loading);
    app.update();

    assert!(
        app.world().get_entity(persistent_bolt).is_ok(),
        "persistent bolt must survive state transition (no CleanupOnExit<NodeState>)"
    );

    tick(&mut app);

    let after_transition = app
        .world()
        .get::<Lifespan>(persistent_bolt)
        .expect("Lifespan present after transition tick")
        .remaining;
    assert!(
        (after_transition - after_playing).abs() < 1e-6,
        "Lifespan.remaining must not change after exiting Playing \
         (run_if(in_state(NodeState::Playing)) gate must be respected); \
         after_playing={after_playing}, after_transition={after_transition}"
    );

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(
        collector.0.is_empty(),
        "no DespawnEntity must be emitted — bolt lifespan stayed positive in both phases"
    );
}
