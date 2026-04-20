//! Group E — `echo_strike_cleanup_destroyed_echoes` (Behaviors 34–41).
//!
//! Pins the `Destroyed<Cell>` consumer:
//! - Removes matching victim from all bolts' `EchoNetwork` deques.
//! - Handles multiple victims per tick.
//! - Non-echo victims are a no-op.
//! - Gating: off when Echo Strike inactive, off when
//!   `NodeState != Playing`.
//! - Empty networks are a safe no-op.
//! - Harness-safe: `reader.clear()` + early-return when `EchoStrikeConfig`
//!   absent.

use bevy::prelude::*;

use super::helpers::{
    build_echo_strike_app, build_echo_strike_app_in_chip_selecting,
    build_echo_strike_app_no_config, read_echo_network, seed_active_protocols_with_echo_strike,
    spawn_bolt_with_echo_network, spawn_cell_empty, write_destroyed_cell,
};
use crate::prelude::*;

fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_echo_strike(app, 3, 0.5, 0.25, 0.1);
}

// ── Behavior 34 — Destroyed<Cell> removes victim from network ──────────────-

#[test]
fn destroyed_cell_removes_victim_from_echo_network() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let b = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_with_echo_network(&mut app, 10.0, vec![a, b]);

    write_destroyed_cell(&mut app, a);
    tick(&mut app);

    assert_eq!(
        read_echo_network(&app, bolt),
        vec![b],
        "A should be evicted from the network"
    );
}

// ── Behavior 35 — multiple Destroyed messages in one tick ──────────────────-

#[test]
fn multiple_destroyed_messages_remove_each_matching_entry() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let b = spawn_cell_empty(&mut app);
    let c = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_with_echo_network(&mut app, 10.0, vec![a, b, c]);

    write_destroyed_cell(&mut app, a);
    write_destroyed_cell(&mut app, c);
    tick(&mut app);

    assert_eq!(
        read_echo_network(&app, bolt),
        vec![b],
        "both A and C should be evicted; B remains"
    );
}

// ── Behavior 36 — non-echo victim is a no-op ───────────────────────────────-

#[test]
fn destroyed_cell_non_echo_victim_is_no_op() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let b = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_with_echo_network(&mut app, 10.0, vec![a, b]);
    let z = spawn_cell_empty(&mut app);

    write_destroyed_cell(&mut app, z);
    tick(&mut app); // must not panic

    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a, b],
        "network must be unchanged — Z was never in it"
    );
}

// ── Behavior 37 — destroyed across multiple bolts ──────────────────────────-

#[test]
fn destroyed_cell_removes_victim_from_all_networks() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let cell_a = spawn_cell_empty(&mut app);
    let cell_b = spawn_cell_empty(&mut app);
    let cell_c = spawn_cell_empty(&mut app);
    let bolt_first = spawn_bolt_with_echo_network(&mut app, 10.0, vec![cell_a, cell_b]);
    let bolt_second = spawn_bolt_with_echo_network(&mut app, 10.0, vec![cell_a, cell_c]);

    write_destroyed_cell(&mut app, cell_a);
    tick(&mut app);

    assert_eq!(
        read_echo_network(&app, bolt_first),
        vec![cell_b],
        "bolt_first should have A evicted, B retained"
    );
    assert_eq!(
        read_echo_network(&app, bolt_second),
        vec![cell_c],
        "bolt_second should have A evicted, C retained"
    );
}

// ── Behavior 38 — gated off when Echo Strike inactive ──────────────────────-

#[test]
fn cleanup_destroyed_gated_off_when_echo_strike_inactive() {
    let mut app = build_echo_strike_app();
    // Do NOT seed ActiveProtocols.
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_with_echo_network(&mut app, 10.0, vec![a]);

    write_destroyed_cell(&mut app, a);
    tick(&mut app);

    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a],
        "network must be unchanged when Echo Strike inactive"
    );
}

// ── Behavior 39 — gated off when NodeState != Playing ──────────────────────-

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
        "network must be unchanged in ChipSelecting"
    );
}

// ── Behavior 40 — empty network is safe no-op ──────────────────────────────-

#[test]
fn cleanup_destroyed_with_empty_network_is_no_op() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_echo_network(&mut app, 10.0, vec![]);

    write_destroyed_cell(&mut app, Entity::PLACEHOLDER);
    tick(&mut app); // must not panic

    assert_eq!(
        read_echo_network(&app, bolt),
        Vec::<Entity>::new(),
        "network should remain empty"
    );
}

// ── Behavior 41 — early-returns + clears reader when config absent ─────────-

#[test]
fn cleanup_destroyed_early_returns_and_clears_reader_when_config_absent() {
    let mut app = build_echo_strike_app_no_config();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let b = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_with_echo_network(&mut app, 10.0, vec![a, b]);

    write_destroyed_cell(&mut app, a);
    tick(&mut app); // config absent → early return

    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a, b],
        "network must be unchanged when config absent"
    );

    // Second quiet tick — buffered message must NOT be processed.
    tick(&mut app);
    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a, b],
        "network must remain unchanged after quiet tick 2 (reader was drained)"
    );
}
