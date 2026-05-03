//! Behaviors 4, 6, 7, 8 — spawn lifecycle regression guards covering the
//! at-most-one-phantom invariant, rising-edge gating, continuous Dashing
//! suppression, and the absent-config early-return contract.

use bevy::prelude::*;

use super::{
    super::{
        super::system::AfterimageConfig,
        helpers::{
            build_afterimage_app, build_afterimage_app_no_config,
            seed_active_protocols_with_afterimage,
        },
    },
    drive_rising_edge, override_config_to_2_5, spawn_real_breaker_via_builder,
};
use crate::{
    breaker::components::{DashState, PhantomBreaker},
    prelude::*,
};

// ── Behavior 4 — Despawn-existing invariant preserved (regression guard) ────

/// Pins that the migration preserves the "at most one phantom globally"
/// invariant: when a second rising edge fires, the FIRST phantom is despawned
/// before the new one is spawned.
///
/// This test PASSES at RED — it guards against a migration that drops the
/// despawn-existing loop. If a future writer-code drops the loop, this fails.
#[test]
fn second_rising_edge_despawns_existing_phantom_before_spawning_new_one() {
    let mut app = build_afterimage_app();
    app.init_asset::<Mesh>();
    app.init_asset::<ColorMaterial>();
    override_config_to_2_5(&mut app);
    seed_active_protocols_with_afterimage(&mut app);

    let real = spawn_real_breaker_via_builder(&mut app);

    // First rising edge: Idle → Dashing.
    tick(&mut app);
    *app.world_mut().get_mut::<DashState>(real).unwrap() = DashState::Dashing;
    tick(&mut app);

    // Exactly one phantom after first edge.
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<PhantomBreaker>>()
            .iter(app.world())
            .count(),
        1,
        "exactly one phantom after first rising edge"
    );
    let first_phantom = app
        .world_mut()
        .query_filtered::<Entity, With<PhantomBreaker>>()
        .iter(app.world())
        .next()
        .unwrap();

    // Return to Idle so the next Dashing is a new rising edge.
    *app.world_mut().get_mut::<DashState>(real).unwrap() = DashState::Idle;
    tick(&mut app);

    // Second rising edge.
    *app.world_mut().get_mut::<DashState>(real).unwrap() = DashState::Dashing;
    tick(&mut app);

    // Exactly ONE phantom survives after the second edge.
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<PhantomBreaker>>()
            .iter(app.world())
            .count(),
        1,
        "exactly one phantom must exist after second rising edge (old one despawned)"
    );
    let surviving_phantom = app
        .world_mut()
        .query_filtered::<Entity, With<PhantomBreaker>>()
        .iter(app.world())
        .next()
        .unwrap();

    // The surviving phantom is a NEW entity — the first was despawned.
    assert_ne!(
        surviving_phantom, first_phantom,
        "surviving phantom must be a new entity (first phantom despawned)"
    );

    // Edge case: the real breaker is still alive — the despawn loop must
    // only target With<PhantomBreaker> entities.
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, (With<Breaker>, Without<PhantomBreaker>)>()
            .iter(app.world())
            .count(),
        1,
        "real breaker must still be present (despawn loop must not touch non-phantoms)"
    );
}

// ── Behavior 6 — Rising-edge gates spawn (Idle → Idle = no spawn) ───────────

/// Pins that the system only spawns on the Idle → Dashing rising edge.
/// Ticking twice while Idle must produce no phantom.
///
/// This test PASSES at RED — regression guard that fires if the migration
/// replaces the edge-trigger with "spawn every Dashing tick".
#[test]
fn no_phantom_spawn_on_idle_to_idle_transition() {
    let mut app = build_afterimage_app();
    app.init_asset::<Mesh>();
    app.init_asset::<ColorMaterial>();
    override_config_to_2_5(&mut app);
    seed_active_protocols_with_afterimage(&mut app);

    let _real = spawn_real_breaker_via_builder(&mut app);

    // Two ticks with Idle — no rising edge.
    tick(&mut app);
    tick(&mut app);

    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<PhantomBreaker>>()
            .iter(app.world())
            .count(),
        0,
        "Idle → Idle must NOT spawn a phantom"
    );

    // Edge case: Braking also must not trigger a spawn.
    // Real breaker is still Idle from builder spawn — set Braking and tick.
    let real = app
        .world_mut()
        .query_filtered::<Entity, (With<Breaker>, Without<PhantomBreaker>)>()
        .iter(app.world())
        .next()
        .expect("real breaker present");
    *app.world_mut().get_mut::<DashState>(real).unwrap() = DashState::Braking;
    tick(&mut app);

    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<PhantomBreaker>>()
            .iter(app.world())
            .count(),
        0,
        "Idle → Braking must NOT spawn a phantom (only Idle → Dashing triggers)"
    );
}

