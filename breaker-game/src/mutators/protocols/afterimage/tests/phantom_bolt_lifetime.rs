//! Group G — REUSED `tick_phantom_lifetime` regression guard
//! (Behaviors G1–G4).
//!
//! Afterimage does NOT own a tick system for spawned phantom bolts — it
//! reuses the existing
//! `crate::effect_v3::effects::phantom_bolt::systems::tick_phantom_lifetime`
//! registered by `EffectV3Plugin` via `SpawnPhantomConfig::wire`. These
//! tests pin the OBSERVED behaviour under afterimage's test harness
//! (`build_afterimage_app` brings up the full `effect_v3` wiring).

use bevy::prelude::{Entity, Vec2, With};

use super::helpers::{
    build_afterimage_app, seed_active_protocols_with_afterimage, spawn_phantom_bolt_entity, tick_n,
};
use crate::{
    effect_v3::effects::phantom_bolt::components::{PhantomBolt, PhantomLifetime},
    prelude::*,
};

// ── G1 — spawned phantom's PhantomLifetime.0 decrements each tick ─────────

#[test]
fn spawned_phantom_lifetime_decrements_by_delta_secs_each_tick() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let dummy_owner = app.world_mut().spawn_empty().id();
    let phantom = spawn_phantom_bolt_entity(&mut app, dummy_owner, Vec2::ZERO, Vec2::ZERO, 3.0);

    tick(&mut app);

    let lifetime = app
        .world()
        .get::<PhantomLifetime>(phantom)
        .expect("phantom must still exist after one tick");
    let expected = 3.0 - 1.0 / 64.0;
    assert!(
        (lifetime.0 - expected).abs() < 1e-5,
        "phantom lifetime expected ~{expected}, got {}",
        lifetime.0
    );
}

// ── G1 (edge case) — 32 ticks → lifetime ≈ 2.5 ────────────────────────────

#[test]
fn spawned_phantom_lifetime_after_32_ticks_is_about_2_5() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let dummy_owner = app.world_mut().spawn_empty().id();
    let phantom = spawn_phantom_bolt_entity(&mut app, dummy_owner, Vec2::ZERO, Vec2::ZERO, 3.0);

    tick_n(&mut app, 32);

    let lifetime = app
        .world()
        .get::<PhantomLifetime>(phantom)
        .expect("phantom must still exist at 2.5s remaining");
    assert!(
        (lifetime.0 - 2.5).abs() < 1e-3,
        "phantom lifetime expected ~2.5 after 32 ticks, got {}",
        lifetime.0
    );
}

// ── G2 — phantom ENTITY is despawned when lifetime ≤ 0.0 ──────────────────

#[test]
fn spawned_phantom_entity_despawns_when_lifetime_reaches_zero() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let dummy_owner = app.world_mut().spawn_empty().id();
    let phantom = spawn_phantom_bolt_entity(&mut app, dummy_owner, Vec2::ZERO, Vec2::ZERO, 0.01);

    tick_n(&mut app, 2);

    assert!(
        app.world().get_entity(phantom).is_err(),
        "phantom-bolt entity must be despawned once PhantomLifetime.0 <= 0.0"
    );
}

// ── G2 (edge case) — exactly at threshold despawns on same tick ───────────

#[test]
fn spawned_phantom_exactly_at_threshold_despawns_on_same_tick() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let dummy_owner = app.world_mut().spawn_empty().id();
    let phantom =
        spawn_phantom_bolt_entity(&mut app, dummy_owner, Vec2::ZERO, Vec2::ZERO, 1.0 / 64.0);

    tick(&mut app);

    assert!(
        app.world().get_entity(phantom).is_err(),
        "phantom with lifetime 1/64 must be despawned on the first tick — despawn \
         predicate MUST use `remaining <= 0.0`, not `< 0.0`"
    );
}

// ── G3 — multiple phantom-bolt entities tick independently ────────────────

#[test]
fn multiple_phantom_bolt_entities_tick_independently() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let dummy_a = app.world_mut().spawn_empty().id();
    let dummy_b = app.world_mut().spawn_empty().id();
    let phantom_a = spawn_phantom_bolt_entity(&mut app, dummy_a, Vec2::ZERO, Vec2::ZERO, 3.0);
    let phantom_b = spawn_phantom_bolt_entity(&mut app, dummy_b, Vec2::ZERO, Vec2::ZERO, 0.5);

    tick_n(&mut app, 32);

    let a_lifetime = app
        .world()
        .get::<PhantomLifetime>(phantom_a)
        .expect("phantom_a (3.0s) must still be alive after 32 ticks");
    assert!(
        (a_lifetime.0 - 2.5).abs() < 1e-3,
        "phantom_a lifetime expected ~2.5, got {}",
        a_lifetime.0
    );

    assert!(
        app.world().get_entity(phantom_b).is_err(),
        "phantom_b (0.5s) must be despawned after 32 ticks"
    );
}

// ── G4 — quiet tick safety: no phantom-bolt entities, no panic ────────────

#[test]
fn quiet_tick_safety_no_phantom_bolt_entities_no_panic() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);

    tick_n(&mut app, 5);

    let world = app.world_mut();
    let count = world
        .query_filtered::<Entity, (With<Bolt>, With<PhantomBolt>)>()
        .iter(world)
        .count();
    assert_eq!(
        count, 0,
        "quiet ticks with zero phantom bolts must stay zero"
    );
}
