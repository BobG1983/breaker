//! T20 and T21 — uniqueness / no-op on re-bump, and lifespan expiry revert.
//!
//! F7 → T20 (already-phantom no-op, duration NOT reset).
//! F8 DELETED — under mutate-real-bolt there is no separate phantom entity that
//!   outlives the real bolt; if the real bolt despawns, all its components go too.
//! F9 (no config) and F10 (no gate) rewritten to assert "real bolt does NOT gain
//!   `PhantomBolt`" instead of "no new phantom entity spawned."
//! T21 (lifespan expiry revert) added here.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::{
        components::{
            BoltBaseDamage, LifetimeEndBehavior, PhantomBolt, PhantomDamagedCells, PhantomDedupKey,
        },
        systems::tick_bolt_lifespan,
    },
    breaker::messages::BumpGrade,
    prelude::*,
    shared::{Lifespan, PhantomFlicker},
};

// ── Helper — build app with bolt-lifespan tick wired ─────────────────────────

fn build_afterimage_app_with_bolt_lifespan() -> App {
    let mut app = build_afterimage_app();
    app.add_message::<DespawnEntity>();
    attach_message_capture::<DespawnEntity>(&mut app);
    app.add_systems(FixedUpdate, tick_bolt_lifespan);
    // Flush deferred commands so any components inserted before the first tick
    // are materialized and visible to tick_bolt_lifespan on tick 1.
    app.world_mut().flush();
    app
}

// ── T20 — second Perfect bump when real bolt already has PhantomBolt is a no-op

#[test]
fn second_perfect_bump_does_not_reset_lifespan() {
    let mut app = build_afterimage_app_with_bolt_lifespan();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    // Pre-seed the real bolt as already-phantom with Lifespan 1.0 (well below 3.0).
    app.world_mut().entity_mut(real_bolt).insert((
        PhantomBolt,
        PhantomDedupKey::Bolt(real_bolt),
        PhantomDamagedCells::default(),
        Lifespan { remaining: 1.0 },
        LifetimeEndBehavior::RevertToNormalBolt,
        PhantomFlicker::default(),
    ));

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    // PhantomBolt still present — bolt is still phantom.
    assert!(
        app.world().get::<PhantomBolt>(real_bolt).is_some(),
        "real bolt must remain a phantom after second Perfect bump"
    );

    // Lifespan decremented by one tick, NOT reset to 3.0.
    let dt = 1.0_f32 / 64.0;
    let lifespan = app
        .world()
        .get::<Lifespan>(real_bolt)
        .expect("Lifespan must still be present");
    let expected = 1.0 - dt;
    assert!(
        (lifespan.remaining - expected).abs() < 1e-4,
        "Lifespan must be ~{expected} (decremented 1/64), NOT reset to 3.0; got {}",
        lifespan.remaining
    );

    // LifetimeEndBehavior unchanged.
    assert_eq!(
        *app.world()
            .get::<LifetimeEndBehavior>(real_bolt)
            .expect("LifetimeEndBehavior must still be present"),
        LifetimeEndBehavior::RevertToNormalBolt,
        "LifetimeEndBehavior must not change on second bump"
    );

    // PhantomFlicker still present.
    assert!(
        app.world().get::<PhantomFlicker>(real_bolt).is_some(),
        "PhantomFlicker must still be present after second bump"
    );

    // Exactly one phantom-bolt entity — the real bolt itself.
    assert_eq!(
        phantom_bolt_count(&mut app),
        1,
        "still exactly one phantom bolt (the real bolt)"
    );
}

// ── T20 (edge case) — existing Lifespan 2.99 decrements, NOT reset to 3.0 ─────

#[test]
fn second_perfect_bump_with_2_99_lifespan_does_not_reset() {
    let mut app = build_afterimage_app_with_bolt_lifespan();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    app.world_mut().entity_mut(real_bolt).insert((
        PhantomBolt,
        PhantomDedupKey::Bolt(real_bolt),
        PhantomDamagedCells::default(),
        Lifespan { remaining: 2.99 },
        LifetimeEndBehavior::RevertToNormalBolt,
        PhantomFlicker::default(),
    ));

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    let dt = 1.0_f32 / 64.0;
    let lifespan = app
        .world()
        .get::<Lifespan>(real_bolt)
        .expect("Lifespan must still be present");
    let expected = 2.99 - dt;
    assert!(
        (lifespan.remaining - expected).abs() < 1e-4,
        "Lifespan 2.99 must decrement by 1/64, NOT reset to 3.0; expected ~{expected}, got {}",
        lifespan.remaining
    );
}

// ── F9 — AfterimageConfig absent: reader drained, real bolt unchanged ─────────

