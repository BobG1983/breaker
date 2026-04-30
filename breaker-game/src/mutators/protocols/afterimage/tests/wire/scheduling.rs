use bevy::prelude::*;

use super::{
    super::{
        super::system::{AfterimageConfig, PhantomBreaker},
        helpers::{
            build_afterimage_app, build_afterimage_app_no_config, captured_bump_performed,
            phantom_bolt_count, phantom_bolts_owned_by, phantom_breaker_count,
            seed_active_protocols_with_afterimage, spawn_breaker_with_bump_state,
            spawn_breaker_with_dash, spawn_phantom_breaker_at, spawn_real_bolt, tick_n,
            write_bump_performed,
        },
    },
    helpers::perfect_bump_state,
};
use crate::{
    breaker::{components::DashState, messages::BumpGrade},
    prelude::*,
};

// ── I1 — wire wires spawn_phantom_breaker in FixedUpdate ──────────────

#[test]
fn register_wires_spawn_phantom_breaker_in_fixed_update() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let breaker = spawn_breaker_with_dash(&mut app, DashState::Idle, Vec2::ZERO);

    app.world_mut()
        .entity_mut(breaker)
        .insert(DashState::Dashing);
    tick(&mut app);

    assert_eq!(
        phantom_breaker_count(&mut app),
        1,
        "afterimage_spawn_phantom_breaker must run via wire"
    );
}

// ── I4 — tick_phantom_breaker wired + gated ───────────────────────────────

#[test]
fn tick_phantom_breaker_wired_and_gated_on_active_and_playing() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 0.01);

    tick_n(&mut app, 2);

    assert!(
        app.world().get::<PhantomBreaker>(phantom).is_none(),
        "tick_phantom_breaker must despawn the short-lived phantom"
    );
}

// ── I5 — check_phantom_bounce wired + gated on active + Playing ───────────

#[test]
fn check_phantom_bounce_wired_and_gated_on_active_and_playing() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0), // overlapping top face
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        velocity.0.y > 0.0,
        "check_phantom_bounce must reflect the bolt via wire, got y={}",
        velocity.0.y
    );
    let bumps = captured_bump_performed(&app);
    assert_eq!(bumps.len(), 1);
    assert_eq!(bumps[0].breaker, phantom);
}

// ── I6 — spawn_phantom_bolt wired + gated on active + Playing ─────────────

#[test]
fn spawn_phantom_bolt_wired_and_gated_on_active_and_playing() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 10.0, 6.0);

    write_bump_performed(&mut app, phantom, Some(real_bolt), BumpGrade::Perfect);
    tick(&mut app);

    let owned = phantom_bolts_owned_by(&mut app, real_bolt);
    assert_eq!(owned.len(), 1);
}

// ── I9 — tick_phantom_lifetime is live + despawns afterimage phantoms ─────

#[test]
fn tick_phantom_lifetime_is_live_and_despawns_afterimage_phantoms() {
    use crate::effect_v3::effects::phantom_bolt::components::PhantomLifetime;

    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 10.0, 6.0);

    write_bump_performed(&mut app, phantom, Some(real_bolt), BumpGrade::Perfect);
    tick(&mut app);
    let owned = phantom_bolts_owned_by(&mut app, real_bolt);
    assert_eq!(owned.len(), 1, "phantom bolt spawned");

    // Despawn by manipulating the PhantomLifetime so tick_phantom_lifetime
    // terminates the entity quickly — instead of waiting 3.0 seconds.
    let phantom_entity = owned[0];
    app.world_mut()
        .entity_mut(phantom_entity)
        .insert(PhantomLifetime(0.01));

    tick_n(&mut app, 2);

    assert!(
        app.world().get_entity(phantom_entity).is_err(),
        "tick_phantom_lifetime must despawn the afterimage-spawned phantom"
    );
}

// ── I11 — quiet tick safety under the full schedule ───────────────────────

#[test]
fn quiet_tick_safety_under_full_schedule() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);

    tick_n(&mut app, 3);

    assert_eq!(
        *app.world().resource::<AfterimageConfig>(),
        AfterimageConfig {
            phantom_duration:      2.0,
            phantom_bolt_duration: 3.0,
        },
        "AfterimageConfig must remain canonical across quiet ticks"
    );
    assert_eq!(phantom_breaker_count(&mut app), 0);
    assert_eq!(phantom_bolt_count(&mut app), 0);
}

// ── I12 — wire does not panic when AfterimageConfig absent ────────────

#[test]
fn register_does_not_panic_when_afterimage_config_absent() {
    let mut app = build_afterimage_app_no_config();
    seed_active_protocols_with_afterimage(&mut app);

    tick_n(&mut app, 3);

    assert!(
        app.world().get_resource::<AfterimageConfig>().is_none(),
        "wire must not side-effect-insert AfterimageConfig"
    );
}
