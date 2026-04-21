//! Group C — `afterimage_spawn_phantom_breaker` (Behaviors C1–C7).
//!
//! Pins the Idle → Dashing edge-trigger, steady-state suppression, re-dash
//! despawn/respawn, non-Dashing transitions, despawned-breaker tolerance,
//! and the harness-safe early-return when `AfterimageConfig` is absent.

use bevy::prelude::*;
use rantzsoft_stateflow::CleanupOnExit;

use super::{
    super::system::{PhantomBreaker, PhantomBreakerLifetime},
    helpers::{
        build_afterimage_app, build_afterimage_app_no_config, phantom_breaker_count,
        seed_active_protocols_with_afterimage, spawn_breaker_with_dash, spawn_phantom_breaker_at,
        tick_n,
    },
};
use crate::{
    breaker::components::{BaseHeight, BaseWidth, DashState},
    prelude::*,
};

// ── C1 — Idle → Dashing spawns one PhantomBreaker at breaker's position ────

#[test]
fn idle_to_dashing_spawns_one_phantom_breaker_at_breaker_position() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::new(100.0, 50.0));

    // Transition the breaker to Dashing.
    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Dashing);
    tick(&mut app);

    let world = app.world_mut();
    let mut q = world.query_filtered::<
        (Entity, &PhantomBreakerLifetime, &Position2D),
        (With<PhantomBreaker>, Without<Breaker>),
    >();
    let rows: Vec<(Entity, PhantomBreakerLifetime, Vec2)> =
        q.iter(world).map(|(e, l, p)| (e, *l, p.0)).collect();
    assert_eq!(
        rows.len(),
        1,
        "Idle → Dashing must spawn EXACTLY one PhantomBreaker, got {}",
        rows.len()
    );
    assert!(
        (rows[0].1.0 - 2.0).abs() < 1e-4,
        "PhantomBreakerLifetime expected 2.0 (phantom_duration), got {}",
        rows[0].1.0
    );
    assert!(
        (rows[0].2 - Vec2::new(100.0, 50.0)).length() < 1e-4,
        "phantom spawned at breaker position (100.0, 50.0), got {:?}",
        rows[0].2
    );
    // CleanupOnExit attached at spawn.
    assert!(
        app.world()
            .get::<CleanupOnExit<NodeState>>(rows[0].0)
            .is_some(),
        "spawned PhantomBreaker must carry CleanupOnExit::<NodeState> for \
         stateflow node-exit cleanup"
    );
}

// ── C1 (edge case) — breaker entity must NOT receive PhantomBreaker ────────

#[test]
fn breaker_itself_does_not_receive_phantom_breaker_marker() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::new(100.0, 50.0));

    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Dashing);
    tick(&mut app);

    let world = app.world_mut();
    let mut q = world.query_filtered::<(), (With<Breaker>, With<PhantomBreaker>)>();
    assert_eq!(
        q.iter(world).count(),
        0,
        "the real breaker entity must NOT be marked with PhantomBreaker"
    );
}

// ── C2 — steady-state Dashing does NOT spawn additional PhantomBreakers ────

#[test]
fn steady_state_dashing_does_not_spawn_additional_phantom_breakers() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    // Breaker already Dashing with a pre-seeded PhantomBreaker.
    let _breaker = spawn_breaker_with_dash(&mut app, DashState::Dashing, Vec2::new(100.0, 50.0));
    spawn_phantom_breaker_at(&mut app, Vec2::new(-50.0, 50.0), 1.5);

    tick(&mut app);
    tick(&mut app);

    assert_eq!(
        phantom_breaker_count(&mut app),
        1,
        "edge-trigger: spawn must NOT fire while DashState stays == Dashing"
    );
}

// ── C2 (edge case) — starting in Dashing with no prior phantom → spawn one ─

#[test]
fn starting_in_dashing_with_no_prior_phantom_spawns_one_on_first_tick() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _breaker = spawn_breaker_with_dash(&mut app, DashState::Dashing, Vec2::new(100.0, 50.0));

    tick(&mut app);

    assert_eq!(
        phantom_breaker_count(&mut app),
        1,
        "first tick with breaker starting in Dashing must spawn EXACTLY one PhantomBreaker"
    );
}

// ── C3 — re-dash despawns old PhantomBreaker, spawns new one at new pos ────

#[test]
fn redash_despawns_old_phantom_breaker_and_spawns_new_one() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let old_phantom = spawn_phantom_breaker_at(&mut app, Vec2::new(-50.0, 50.0), 1.0);
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::new(200.0, 50.0));

    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Dashing);
    tick(&mut app);

    assert_eq!(
        phantom_breaker_count(&mut app),
        1,
        "after re-dash there must be EXACTLY one PhantomBreaker"
    );
    let world = app.world_mut();
    let mut q =
        world.query_filtered::<(&PhantomBreakerLifetime, &Position2D), With<PhantomBreaker>>();
    let (lifetime, position) = q.iter(world).map(|(l, p)| (*l, p.0)).next().unwrap();
    assert!(
        (position - Vec2::new(200.0, 50.0)).length() < 1e-4,
        "new PhantomBreaker must spawn at the NEW breaker position (200.0, 50.0), got {position:?}"
    );
    assert!(
        (lifetime.0 - 2.0).abs() < 1e-4,
        "new PhantomBreaker lifetime must be reset to phantom_duration (2.0), got {}",
        lifetime.0
    );

    // Old phantom entity is no longer alive.
    assert!(
        app.world().get::<PhantomBreaker>(old_phantom).is_none(),
        "old PhantomBreaker entity must be despawned after re-dash"
    );
    assert!(
        app.world().get_entity(old_phantom).is_err(),
        "old PhantomBreaker Entity ID must be recycled / dead"
    );
}

