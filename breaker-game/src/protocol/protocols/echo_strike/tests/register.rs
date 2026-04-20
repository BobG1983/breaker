//! Group G — `register` wiring + run-condition gates (Behaviors 46–60).
//!
//! Pins that `register`-wired systems run under the correct schedules, gated
//! by `protocol_active(EchoStrike)` + `in_state(NodeState::Playing)` on the
//! three `FixedUpdate` reader systems, that cleanup runs on
//! `OnExit(NodeState::Playing)` unconditionally, that same-tick ordering
//! anchors work, and that the schedule is harness-safe under missing
//! resources + quiet ticks.

use bevy::prelude::*;

use super::{
    super::system::{ECHO_STRIKE_SENTINEL, EchoNetwork, EchoPrimed, EchoStrikeConfig},
    helpers::{
        build_echo_strike_app, build_echo_strike_app_in_chip_selecting,
        build_echo_strike_app_no_config, collected_echo_strike_damage, read_echo_network,
        seed_active_protocols_with_echo_strike, spawn_bolt_primed_with_network,
        spawn_bolt_with_base_damage, spawn_bolt_with_echo_network, spawn_cell_empty,
        write_bolt_impact_cell, write_bump_performed, write_destroyed_cell,
    },
};
use crate::{breaker::messages::BumpGrade, prelude::*};

fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_echo_strike(app, 3, 0.5, 0.25, 0.1);
}

// ── Behavior 46 — on_bump wired + gated on active + Playing ────────────────-

#[test]
fn register_wires_on_bump_gated_on_active_and_playing() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        app.world().get::<EchoPrimed>(bolt).is_some(),
        "on_bump must run via register"
    );
}

// ── Behavior 47 — on_bump gated off when inactive ──────────────────────────-

#[test]
fn on_bump_gated_off_when_echo_strike_not_active() {
    let mut app = build_echo_strike_app();
    // Do NOT seed ActiveProtocols.
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "on_bump must not run when Echo Strike inactive"
    );
}

// ── Behavior 48 — on_bump gated off when NodeState != Playing ──────────────-

#[test]
fn on_bump_gated_off_when_node_state_not_playing() {
    let mut app = build_echo_strike_app_in_chip_selecting();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "on_bump must not run in ChipSelecting"
    );
}

// ── Behavior 49 — on_impact wired + gated on active + Playing ──────────────-

#[test]
fn register_wires_on_impact_gated_on_active_and_playing() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 10.0, vec![a]);
    let b = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, b, bolt);
    tick(&mut app);

    let msgs = collected_echo_strike_damage(&app);
    assert_eq!(msgs.len(), 1, "one echo-damage message expected");
    assert_eq!(msgs[0].source_chip.as_deref(), Some(ECHO_STRIKE_SENTINEL));
    assert!(
        (msgs[0].amount - 5.0).abs() < 1e-4,
        "amount = 10.0 * 0.5 = 5.0, got {}",
        msgs[0].amount
    );
}

// ── Behavior 50 — on_impact gated off when inactive ────────────────────────-

#[test]
fn on_impact_gated_off_when_echo_strike_not_active() {
    let mut app = build_echo_strike_app();
    // Do NOT seed ActiveProtocols.
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 10.0, vec![a]);
    let b = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, b, bolt);
    tick(&mut app);

    assert!(
        collected_echo_strike_damage(&app).is_empty(),
        "no echo damage when inactive"
    );
    assert!(
        app.world().get::<EchoPrimed>(bolt).is_some(),
        "EchoPrimed must remain — system never ran"
    );
}

// ── Behavior 51 — on_impact gated off when NodeState != Playing ────────────-

#[test]
fn on_impact_gated_off_when_node_state_not_playing() {
    let mut app = build_echo_strike_app_in_chip_selecting();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 10.0, vec![a]);
    let b = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, b, bolt);
    tick(&mut app);

    assert!(
        collected_echo_strike_damage(&app).is_empty(),
        "no echo damage in ChipSelecting"
    );
    assert!(
        app.world().get::<EchoPrimed>(bolt).is_some(),
        "EchoPrimed must remain"
    );
}

// ── Behavior 52 — cleanup_destroyed wired + gated ──────────────────────────-

#[test]
fn register_wires_cleanup_destroyed_gated_on_active_and_playing() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_with_echo_network(&mut app, 10.0, vec![a]);

    write_destroyed_cell(&mut app, a);
    tick(&mut app);

    assert!(
        read_echo_network(&app, bolt).is_empty(),
        "cleanup_destroyed must run via register"
    );
}

