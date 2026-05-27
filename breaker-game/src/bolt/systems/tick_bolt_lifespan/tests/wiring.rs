//! Schedule wiring + no-Lifespan no-op tests:
//! Beh 5 — No `Lifespan` component is a no-op
//! Beh 6 — `BoltPlugin` registers `tick_bolt_lifespan` in `FixedUpdate`

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::components::{ExtraBolt, LifetimeEndBehavior},
    prelude::*,
    shared::Lifespan,
    state::types::NodeState,
};

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
