//! Group J — `OnExit(NodeState::Playing)` cleanup via
//! `CleanupOnExit::<NodeState>` (Behaviors J1–J5).
//!
//! Cleanup is delegated to the stateflow handler attached at spawn time.
//! Afterimage does NOT define a custom cleanup system.
//!
//! These tests drive the `Playing → AnimateOut → Teardown` chain so
//! `OnEnter(NodeState::Teardown)` fires `cleanup_on_exit::<NodeState>`.

use bevy::prelude::*;

use super::{
    super::system::PhantomBreaker,
    helpers::{
        build_afterimage_app, drive_to_teardown, phantom_bolt_count, phantom_breaker_count,
        seed_active_protocols_with_afterimage, spawn_phantom_breaker_at, spawn_real_bolt, tick_n,
    },
};
use crate::{
    bolt::components::{LifetimeEndBehavior, PhantomBolt},
    prelude::*,
    shared::{Lifespan, PhantomFlicker},
};

// ── J1 — OnExit(Playing) despawns PhantomBreaker spawned by the system ────
//
// The production system MUST attach `CleanupOnExit::<NodeState>::default()`
// to every PhantomBreaker it spawns. This test drives the spawn through the
// real `afterimage_spawn_phantom_breaker` system (Idle → Dashing edge) and
// asserts the spawned phantom is despawned via the stateflow cleanup
// handler. Under RED (stub spawn system), no phantom is ever spawned —
// precondition fails. Under GREEN (real spawn), the phantom carries
// CleanupOnExit and is correctly despawned on teardown.

#[test]
fn on_exit_playing_despawns_phantom_breaker_spawned_via_production_system() {
    use super::helpers::spawn_breaker_with_dash;
    use crate::breaker::components::DashState;

    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::ZERO);
    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Dashing);
    tick(&mut app);
    assert_eq!(
        phantom_breaker_count(&mut app),
        1,
        "precondition: production spawn system must have produced one PhantomBreaker"
    );

    drive_to_teardown(&mut app);

    assert_eq!(
        phantom_breaker_count(&mut app),
        0,
        "the production-spawned PhantomBreaker MUST be despawned by stateflow cleanup \
         — which requires the production spawn to attach CleanupOnExit::<NodeState>"
    );
}

// ── J1 (edge case) — zero phantoms at exit → no panic, still zero after ──

#[test]
fn zero_phantoms_at_exit_stays_zero_after() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    assert_eq!(phantom_breaker_count(&mut app), 0);

    drive_to_teardown(&mut app);

    assert_eq!(phantom_breaker_count(&mut app), 0);
}

// ── J2 (edge case) — phantom bolt on real bolt entity is despawned on exit ─
//
// Under the W4A mutate-real-bolt design the "phantom bolt" is the real bolt
// entity itself carrying PhantomBolt. When the node exits, CleanupOnExit
// must despawn it just like any other extra bolt carrying the marker.

#[test]
fn phantom_bolt_on_real_bolt_entity_is_despawned_on_exit() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    // Manually install the phantom marker set on a real-bolt-shaped entity,
    // replicating what afterimage_spawn_phantom_bolt does.
    let real_bolt = app
        .world_mut()
        .spawn((
            Bolt,
            PhantomBolt,
            Lifespan { remaining: 3.0 },
            LifetimeEndBehavior::Despawn,
            PhantomFlicker::default(),
            CleanupOnExit::<NodeState>::default(),
        ))
        .id();

    drive_to_teardown(&mut app);

    assert!(
        app.world().get_entity(real_bolt).is_err(),
        "phantom-bolt entity must be despawned on exit via CleanupOnExit"
    );
}

// ── J3 — cleanup runs even when Afterimage is NOT in ActiveProtocols ──────

#[test]
fn cleanup_runs_even_when_afterimage_not_active() {
    let mut app = build_afterimage_app();
    // Do NOT seed ActiveProtocols.
    let phantom_breaker = app
        .world_mut()
        .spawn((PhantomBreaker, CleanupOnExit::<NodeState>::default()))
        .id();
    let phantom_bolt = app
        .world_mut()
        .spawn((
            Bolt,
            PhantomBolt,
            Lifespan { remaining: 3.0 },
            LifetimeEndBehavior::Despawn,
            CleanupOnExit::<NodeState>::default(),
        ))
        .id();

    drive_to_teardown(&mut app);

    assert!(
        app.world().get_entity(phantom_breaker).is_err(),
        "PhantomBreaker must be despawned even when Afterimage is inactive \
         (stateflow cleanup has no protocol_active gate)"
    );
    assert!(
        app.world().get_entity(phantom_bolt).is_err(),
        "phantom bolt must be despawned even when Afterimage is inactive"
    );
}

// ── J4 — re-entry does not resurrect stale phantom state ──────────────────

#[test]
fn re_entry_does_not_resurrect_stale_phantom_state() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 2.0);
    // Spawn a phantom bolt using the W4A mutate-real-bolt shape.
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::ZERO, 10.0, 6.0);
    app.world_mut().entity_mut(real_bolt).insert((
        PhantomBolt,
        Lifespan { remaining: 3.0 },
        LifetimeEndBehavior::Despawn,
        CleanupOnExit::<NodeState>::default(),
    ));

    drive_to_teardown(&mut app);
    assert_eq!(
        phantom_breaker_count(&mut app),
        0,
        "post-exit: zero PhantomBreakers"
    );
    assert_eq!(
        phantom_bolt_count(&mut app),
        0,
        "post-exit: zero phantom bolts"
    );

    // Re-enter Playing.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();
    tick_n(&mut app, 2);

    assert_eq!(
        phantom_breaker_count(&mut app),
        0,
        "re-entry must not resurrect PhantomBreakers"
    );
    assert_eq!(
        phantom_bolt_count(&mut app),
        0,
        "re-entry must not resurrect phantom bolts"
    );
}

// ── J5 — deferred writes do not resurrect state after re-entry ────────────

#[test]
fn deferred_writes_do_not_resurrect_state_after_re_entry() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 2.0);
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::ZERO, 10.0, 6.0);
    app.world_mut().entity_mut(real_bolt).insert((
        PhantomBolt,
        Lifespan { remaining: 3.0 },
        LifetimeEndBehavior::Despawn,
        CleanupOnExit::<NodeState>::default(),
    ));

    drive_to_teardown(&mut app);
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();
    tick_n(&mut app, 4);

    assert_eq!(phantom_breaker_count(&mut app), 0);
    assert_eq!(phantom_bolt_count(&mut app), 0);
}