// ── Behavior 53 — cleanup_destroyed gated off when inactive ────────────────-

#[test]
fn cleanup_destroyed_gated_off_when_echo_strike_not_active() {
    let mut app = build_echo_strike_app();
    // Do NOT seed ActiveProtocols.
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_with_echo_network(&mut app, 10.0, vec![a]);

    write_destroyed_cell(&mut app, a);
    tick(&mut app);

    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a],
        "network unchanged when inactive"
    );
}

// ── Behavior 54 — cleanup_destroyed gated off when NodeState != Playing ────-

#[test]
fn cleanup_destroyed_gated_off_when_node_state_not_playing() {
    let mut app = build_echo_strike_app_in_chip_selecting();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_with_echo_network(&mut app, 10.0, vec![a]);

    write_destroyed_cell(&mut app, a);
    tick(&mut app);

    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a],
        "network unchanged in ChipSelecting"
    );
}

// ── Behavior 55 — cleanup_node wired unconditionally on OnExit(Playing) ────-

#[test]
fn register_wires_cleanup_node_on_exit_unconditionally() {
    let mut app = build_echo_strike_app();
    // Do NOT seed ActiveProtocols — cleanup must still fire.
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_with_echo_network(&mut app, 10.0, vec![a]);
    app.world_mut().entity_mut(bolt).insert(EchoPrimed);

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    assert!(
        app.world().get::<EchoNetwork>(bolt).is_none(),
        "cleanup must run on OnExit(Playing) even when protocol not active"
    );
    assert!(app.world().get::<EchoPrimed>(bolt).is_none());
}

// ── Behavior 56 — on_bump ordering (after BreakerSystems::GradeBump) ───────-

#[test]
fn register_wires_on_bump_to_consume_bump_performed_same_tick() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        app.world().get::<EchoPrimed>(bolt).is_some(),
        "on_bump must consume same-tick BumpPerformed"
    );
}

// ── Behavior 57 — on_impact ordering (after BoltSystems::CellCollision) ────-

#[test]
fn register_wires_on_impact_to_consume_bolt_impact_cell_same_tick() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 10.0, vec![a]);
    let b = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, b, bolt);
    tick(&mut app);

    let msgs = collected_echo_strike_damage(&app);
    assert_eq!(msgs.len(), 1, "one echo-damage message same-tick");
}

// ── Behavior 58 — cleanup_destroyed ordering (after HandleKill) ────────────-

#[test]
fn register_wires_cleanup_destroyed_to_consume_destroyed_same_tick() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_with_echo_network(&mut app, 10.0, vec![a]);

    write_destroyed_cell(&mut app, a);
    tick(&mut app);

    assert!(
        read_echo_network(&app, bolt).is_empty(),
        "cleanup_destroyed must consume same-tick Destroyed<Cell>"
    );
}

// ── Behavior 59 — quiet-tick safety ─────────────────────────────────────────

#[test]
fn register_schedule_ticks_cleanly_with_no_bolts_and_no_messages() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);

    for _ in 0..3 {
        tick(&mut app);
    }

    assert!(
        collected_echo_strike_damage(&app).is_empty(),
        "quiet schedule must not produce echo-damage messages"
    );

    // EchoStrikeConfig still present and unchanged.
    let cfg = app
        .world()
        .get_resource::<EchoStrikeConfig>()
        .expect("EchoStrikeConfig should still be present");
    assert_eq!(cfg.max_echoes, 3);
    assert!((cfg.newest_fraction - 0.5).abs() < f32::EPSILON);
    assert!((cfg.middle_fraction - 0.25).abs() < f32::EPSILON);
    assert!((cfg.oldest_fraction - 0.1).abs() < f32::EPSILON);
}

// ── Behavior 60 — register does not panic when EchoStrikeConfig absent ─────-

#[test]
fn register_does_not_panic_when_config_absent() {
    let mut app = build_echo_strike_app_no_config();
    seed_canonical(&mut app);

    for _ in 0..3 {
        tick(&mut app);
    }

    assert!(
        app.world().get_resource::<EchoStrikeConfig>().is_none(),
        "register must not side-effect-insert EchoStrikeConfig"
    );
    assert!(
        collected_echo_strike_damage(&app).is_empty(),
        "no echo damage captured"
    );
}
