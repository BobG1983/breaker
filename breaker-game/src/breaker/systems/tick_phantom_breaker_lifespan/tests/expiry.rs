//! Tests #10a–#10e: lifespan expiry, `DespawnEntity` emission, and filter correctness.

use super::helpers::{FIXED_DT, lifespan_despawn_app, lifespan_emit_app, spawn_phantom};
use crate::{
    breaker::{components::Breaker, test_utils::spawn_breaker},
    prelude::*,
    shared::Lifespan,
};

// ── Test #10a — Phantom decremented past zero emits exactly one DespawnEntity ──

#[test]
fn phantom_lifespan_past_zero_emits_one_despawn_entity() {
    let mut app = lifespan_emit_app();
    let phantom = spawn_phantom(&mut app, 0.01);

    // Confirm pre-tick state: entity has Lifespan, PhantomBreaker, Breaker.
    {
        let world = app.world();
        let lifespan = world
            .get::<Lifespan>(phantom)
            .expect("Lifespan present before tick");
        assert!(
            (lifespan.remaining - 0.01).abs() < f32::EPSILON,
            "Lifespan.remaining should be 0.01 before tick, got {}",
            lifespan.remaining
        );
        assert!(
            world
                .get::<crate::breaker::components::PhantomBreaker>(phantom)
                .is_some()
        );
        assert!(world.get::<Breaker>(phantom).is_some());
    }

    tick(&mut app);

    // Exactly one DespawnEntity emitted for this phantom.
    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected exactly one DespawnEntity, got {}",
        collector.0.len()
    );
    assert_eq!(
        collector.0[0].entity, phantom,
        "DespawnEntity.entity must equal the phantom's id"
    );

    // Lifespan.remaining should be approximately 0.01 - 0.015625 = -0.005625.
    let lifespan = app
        .world()
        .get::<Lifespan>(phantom)
        .expect("Lifespan still on entity after tick");
    assert!(
        (lifespan.remaining - (-0.005_625_f32)).abs() < 1e-4,
        "Lifespan.remaining should be ≈ -0.005625 after tick, got {}",
        lifespan.remaining
    );
}

// ── Test #10a edge — remaining exactly 0.0 at tick start emits one DespawnEntity ──

#[test]
fn phantom_lifespan_at_zero_boundary_emits_one_despawn_entity() {
    let mut app = lifespan_emit_app();
    let phantom = spawn_phantom(&mut app, 0.0);

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        collector.0.len(),
        1,
        "remaining=0.0 boundary: expected one DespawnEntity"
    );
    assert_eq!(collector.0[0].entity, phantom);

    let lifespan = app
        .world()
        .get::<Lifespan>(phantom)
        .expect("Lifespan still present");
    assert!(
        (lifespan.remaining - (-FIXED_DT)).abs() < 1e-4,
        "remaining should be ≈ -FIXED_DT after tick, got {}",
        lifespan.remaining
    );
}

// ── Test #10b — With RantzDmgPlugin, entity is despawned in same tick ──

#[test]
fn phantom_lifespan_expiry_despawns_entity_via_dmg_plugin() {
    let mut app = lifespan_despawn_app();
    let phantom = spawn_phantom(&mut app, 0.01);

    // Confirm alive before tick.
    assert!(
        app.world().get_entity(phantom).is_ok(),
        "phantom must be alive before tick"
    );
    assert!(app.world().get::<Lifespan>(phantom).is_some());
    assert!(
        app.world()
            .get::<crate::breaker::components::PhantomBreaker>(phantom)
            .is_some()
    );
    assert!(app.world().get::<Breaker>(phantom).is_some());

    tick(&mut app);

    assert!(
        app.world().get_entity(phantom).is_err(),
        "phantom entity must be removed by process_despawn_requests within the same tick"
    );
    assert!(
        app.world()
            .get::<crate::breaker::components::PhantomBreaker>(phantom)
            .is_none(),
        "PhantomBreaker component must be gone once entity is despawned"
    );
}

// ── Test #10b edge — without RantzDmgPlugin, entity is NOT removed (system only emits) ──

#[test]
fn phantom_lifespan_system_only_emits_does_not_despawn_directly() {
    // lifespan_emit_app() does NOT install RantzDmgPlugin.
    let mut app = lifespan_emit_app();
    let phantom = spawn_phantom(&mut app, 0.01);

    tick(&mut app);

    // Message was emitted.
    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        collector.0.len(),
        1,
        "DespawnEntity should still be emitted"
    );

    // But entity is NOT removed — no consumer.
    assert!(
        app.world().get_entity(phantom).is_ok(),
        "without RantzDmgPlugin the entity must NOT be despawned by the new system directly"
    );
}

// ── Test #10c — Phantom with remaining > 0.0 after one tick is NOT despawned ──

