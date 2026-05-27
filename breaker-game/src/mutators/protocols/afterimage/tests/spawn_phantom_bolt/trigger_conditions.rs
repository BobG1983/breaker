//! T19 — conditions that prevent the real bolt from being mutated into a phantom.
//!
//! F2–F6 rewritten for the mutate-real-bolt design: assertions change from
//! "no new phantom-bolt entity spawned" to "real bolt does NOT gain `PhantomBolt`."

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::components::{LifetimeEndBehavior, PhantomBolt, PhantomDamagedCells, PhantomDedupKey},
    breaker::messages::BumpGrade,
    prelude::*,
    shared::{Lifespan, PhantomFlicker},
};

// ── T19 — Early grade does NOT mutate the real bolt ───────────────────────────

#[test]
fn early_grade_does_not_mutate_real_bolt() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    write_bump_performed(&mut app, phantom_breaker, Some(real_bolt), BumpGrade::Early);
    tick(&mut app);

    assert!(
        app.world().get::<PhantomBolt>(real_bolt).is_none(),
        "Early grade must NOT add PhantomBolt to real bolt"
    );
    assert!(
        app.world().get::<PhantomDedupKey>(real_bolt).is_none(),
        "Early grade must NOT add PhantomDedupKey to real bolt"
    );
    assert!(
        app.world().get::<PhantomDamagedCells>(real_bolt).is_none(),
        "Early grade must NOT add PhantomDamagedCells to real bolt"
    );
    assert!(
        app.world().get::<Lifespan>(real_bolt).is_none(),
        "Early grade must NOT add Lifespan to real bolt"
    );
    assert!(
        app.world().get::<LifetimeEndBehavior>(real_bolt).is_none(),
        "Early grade must NOT add LifetimeEndBehavior to real bolt"
    );
    assert!(
        app.world().get::<PhantomFlicker>(real_bolt).is_none(),
        "Early grade must NOT add PhantomFlicker to real bolt"
    );
    assert_eq!(
        phantom_bolt_count(&mut app),
        0,
        "Early grade must NOT create any phantom-bolt entities"
    );
}

// ── T19 (edge case) — Late grade also does NOT mutate the real bolt ───────────

#[test]
fn late_grade_does_not_mutate_real_bolt() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    write_bump_performed(&mut app, phantom_breaker, Some(real_bolt), BumpGrade::Late);
    tick(&mut app);

    assert!(
        app.world().get::<PhantomBolt>(real_bolt).is_none(),
        "Late grade must NOT add PhantomBolt to real bolt"
    );
    assert!(
        app.world().get::<PhantomDedupKey>(real_bolt).is_none(),
        "Late grade must NOT add PhantomDedupKey to real bolt"
    );
    assert!(
        app.world().get::<PhantomDamagedCells>(real_bolt).is_none(),
        "Late grade must NOT add PhantomDamagedCells to real bolt"
    );
    assert!(
        app.world().get::<Lifespan>(real_bolt).is_none(),
        "Late grade must NOT add Lifespan to real bolt"
    );
    assert!(
        app.world().get::<LifetimeEndBehavior>(real_bolt).is_none(),
        "Late grade must NOT add LifetimeEndBehavior to real bolt"
    );
    assert!(
        app.world().get::<PhantomFlicker>(real_bolt).is_none(),
        "Late grade must NOT add PhantomFlicker to real bolt"
    );
    assert_eq!(
        phantom_bolt_count(&mut app),
        0,
        "Late grade must NOT create any phantom-bolt entities"
    );
}

// ── F4 — breaker is NOT a PhantomBreaker → real bolt unchanged ───────────────

#[test]
fn non_phantom_breaker_does_not_mutate_real_bolt() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (_phantom_breaker, real_bolt) = canonical_setup(&mut app);

    let real_breaker = app.world_mut().spawn(Breaker).id();
    write_bump_performed(&mut app, real_breaker, Some(real_bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        app.world().get::<PhantomBolt>(real_bolt).is_none(),
        "Perfect bump from a non-PhantomBreaker must NOT add PhantomBolt to real bolt"
    );
    assert_eq!(
        phantom_bolt_count(&mut app),
        0,
        "Perfect bump from a non-PhantomBreaker must NOT create phantom-bolt entities"
    );
}

// ── F4 (edge case) — placeholder breaker → no mutation, no panic ─────────────

#[test]
fn placeholder_breaker_does_not_mutate_and_does_not_panic() {
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
        app.world().get::<PhantomBolt>(real_bolt).is_none(),
        "Placeholder breaker must not mutate real bolt"
    );
}

// ── F5 — bolt: None (spectator) is a no-op ───────────────────────────────────

#[test]
fn spectator_bump_with_bolt_none_does_not_mutate() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    write_bump_performed(&mut app, phantom_breaker, None, BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        app.world().get::<PhantomBolt>(real_bolt).is_none(),
        "bolt: None → real bolt must not gain PhantomBolt"
    );
    assert_eq!(
        phantom_bolt_count(&mut app),
        0,
        "bolt: None → zero phantom-bolt entities in world"
    );
}

// ── F6 — despawned bolt is tolerated ─────────────────────────────────────────

#[test]
fn despawned_bolt_is_tolerated() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    app.world_mut().entity_mut(real_bolt).despawn();

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    // No panic. World contains no phantom-bolt entity (real bolt is gone).
    assert_eq!(
        phantom_bolt_count(&mut app),
        0,
        "despawned bolt must not result in a phantom-bolt mutation or spawn"
    );
}
