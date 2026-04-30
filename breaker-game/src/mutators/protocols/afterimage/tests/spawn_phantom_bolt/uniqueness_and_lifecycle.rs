//! F7–F10 — uniqueness, phantom outliving real bolt, config absent, gate.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    breaker::messages::BumpGrade,
    effect_v3::effects::phantom_bolt::components::{PhantomLifetime, PhantomOwner},
    prelude::*,
};

// ── F7 — one-phantom-per-owner uniqueness (no re-spawn, no duration reset) ─

#[test]
fn second_perfect_bump_does_not_spawn_second_phantom_or_reset_duration() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    // Pre-seed a phantom-bolt entity for real_bolt with lifetime 1.0.
    let existing =
        spawn_phantom_bolt_entity(&mut app, real_bolt, Vec2::ZERO, Vec2::new(0.0, 400.0), 1.0);

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    let owned = phantom_bolts_owned_by(&mut app, real_bolt);
    assert_eq!(
        owned.len(),
        1,
        "uniqueness: only ONE phantom-bolt entity for real_bolt after second Perfect bump"
    );
    assert_eq!(owned[0], existing, "the pre-existing phantom must survive");
    let lifetime = app.world().get::<PhantomLifetime>(existing).unwrap().0;
    let expected = 1.0 - 1.0 / 64.0;
    assert!(
        (lifetime - expected).abs() < 1e-4,
        "existing phantom lifetime must NOT be reset to 3.0 — expected ~{expected}, got {lifetime}"
    );
}

// ── F7 (edge case) — 2.99 existing lifetime stays near 2.99, not 3.0 ──────

#[test]
fn second_perfect_bump_with_2_99_existing_lifetime_does_not_reset() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);
    let existing =
        spawn_phantom_bolt_entity(&mut app, real_bolt, Vec2::ZERO, Vec2::new(0.0, 400.0), 2.99);

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    let lifetime = app.world().get::<PhantomLifetime>(existing).unwrap().0;
    let expected = 2.99 - 1.0 / 64.0;
    assert!(
        (lifetime - expected).abs() < 1e-4,
        "existing lifetime 2.99 must decrement by 1/64, NOT reset to 3.0 — \
         expected ~{expected}, got {lifetime}"
    );
}

// ── F8 — phantom outlives real bolt (stale PhantomOwner) ──────────────────

#[test]
fn phantom_outlives_real_bolt_with_stale_phantom_owner() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    let owned = phantom_bolts_owned_by(&mut app, real_bolt);
    assert_eq!(owned.len(), 1);
    let phantom = owned[0];

    // Despawn the real bolt.
    app.world_mut().entity_mut(real_bolt).despawn();
    tick_n(&mut app, 3);

    assert!(
        app.world().get_entity(phantom).is_ok(),
        "phantom must still exist even though its owner is dead"
    );
    assert_eq!(
        app.world().get::<PhantomOwner>(phantom).unwrap().0,
        real_bolt,
        "PhantomOwner must retain the stale Entity reference"
    );
    assert!(
        app.world().get_entity(real_bolt).is_err(),
        "real bolt must be dead"
    );
    let lifetime = app.world().get::<PhantomLifetime>(phantom).unwrap().0;
    // At spawn lifetime is ~3.0; after 1 (same-tick) + 3 ticks = 4 total
    // ticks, lifetime ≈ 3.0 - 4/64.
    let expected_upper = 3.0 - 3.0 / 64.0;
    let expected_lower = 3.0 - 4.0 / 64.0;
    assert!(
        lifetime < expected_upper + 1e-4 && lifetime > expected_lower - 1e-4,
        "phantom lifetime must have decremented ~4/64 via tick_phantom_lifetime, got {lifetime}"
    );
}

// ── F9 — harness-safe: AfterimageConfig absent drains reader ──────────────

#[test]
fn absent_afterimage_config_drains_reader_does_not_retro_spawn() {
    let mut app = build_afterimage_app_no_config();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom_breaker = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 10.0, 6.0);

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);
    assert!(
        phantom_bolts_owned_by(&mut app, real_bolt).is_empty(),
        "tick 1 with absent AfterimageConfig → no spawn"
    );

    // Insert the config; tick 2 — the message has already been drained.
    app.world_mut()
        .insert_resource(super::super::super::system::AfterimageConfig {
            phantom_duration:      2.0,
            phantom_bolt_duration: 3.0,
        });
    tick(&mut app);

    assert!(
        phantom_bolts_owned_by(&mut app, real_bolt).is_empty(),
        "tick 2 must NOT retro-apply the drained BumpPerformed message"
    );
}

// ── F10 — ActiveProtocols missing Afterimage → no spawn (run_if gate) ─────

#[test]
fn active_protocols_missing_afterimage_does_not_spawn() {
    let mut app = build_afterimage_app();
    // Do NOT seed ActiveProtocols.
    let phantom_breaker = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 10.0, 6.0);

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    assert!(
        phantom_bolts_owned_by(&mut app, real_bolt).is_empty(),
        "run_if gate: no Afterimage in ActiveProtocols → no phantom-bolt spawn"
    );
}
