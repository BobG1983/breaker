//! F2–F6 — conditions that prevent phantom-bolt spawning.

use bevy::prelude::*;

use super::helpers::*;
use crate::{breaker::messages::BumpGrade, prelude::*};

// ── F2 — Early grade does NOT spawn ───────────────────────────────────────

#[test]
fn early_grade_does_not_spawn_phantom_bolt() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    write_bump_performed(&mut app, phantom_breaker, Some(real_bolt), BumpGrade::Early);
    tick(&mut app);

    assert!(
        phantom_bolts_owned_by(&mut app, real_bolt).is_empty(),
        "Early grade must NOT spawn a phantom bolt"
    );
}

// ── F3 — Late grade does NOT spawn ────────────────────────────────────────

#[test]
fn late_grade_does_not_spawn_phantom_bolt() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    write_bump_performed(&mut app, phantom_breaker, Some(real_bolt), BumpGrade::Late);
    tick(&mut app);

    assert!(
        phantom_bolts_owned_by(&mut app, real_bolt).is_empty(),
        "Late grade must NOT spawn a phantom bolt"
    );
}

// ── F4 — breaker is NOT a PhantomBreaker → no spawn ───────────────────────

#[test]
fn non_phantom_breaker_does_not_spawn_phantom_bolt() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (_phantom_breaker, real_bolt) = canonical_setup(&mut app);

    // A real breaker (NOT a PhantomBreaker).
    let real_breaker = app.world_mut().spawn(Breaker).id();
    write_bump_performed(&mut app, real_breaker, Some(real_bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        phantom_bolts_owned_by(&mut app, real_bolt).is_empty(),
        "Perfect bump attributed to a REAL breaker (no PhantomBreaker marker) \
         must NOT spawn a phantom bolt"
    );
}

// ── F4 (edge case) — placeholder / despawned breaker → no spawn, no panic ─

#[test]
fn placeholder_breaker_does_not_spawn_and_does_not_panic() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (_phantom_breaker, real_bolt) = canonical_setup(&mut app);

    write_bump_performed(
        &mut app,
        Entity::PLACEHOLDER,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    assert!(
        phantom_bolts_owned_by(&mut app, real_bolt).is_empty(),
        "placeholder breaker must not spawn a phantom bolt"
    );
}

// ── F5 — bolt: None (spectator) is a no-op ────────────────────────────────

#[test]
fn spectator_bump_with_bolt_none_is_noop() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    write_bump_performed(&mut app, phantom_breaker, None, BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        phantom_bolts_owned_by(&mut app, real_bolt).is_empty(),
        "bolt: None → no phantom-bolt entity spawned for the pre-existing real bolt"
    );
    assert_eq!(
        phantom_bolt_count(&mut app),
        0,
        "bolt: None → zero new phantom-bolt entities in world"
    );
}

// ── F6 — despawned bolt is tolerated ──────────────────────────────────────

#[test]
fn despawned_bolt_is_tolerated() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    // Despawn the real bolt before writing the bump message.
    app.world_mut().entity_mut(real_bolt).despawn();

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    // No panic. No phantom spawned.
    assert!(
        phantom_bolts_owned_by(&mut app, real_bolt).is_empty(),
        "despawned bolt must not result in a phantom-bolt spawn"
    );
}
