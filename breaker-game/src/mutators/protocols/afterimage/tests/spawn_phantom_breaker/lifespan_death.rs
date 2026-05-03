//! Lifespan/death behaviors for `afterimage_spawn_phantom_breaker` —
//! despawned-breaker tolerance, harness-safe early-return when
//! `AfterimageConfig` is absent, and the lifespan tick → `DespawnEntity` flow.

use bevy::prelude::*;
use rantzsoft_dmg::RantzDmgPlugin;

use super::super::{
    super::system::PhantomBreaker,
    helpers::{
        build_afterimage_app, build_afterimage_app_no_config, phantom_breaker_count,
        seed_active_protocols_with_afterimage, spawn_breaker_with_dash, spawn_phantom_breaker_at,
        tick_n,
    },
};
use crate::{
    breaker::{
        components::{BaseHeight, BaseWidth, DashState},
        systems::tick_phantom_breaker_lifespan,
    },
    prelude::*,
    shared::phantom::Lifespan,
};

// ── C6 — despawned breaker during Dashing is tolerated ─────────────────────

#[test]
fn despawned_breaker_during_dashing_is_tolerated() {
    let mut app = build_afterimage_app_with_death_pipeline();
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

// ── Behavior #11 — phantom spawned with Lifespan despawns within duration ──

fn build_afterimage_app_with_death_pipeline() -> App {
    let mut app = build_afterimage_app();
    app.add_plugins(RantzDmgPlugin);
    app.add_systems(
        FixedUpdate,
        tick_phantom_breaker_lifespan.run_if(in_state(NodeState::Playing)),
    );
    app
}

/// Regression guard: phantom spawned via builder (Wave 4C) carries `Lifespan`
/// and is despawned by `tick_phantom_breaker_lifespan` + `process_despawn_requests`
/// within `phantom_duration` seconds. PASSES TODAY — guards against Wave 5
/// deletions accidentally breaking the end-to-end spawn → tick → despawn flow.
#[test]
fn phantom_spawned_with_lifespan_despawns_within_phantom_duration() {
    let mut app = build_afterimage_app_with_death_pipeline();
    seed_active_protocols_with_afterimage(&mut app);
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::new(100.0, 50.0));
    app.world_mut()
        .entity_mut(breaker)
        .insert((BaseWidth(120.0), BaseHeight(20.0)));

    // Idle → Dashing triggers spawn.
    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Dashing);
    tick(&mut app);

    // After the first tick: exactly one PhantomBreaker with Lifespan exists.
    let world = app.world_mut();
    let mut q = world.query_filtered::<(Entity, &Lifespan), With<PhantomBreaker>>();
    let rows: Vec<(Entity, f32)> = q.iter(world).map(|(e, l)| (e, l.remaining)).collect();
    assert_eq!(
        rows.len(),
        1,
        "exactly one PhantomBreaker must exist after Idle → Dashing, got {}",
        rows.len()
    );
    // The phantom is spawned via Commands during the same tick — the entity
    // does not exist when tick_phantom_breaker_lifespan runs in that update.
    // The first decrement happens on the NEXT tick, so remaining is still 2.0.
    assert!(
        (rows[0].1 - 2.0).abs() < 1e-3,
        "Lifespan.remaining after the spawn tick must be ~2.0 (not yet decremented), got {:.4}",
        rows[0].1
    );

    // Tick 130 times (~2.03 s at 64 Hz) — phantom must despawn via DespawnEntity.
    tick_n(&mut app, 130);

    assert_eq!(
        phantom_breaker_count(&mut app),
        0,
        "phantom must be despawned after ~phantom_duration (2.0 s) via Lifespan → DespawnEntity"
    );
}

/// Edge case: real breaker with no `BaseWidth`/`BaseHeight` — phantom still
/// spawns and eventually despawns (no panic, no missing required component).
#[test]
fn phantom_spawned_without_basewidth_baseheight_uses_defaults() {
    let mut app = build_afterimage_app_with_death_pipeline();
    seed_active_protocols_with_afterimage(&mut app);
    // Real breaker WITHOUT BaseWidth / BaseHeight.
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::new(0.0, 0.0));

    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Dashing);
    tick(&mut app);

    assert_eq!(
        phantom_breaker_count(&mut app),
        1,
        "phantom must spawn even when real breaker has no BaseWidth/BaseHeight"
    );

    // Tick long enough for the phantom to despawn.
    tick_n(&mut app, 130);

    assert_eq!(
        phantom_breaker_count(&mut app),
        0,
        "phantom must despawn within phantom_duration even without BaseWidth/BaseHeight on spawner"
    );
}