#[test]
fn absent_afterimage_config_does_not_mutate_real_bolt() {
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
        app.world().get::<PhantomBolt>(real_bolt).is_none(),
        "tick 1 with absent AfterimageConfig must NOT add PhantomBolt to real bolt"
    );

    // Insert config; tick 2 — message was already drained.
    app.world_mut()
        .insert_resource(super::super::super::system::AfterimageConfig {
            phantom_duration:      2.0,
            phantom_bolt_duration: 3.0,
        });
    tick(&mut app);

    assert!(
        app.world().get::<PhantomBolt>(real_bolt).is_none(),
        "tick 2 must NOT retro-apply the drained BumpPerformed — real bolt must stay normal"
    );
}

// ── F10 — ActiveProtocols missing Afterimage → real bolt unchanged ────────────

#[test]
fn active_protocols_missing_afterimage_does_not_mutate_real_bolt() {
    let mut app = build_afterimage_app();
    // Do NOT call seed_active_protocols_with_afterimage.
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
        app.world().get::<PhantomBolt>(real_bolt).is_none(),
        "run_if gate: no Afterimage in ActiveProtocols → real bolt must not gain PhantomBolt"
    );
}

// ── T21 — lifespan expiry reverts the real bolt to a normal bolt ──────────────

#[test]
fn phantom_bolt_reverts_to_normal_on_lifespan_expiry() {
    let mut app = build_afterimage_app_with_bolt_lifespan();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    // Install-tick: write the bump and let the system fire once.
    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    // Verify pre-condition: real bolt is now phantom.
    assert!(
        app.world().get::<PhantomBolt>(real_bolt).is_some(),
        "pre-condition: real bolt must be phantom after install-tick"
    );

    // Advance 191 more ticks to drive Lifespan.remaining to ≤ 0.
    // The installer pre-subtracts one `delta_secs()` to compensate for the
    // same-FixedUpdate deferred-flush gap, so post-flush `remaining` = 3.0 − 1/64
    // = 2.984375. 191 × 1/64 = 2.984375; the final tick pushes remaining ≤ 0.0
    // and fires expiry.
    tick_n(&mut app, 191);

    // PhantomBolt stripped.
    assert!(
        app.world().get::<PhantomBolt>(real_bolt).is_none(),
        "PhantomBolt must be removed on lifespan expiry"
    );
    assert!(
        app.world().get::<PhantomDedupKey>(real_bolt).is_none(),
        "PhantomDedupKey must be removed on lifespan expiry"
    );
    assert!(
        app.world().get::<PhantomDamagedCells>(real_bolt).is_none(),
        "PhantomDamagedCells must be removed on lifespan expiry"
    );
    assert!(
        app.world().get::<PhantomFlicker>(real_bolt).is_none(),
        "PhantomFlicker must be removed on lifespan expiry"
    );
    assert!(
        app.world().get::<Lifespan>(real_bolt).is_none(),
        "Lifespan must be removed on lifespan expiry"
    );
    assert!(
        app.world().get::<LifetimeEndBehavior>(real_bolt).is_none(),
        "LifetimeEndBehavior must be removed on lifespan expiry"
    );

    // Real bolt still alive — NOT despawned.
    assert!(
        app.world().get_entity(real_bolt).is_ok(),
        "real bolt must still exist after RevertToNormalBolt expiry (not despawned)"
    );
    assert!(
        app.world().get::<Bolt>(real_bolt).is_some(),
        "Bolt marker must still be present after revert"
    );

    // Gameplay state preserved.
    let vel = app.world().get::<Velocity2D>(real_bolt).unwrap().0;
    assert!(
        (vel - Vec2::new(0.0, 400.0)).length() < 1e-3,
        "velocity must be broadly unchanged from canonical spawn after revert; got {vel:?}"
    );
    assert!(
        (app.world().get::<BoltBaseDamage>(real_bolt).unwrap().0 - 10.0).abs() < 1e-6,
        "BoltBaseDamage must be 10.0 after revert"
    );
}

// ── T21 (edge case) — LifetimeEndBehavior::Despawn emits DespawnEntity ────────

#[test]
fn phantom_bolt_with_despawn_behavior_emits_despawn_entity_on_expiry() {
    let mut app = build_afterimage_app_with_bolt_lifespan();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    // Install-tick: write the bump, system fires and installs RevertToNormalBolt.
    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    // Override to Despawn AFTER the install-tick (afterimage sets RevertToNormalBolt).
    app.world_mut()
        .entity_mut(real_bolt)
        .insert(LifetimeEndBehavior::Despawn);

    // Advance to expiry.
    tick_n(&mut app, 191);

    // DespawnEntity must have been emitted for the real bolt.
    assert!(
        app.world()
            .resource::<MessageCollector<DespawnEntity>>()
            .0
            .iter()
            .any(|m| m.entity == real_bolt),
        "expiry with LifetimeEndBehavior::Despawn must emit DespawnEntity for the real bolt"
    );
}