#[test]
fn phantom_lifespan_not_expired_emits_no_despawn_entity() {
    let mut app = lifespan_despawn_app();
    let phantom = spawn_phantom(&mut app, 1.0);

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(
        collector.0.is_empty(),
        "no DespawnEntity for phantom with 1.0s lifespan"
    );

    assert!(
        app.world().get_entity(phantom).is_ok(),
        "phantom with lifespan 1.0 must still be alive"
    );

    let lifespan = app
        .world()
        .get::<Lifespan>(phantom)
        .expect("Lifespan present");
    let expected = 1.0_f32 - FIXED_DT;
    assert!(
        (lifespan.remaining - expected).abs() < 1e-4,
        "Lifespan.remaining should be ≈ {} after one tick, got {}",
        expected,
        lifespan.remaining
    );
}

// ── Test #10c edge — two phantoms: only the short-lifespan one emits DespawnEntity ──

#[test]
fn two_phantoms_only_short_lifespan_emits_despawn_entity() {
    let mut app = lifespan_despawn_app();
    let short_phantom = spawn_phantom(&mut app, 0.01);
    let long_phantom = spawn_phantom(&mut app, 1.0);

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        collector.0.len(),
        1,
        "exactly one DespawnEntity for two phantoms with different lifespans"
    );
    assert_eq!(
        collector.0[0].entity, short_phantom,
        "the DespawnEntity must be for the short-lifespan phantom, not the long one"
    );

    assert!(
        app.world().get_entity(long_phantom).is_ok(),
        "long-lifespan phantom must still be alive"
    );
}

// ── Test #10d — Real breaker (no PhantomBreaker) is never touched ──

#[test]
fn real_breaker_with_lifespan_zero_is_not_ticked_or_despawned() {
    let mut app = lifespan_despawn_app();
    let real_breaker = spawn_breaker(&mut app, 0.0, 0.0);

    // Manually insert Lifespan { remaining: 0.0 } to defeat any missing filter.
    app.world_mut()
        .entity_mut(real_breaker)
        .insert(Lifespan { remaining: 0.0 });

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(
        collector.0.is_empty(),
        "no DespawnEntity should be emitted for a real breaker (no PhantomBreaker marker)"
    );

    assert!(
        app.world().get_entity(real_breaker).is_ok(),
        "real breaker must still be alive"
    );

    // The system must NOT have decremented remaining — the WithPhantomBreaker filter excludes it.
    let lifespan = app
        .world()
        .get::<Lifespan>(real_breaker)
        .expect("Lifespan still present on real breaker");
    assert!(
        lifespan.remaining.abs() < f32::EPSILON,
        "Lifespan.remaining on a real breaker must be exactly 0.0 (not ticked), got {}",
        lifespan.remaining
    );
}

// ── Test #10d edge — real breaker and phantom coexist; only phantom is despawned ──

#[test]
fn real_breaker_and_phantom_coexist_only_phantom_is_despawned() {
    let mut app = lifespan_despawn_app();
    let real_breaker = spawn_breaker(&mut app, 0.0, 0.0);
    let phantom = spawn_phantom(&mut app, 0.0);

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        collector.0.len(),
        1,
        "only one DespawnEntity (for the phantom)"
    );
    assert_eq!(
        collector.0[0].entity, phantom,
        "DespawnEntity must be for the phantom"
    );

    assert!(
        app.world().get_entity(real_breaker).is_ok(),
        "real breaker must be untouched"
    );
}

// ── Test #10e — System runs in FixedUpdate, not Update ──

#[test]
fn lifespan_not_ticked_without_fixed_overstep() {
    // No tick() — just a bare app.update() with ManualDuration(ZERO) pinned.
    let mut app = lifespan_despawn_app();
    let phantom = spawn_phantom(&mut app, 0.01);

    // Bare update: ManualDuration(ZERO) means no fixed overstep accumulates.
    app.update();

    let lifespan = app
        .world()
        .get::<Lifespan>(phantom)
        .expect("Lifespan present");
    assert!(
        (lifespan.remaining - 0.01).abs() < f32::EPSILON,
        "Lifespan.remaining must be unchanged after bare update (no FixedUpdate ran), got {}",
        lifespan.remaining
    );

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(
        collector.0.is_empty(),
        "no DespawnEntity without FixedUpdate execution"
    );

    assert!(
        app.world().get_entity(phantom).is_ok(),
        "phantom still alive"
    );
}

// ── Test #10e edge — after the no-op update, tick() DOES fire the system ──

#[test]
fn lifespan_ticked_after_fixed_overstep_following_no_op_update() {
    let mut app = lifespan_despawn_app();
    let phantom = spawn_phantom(&mut app, 0.01);

    // No-op update (proves system didn't run).
    app.update();

    // Now accumulate and tick.
    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        collector.0.len(),
        1,
        "after tick() the system must fire and emit DespawnEntity"
    );
    assert_eq!(collector.0[0].entity, phantom);

    assert!(
        app.world().get_entity(phantom).is_err(),
        "phantom must be despawned after the tick"
    );
}
