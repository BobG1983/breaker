//! Group G — `burnout_cleanup_node` on `OnExit(NodeState::Playing)`
//! (Behaviors G1–G5).
//!
//! Pins that cleanup:
//! - Removes `BurnoutHeat` from every breaker that carries it.
//! - Removes `BurnoutSpeedBoost` from every breaker that carries it.
//! - Removes `BurnoutDamageBoost` from every bolt that carries it.
//! - Runs unconditionally — no `run_if(protocol_active)` gate.
//! - Tolerates absent `BurnoutConfig`.
//! - Does not resurrect stale state on re-entry into `Playing`.
//!
//! State transitions use `app.update()` — state transitions fire on
//! schedule edges, mirroring the Reckless Dash cleanup convention.

use bevy::prelude::*;

use super::{
    super::system::{BurnoutDamageBoost, BurnoutHeat, BurnoutSpeedBoost},
    helpers::{
        build_burnout_app, build_burnout_app_no_config, install_burnout_damage_boost,
        install_burnout_speed_boost, seed_active_protocols_with_burnout, set_heat_state,
        spawn_bolt_with_base_damage, spawn_breaker_stationary,
    },
};
use crate::prelude::*;

fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_burnout(app, 4.0, 2.0, 1.5, 4.0, 2.0);
}

// ── G1 — Cleanup removes BurnoutHeat from all breakers ─────────────────────-

#[test]
fn exit_playing_removes_burnout_heat_from_all_breakers() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker_a = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker_a, 0.7, 0.3, true);
    let breaker_b = spawn_breaker_stationary(&mut app, Vec2::new(100.0, -400.0));
    set_heat_state(&mut app, breaker_b, 0.7, 0.3, true);

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    assert!(
        app.world().get::<BurnoutHeat>(breaker_a).is_none(),
        "breaker_a must not carry BurnoutHeat after OnExit(Playing)"
    );
    assert!(
        app.world().get::<BurnoutHeat>(breaker_b).is_none(),
        "breaker_b must not carry BurnoutHeat after OnExit(Playing)"
    );
}

// ── G2 — Cleanup removes BurnoutSpeedBoost from all breakers ───────────────-

#[test]
fn exit_playing_removes_burnout_speed_boost_from_all_breakers() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker_a = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    install_burnout_speed_boost(&mut app, breaker_a, 1.5);
    let breaker_b = spawn_breaker_stationary(&mut app, Vec2::new(100.0, -400.0));
    install_burnout_speed_boost(&mut app, breaker_b, 1.5);

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    assert!(
        app.world().get::<BurnoutSpeedBoost>(breaker_a).is_none(),
        "breaker_a must not carry BurnoutSpeedBoost after OnExit(Playing)"
    );
    assert!(
        app.world().get::<BurnoutSpeedBoost>(breaker_b).is_none(),
        "breaker_b must not carry BurnoutSpeedBoost after OnExit(Playing)"
    );
}

// ── G3 — Cleanup removes BurnoutDamageBoost from all bolts ─────────────────-

#[test]
fn exit_playing_removes_burnout_damage_boost_from_all_bolts() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let bolt_a = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt_a, 4.0);
    let bolt_b = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt_b, 4.0);

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    assert!(
        app.world().get::<BurnoutDamageBoost>(bolt_a).is_none(),
        "bolt_a must not carry BurnoutDamageBoost after OnExit(Playing)"
    );
    assert!(
        app.world().get::<BurnoutDamageBoost>(bolt_b).is_none(),
        "bolt_b must not carry BurnoutDamageBoost after OnExit(Playing)"
    );
}

// ── G4 — Cleanup runs unconditionally — even when Burnout NOT active ───────-

#[test]
fn cleanup_runs_even_when_burnout_not_active() {
    let mut app = build_burnout_app();
    // Do NOT seed ActiveProtocols.
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 0.7, 0.3, true);
    install_burnout_speed_boost(&mut app, breaker, 1.5);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt, 4.0);

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    assert!(
        app.world().get::<BurnoutHeat>(breaker).is_none(),
        "cleanup must remove BurnoutHeat even when Burnout NOT active"
    );
    assert!(
        app.world().get::<BurnoutSpeedBoost>(breaker).is_none(),
        "cleanup must remove BurnoutSpeedBoost even when Burnout NOT active"
    );
    assert!(
        app.world().get::<BurnoutDamageBoost>(bolt).is_none(),
        "cleanup must remove BurnoutDamageBoost even when Burnout NOT active"
    );
}

// ── G5a — Re-entry into Playing does NOT resurrect stale state ─────────────-

#[test]
fn re_entry_does_not_resurrect_stale_burnout_components() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 0.7, 0.3, true);
    install_burnout_speed_boost(&mut app, breaker, 1.5);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt, 4.0);

    // Exit Playing → cleanup fires.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    // Re-enter Playing.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();

    // Two further ticks.
    tick(&mut app);
    tick(&mut app);

    assert!(
        app.world().get::<BurnoutSpeedBoost>(breaker).is_none(),
        "BurnoutSpeedBoost must remain absent after re-entry"
    );
    assert!(
        app.world().get::<BurnoutDamageBoost>(bolt).is_none(),
        "BurnoutDamageBoost must remain absent after re-entry"
    );
}

// ── G5b — Cleanup tolerates absent BurnoutConfig ───────────────────────────-

#[test]
fn cleanup_tolerates_absent_burnout_config() {
    let mut app = build_burnout_app_no_config();
    // Do NOT seed ActiveProtocols.
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 0.7, 0.3, true);
    install_burnout_speed_boost(&mut app, breaker, 1.5);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt, 4.0);

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update(); // must not panic

    assert!(app.world().get::<BurnoutHeat>(breaker).is_none());
    assert!(app.world().get::<BurnoutSpeedBoost>(breaker).is_none());
    assert!(app.world().get::<BurnoutDamageBoost>(bolt).is_none());
}
