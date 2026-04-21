use super::{
    super::{
        super::system::EchoPrimed,
        helpers::{
            build_echo_strike_app, collected_echo_strike_damage, find_amount_for,
            read_echo_network, spawn_bolt_primed_with_network, spawn_cell_empty,
            write_bolt_impact_cell,
        },
    },
    helpers::seed_canonical,
};
use crate::prelude::*;

// ── Behavior 19 — 2-echo: oldest + newest (middle slot SKIPPED) ────────────-

#[test]
fn primed_impact_two_echoes_uses_oldest_and_newest_skipping_middle() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let b = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 20.0, vec![a, b]);
    let c = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, c, bolt);
    tick(&mut app);

    let msgs = collected_echo_strike_damage(&app);
    assert_eq!(
        msgs.len(),
        2,
        "2-echo network should emit exactly 2 echo-damage messages"
    );

    let amt_b = find_amount_for(&msgs, b).expect("echo message for B (newest)");
    assert!(
        (amt_b - 10.0).abs() < 1e-4,
        "newest (B) amount = 20.0 * 0.5 = 10.0, got {amt_b}"
    );

    let amt_a = find_amount_for(&msgs, a).expect("echo message for A (oldest)");
    assert!(
        (amt_a - 2.0).abs() < 1e-4,
        "oldest (A) amount = 20.0 * 0.1 = 2.0, got {amt_a}"
    );

    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a, b, c],
        "network after: [A, B, C]"
    );
    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "EchoPrimed removed after primed impact"
    );
}

// ── Behavior 19 edge (19a) — newly impacted C not echo-damaged ─────────────-

#[test]
fn primed_impact_two_echoes_new_cell_not_in_echo_damage() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let b = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 20.0, vec![a, b]);
    let c = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, c, bolt);
    tick(&mut app);

    let msgs = collected_echo_strike_damage(&app);
    assert!(
        find_amount_for(&msgs, c).is_none(),
        "no echo-damage should target the impacted cell C this frame"
    );
}

// ── Behavior 20 — 3-echo at max: all three fractions + FIFO evict ──────────-

#[test]
fn primed_impact_three_echoes_full_falloff_and_fifo_eviction() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let b = spawn_cell_empty(&mut app);
    let c = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 20.0, vec![a, b, c]);
    let d = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, d, bolt);
    tick(&mut app);

    let msgs = collected_echo_strike_damage(&app);
    assert_eq!(
        msgs.len(),
        3,
        "3-echo network should emit exactly 3 messages"
    );

    let amt_c = find_amount_for(&msgs, c).expect("echo message for C (newest)");
    assert!(
        (amt_c - 10.0).abs() < 1e-4,
        "C (newest) amount = 20.0 * 0.5 = 10.0, got {amt_c}"
    );

    let amt_b = find_amount_for(&msgs, b).expect("echo message for B (middle)");
    assert!(
        (amt_b - 5.0).abs() < 1e-4,
        "B (middle) amount = 20.0 * 0.25 = 5.0, got {amt_b}"
    );

    let amt_a = find_amount_for(&msgs, a).expect("echo message for A (oldest)");
    assert!(
        (amt_a - 2.0).abs() < 1e-4,
        "A (oldest) amount = 20.0 * 0.1 = 2.0, got {amt_a}"
    );

    assert_eq!(
        read_echo_network(&app, bolt),
        vec![b, c, d],
        "network after: A FIFO-evicted, D pushed to back"
    );
    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "EchoPrimed removed after primed impact"
    );
}
