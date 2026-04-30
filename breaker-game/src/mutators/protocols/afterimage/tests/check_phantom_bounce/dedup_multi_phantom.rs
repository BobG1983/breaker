use bevy::prelude::*;

use super::{
    super::{
        super::system::{PhantomBreaker, PhantomBreakerLifetime},
        helpers::{
            build_afterimage_app, captured_bump_performed, seed_active_protocols_with_afterimage,
            spawn_breaker_with_bump_state, spawn_phantom_bolt_entity, spawn_real_bolt,
        },
    },
    perfect_bump_state, spawn_canonical_phantom_breaker,
};
use crate::{
    breaker::components::{BaseHeight, BaseWidth},
    prelude::*,
};

// ── CB7 — Without<PhantomBolt> filter prevents cascade ─────────────────────

#[test]
fn phantom_bolt_filter_prevents_cascade_re_bounce() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom_breaker = spawn_canonical_phantom_breaker(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    // Spawn a PHANTOM bolt OVERLAPPING the phantom breaker AABB (center
    // inside the top-face) moving DOWN — identical geometry to CB1 so the
    // only thing preventing reflection is the `Without<PhantomBolt>`
    // filter.
    let dummy = app.world_mut().spawn_empty().id();
    let phantom_bolt = spawn_phantom_bolt_entity(
        &mut app,
        dummy,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        3.0,
    );

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(phantom_bolt).unwrap();
    assert!(
        (velocity.0.y - (-400.0)).abs() < 1.0,
        "phantom bolt must NOT be reflected (Without<PhantomBolt> filter), got y={}",
        velocity.0.y
    );
    assert!(
        captured_bump_performed(&app).is_empty(),
        "cascade prevention: phantom bolt overlapping phantom breaker must emit zero BumpPerformed"
    );
}

// ── CB7 (edge case) — stale PhantomOwner does not panic ───────────────────

#[test]
fn phantom_bolt_with_stale_owner_does_not_panic() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom_breaker = spawn_canonical_phantom_breaker(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    // Spawn a dummy, then IMMEDIATELY despawn it — the PhantomOwner points
    // at a dead entity.
    let dummy = app.world_mut().spawn_empty().id();
    let phantom_bolt = spawn_phantom_bolt_entity(
        &mut app,
        dummy,
        Vec2::new(0.0, -16.0),
        Vec2::new(0.0, 400.0),
        3.0,
    );
    app.world_mut().entity_mut(dummy).despawn();

    tick(&mut app);

    // No panic. Phantom still alive (hasn't expired).
    assert!(
        app.world().get_entity(phantom_bolt).is_ok(),
        "phantom bolt must survive even with stale PhantomOwner"
    );
    assert!(
        captured_bump_performed(&app).is_empty(),
        "cascade prevention holds even with stale PhantomOwner"
    );
}

// ── CB8 — bounced_this_frame dedup: bolt overlapping two phantoms ─────────

#[test]
fn bounced_this_frame_dedup_emits_exactly_one_bump_even_when_two_phantoms_overlap() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);

    // Two PhantomBreakers — their AABBs overlap each other, and both
    // contain the origin.
    let phantom_a = app
        .world_mut()
        .spawn((
            PhantomBreaker,
            PhantomBreakerLifetime(1.5),
            Position2D(Vec2::new(-10.0, 0.0)),
            BaseWidth(100.0),
            BaseHeight(20.0),
        ))
        .id();
    let phantom_b = app
        .world_mut()
        .spawn((
            PhantomBreaker,
            PhantomBreakerLifetime(1.5),
            Position2D(Vec2::new(10.0, 0.0)),
            BaseWidth(100.0),
            BaseHeight(20.0),
        ))
        .id();

    // Entry-guard requires downward motion: bolt ABOVE the phantom AABB
    // moving DOWN into it.
    let real_bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);

    tick(&mut app);

    let bumps = captured_bump_performed(&app);
    assert_eq!(
        bumps.len(),
        1,
        "EXACTLY one BumpPerformed must be emitted even when the bolt overlaps two \
         PhantomBreakers — dedup must coalesce, got {}",
        bumps.len()
    );
    assert_eq!(bumps[0].bolt, Some(real_bolt));
    assert!(
        bumps[0].breaker == phantom_a || bumps[0].breaker == phantom_b,
        "emitted breaker must be one of the two phantoms"
    );
    let velocity = app.world().get::<Velocity2D>(real_bolt).unwrap();
    assert!(
        velocity.0.y > 0.0,
        "bolt must be reflected ONCE, got velocity.y={}",
        velocity.0.y
    );
}

// ── CB8 (edge case) — three overlapping phantoms still emit exactly one ───

#[test]
fn three_overlapping_phantoms_emit_exactly_one_bump_performed() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);

    for offset in [-10.0, 0.0, 10.0] {
        app.world_mut().spawn((
            PhantomBreaker,
            PhantomBreakerLifetime(1.5),
            Position2D(Vec2::new(offset, 0.0)),
            BaseWidth(100.0),
            BaseHeight(20.0),
        ));
    }

    let _real_bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);

    tick(&mut app);

    assert_eq!(
        captured_bump_performed(&app).len(),
        1,
        "EXACTLY one BumpPerformed even with three overlapping phantoms"
    );
}