// ── C4 — Idle → Settling does NOT spawn a PhantomBreaker ───────────────────

#[test]
fn idle_to_settling_does_not_spawn_phantom_breaker() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::new(0.0, 0.0));

    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Settling);
    tick(&mut app);

    assert_eq!(
        phantom_breaker_count(&mut app),
        0,
        "Idle → Settling must NOT spawn a PhantomBreaker (only Dashing triggers)"
    );
}

// ── C4 (edge case) — Idle → Braking must NOT spawn either ──────────────────

#[test]
fn idle_to_braking_does_not_spawn_phantom_breaker() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::new(0.0, 0.0));

    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Braking);
    tick(&mut app);

    assert_eq!(
        phantom_breaker_count(&mut app),
        0,
        "Idle → Braking must NOT spawn a PhantomBreaker"
    );
}

// ── C5 — Dashing → Braking does NOT spawn a new PhantomBreaker ─────────────

#[test]
fn dashing_to_braking_does_not_spawn_new_phantom_breaker() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Dashing, Vec2::new(0.0, 0.0));
    spawn_phantom_breaker_at(&mut app, Vec2::new(-50.0, 0.0), 1.5);

    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Braking);
    tick(&mut app);

    assert_eq!(
        phantom_breaker_count(&mut app),
        1,
        "Dashing → Braking must NOT spawn a second PhantomBreaker — count stays 1"
    );
}

// ── C6 — despawned breaker during Dashing is tolerated ─────────────────────

#[test]
fn despawned_breaker_during_dashing_is_tolerated() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Dashing, Vec2::new(0.0, 0.0));
    let _phantom = spawn_phantom_breaker_at(&mut app, Vec2::new(0.0, 0.0), 0.1);

    // Despawn the real breaker.
    app.world_mut().entity_mut(breaker).despawn();

    // Two ticks — phantom's lifetime should tick down; no panics.
    tick(&mut app);
    tick(&mut app);

    // 0.1s - 2 * 1/64 = ~0.069 → still alive.
    // Keep ticking until expiry; assert no panic & eventual despawn.
    tick_n(&mut app, 20);
    assert_eq!(
        phantom_breaker_count(&mut app),
        0,
        "phantom must eventually despawn even after its breaker is gone"
    );
}

// ── C7 — harness-safe: AfterimageConfig absent → no spawn, no panic ────────

#[test]
fn harness_safe_afterimage_config_absent_does_not_spawn() {
    let mut app = build_afterimage_app_no_config();
    seed_active_protocols_with_afterimage(&mut app);
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::new(0.0, 0.0));

    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Dashing);
    tick(&mut app);

    assert_eq!(
        phantom_breaker_count(&mut app),
        0,
        "absent AfterimageConfig → early return, zero PhantomBreaker spawned"
    );

    // A second quiet tick must not retro-fire.
    tick(&mut app);
    assert_eq!(
        phantom_breaker_count(&mut app),
        0,
        "absent AfterimageConfig → second tick still zero PhantomBreaker"
    );
}

// ── Phantom inherits BaseWidth/BaseHeight from real breaker when present ──

#[test]
fn phantom_breaker_inherits_base_width_and_height_from_real_breaker_when_present() {
    // Given: a real breaker carrying non-default BaseWidth(123.0) and
    // BaseHeight(45.0). Driving Idle → Dashing spawns a PhantomBreaker that
    // must COPY those raw values (not the 100.0 / 20.0 fallbacks).
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::ZERO);
    app.world_mut()
        .entity_mut(breaker)
        .insert((BaseWidth(123.0), BaseHeight(45.0)));

    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Dashing);
    tick(&mut app);

    let world = app.world_mut();
    let mut q = world
        .query_filtered::<(&BaseWidth, &BaseHeight), (With<PhantomBreaker>, Without<Breaker>)>();
    let rows: Vec<(f32, f32)> = q.iter(world).map(|(w, h)| (w.0, h.0)).collect();
    assert_eq!(
        rows.len(),
        1,
        "exactly one PhantomBreaker must exist after Idle → Dashing"
    );
    assert!(
        (rows[0].0 - 123.0).abs() < f32::EPSILON,
        "phantom BaseWidth must inherit the real breaker's 123.0 (NOT the \
         100.0 fallback), got {}",
        rows[0].0
    );
    assert!(
        (rows[0].1 - 45.0).abs() < f32::EPSILON,
        "phantom BaseHeight must inherit the real breaker's 45.0 (NOT the \
         20.0 fallback), got {}",
        rows[0].1
    );
}