// ── Behavior 7 — Continuous Dashing does not respawn (edge fires once) ───────

/// Pins that once the phantom is spawned on the first Idle → Dashing tick,
/// subsequent Dashing ticks do NOT spawn additional phantoms.
///
/// This test PASSES at RED — regression guard.
#[test]
fn continuous_dashing_does_not_respawn_phantom() {
    let mut app = build_afterimage_app();
    app.init_asset::<Mesh>();
    app.init_asset::<ColorMaterial>();
    override_config_to_2_5(&mut app);
    seed_active_protocols_with_afterimage(&mut app);

    let real = spawn_real_breaker_via_builder(&mut app);

    // First tick: records Idle.
    tick(&mut app);

    // Rising edge: spawns phantom A.
    *app.world_mut().get_mut::<DashState>(real).unwrap() = DashState::Dashing;
    tick(&mut app);

    let first_phantom = app
        .world_mut()
        .query_filtered::<Entity, With<PhantomBreaker>>()
        .iter(app.world())
        .next()
        .expect("phantom A must exist after first rising edge");

    // Third tick without changing DashState: still Dashing, no new edge.
    tick(&mut app);

    // Exactly one phantom still; same entity.
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<PhantomBreaker>>()
            .iter(app.world())
            .count(),
        1,
        "continuous Dashing must NOT spawn a second phantom"
    );
    let surviving = app
        .world_mut()
        .query_filtered::<Entity, With<PhantomBreaker>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert_eq!(
        surviving, first_phantom,
        "the surviving phantom must be the same entity as phantom A (no despawn-respawn)"
    );
}

// ── Behavior 8 — No spawn when AfterimageConfig absent ───────────────────────

/// Pins that the system early-returns without spawning when `AfterimageConfig`
/// is absent, even though `Assets<Mesh>` and `Assets<ColorMaterial>` are
/// present. Preserves the harness-safe early-return contract.
///
/// This test PASSES at RED — regression guard.
#[test]
fn phantom_does_not_spawn_when_afterimage_config_absent() {
    let mut app = build_afterimage_app_no_config();
    app.init_asset::<Mesh>();
    app.init_asset::<ColorMaterial>();
    // Deliberately do NOT insert AfterimageConfig.
    seed_active_protocols_with_afterimage(&mut app);

    let real = spawn_real_breaker_via_builder(&mut app);
    drive_rising_edge(&mut app, real);

    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<PhantomBreaker>>()
            .iter(app.world())
            .count(),
        0,
        "absent AfterimageConfig → early return, zero phantoms spawned"
    );

    // app.update() must not panic — the early-return on Option<Res<AfterimageConfig>>
    // succeeds even with Assets<Mesh> and Assets<ColorMaterial> present.
    tick(&mut app);
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<PhantomBreaker>>()
            .iter(app.world())
            .count(),
        0,
        "still zero phantoms after additional tick with absent config"
    );
}

/// Edge case: the system does NOT update `prev_state` when `AfterimageConfig`
/// is absent, so inserting the config mid-test and continuing to tick with
/// `DashState::Dashing` already set fires the rising edge on the FIRST
/// post-config tick (because `prev_state` is `None`, so `was_dashing ==
/// false`).
#[test]
fn phantom_does_not_spawn_when_afterimage_config_absent_until_inserted() {
    let mut app = build_afterimage_app_no_config();
    app.init_asset::<Mesh>();
    app.init_asset::<ColorMaterial>();
    seed_active_protocols_with_afterimage(&mut app);

    let real = spawn_real_breaker_via_builder(&mut app);

    // Drive rising edge while config is absent — no spawn, prev_state stays None.
    drive_rising_edge(&mut app, real);

    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<PhantomBreaker>>()
            .iter(app.world())
            .count(),
        0,
        "absent config: no spawn even on rising edge"
    );

    // Now insert config with breaker still Dashing.
    app.insert_resource(AfterimageConfig {
        phantom_duration:      2.5,
        phantom_bolt_duration: 1.0,
    });

    // First post-config tick: prev_state is None → was_dashing == false →
    // is_rising == true (because current == Dashing). Phantom spawns.
    tick(&mut app);

    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<PhantomBreaker>>()
            .iter(app.world())
            .count(),
        1,
        "first post-config tick with Dashing must spawn phantom (prev_state was None)"
    );
}
