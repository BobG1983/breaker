//! Group D — `echo_strike_on_impact` (Behaviors 16–33).
//!
//! Pins the `BoltImpactCell` consumer:
//! - Harness-safe early return + reader drain when `EchoStrikeConfig` absent.
//! - Primed empty network registers impact target, emits zero echo damage.
//! - 1/2/3-echo networks emit the correct age-based falloff (newest / middle /
//!   oldest fractions assigned by position, not fixed slot — 2-echo SKIPS
//!   middle).
//! - FIFO eviction at `max_echoes`.
//! - Same-cell dedup + push-to-back.
//! - `BoltBaseDamage` fallback to `DEFAULT_BOLT_BASE_DAMAGE`.
//! - Zero impact damage emits zero echo-damage messages (network still updated).
//! - `source_chip` sentinel + `dealer` pinned on every emitted message.
//! - `EchoPrimed` removed after a primed impact.
//! - Despawned bolt tolerated; non-primed bolt impact is a no-op.
//! - Multi-bolt isolation.
//! - Gating: Echo Strike inactive / `NodeState != Playing` → no-op.
//! - Single-shot on same-tick pierce — only first impact processed, second
//!   ignored even though `EchoPrimed` removal is deferred.
//!
//! NOTE: Behavior 30 (echo-damage for a destroyed echo cell in the same
//! tick) is implicitly covered by Behaviors 20 and 29 — the system reads
//! the current entity list regardless of victim health. No separate test.

use bevy::prelude::*;

use super::{
    super::system::{ECHO_STRIKE_SENTINEL, EchoNetwork, EchoPrimed},
    helpers::{
        build_echo_strike_app, build_echo_strike_app_in_chip_selecting,
        build_echo_strike_app_no_config, collected_echo_strike_damage, find_amount_for,
        read_echo_network, seed_active_protocols_with_echo_strike, spawn_bolt_primed,
        spawn_bolt_primed_with_network, spawn_bolt_with_echo_network, spawn_cell_empty,
        write_bolt_impact_cell,
    },
};
use crate::{bolt::resources::DEFAULT_BOLT_BASE_DAMAGE, prelude::*};

fn seed_canonical(app: &mut App) {
    // All system-behavior tests use 0.1 oldest (design-doc worked example).
    seed_active_protocols_with_echo_strike(app, 3, 0.5, 0.25, 0.1);
}

// ── Behavior 16 — on_impact early-returns + clears reader when config absent

#[test]
fn on_impact_early_returns_and_clears_reader_when_config_absent() {
    let mut app = build_echo_strike_app_no_config();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 10.0, vec![a]);
    let b = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, b, bolt);
    tick(&mut app); // tick 1 — config absent → early return

    assert!(
        app.world().get::<EchoPrimed>(bolt).is_some(),
        "EchoPrimed must remain when config absent"
    );
    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a],
        "network must be unchanged when config absent"
    );
    assert!(
        collected_echo_strike_damage(&app).is_empty(),
        "no echo damage emitted when config absent"
    );

    // Second quiet tick — buffered message must NOT be processed.
    tick(&mut app);
    assert!(
        app.world().get::<EchoPrimed>(bolt).is_some(),
        "EchoPrimed must still be present after quiet tick 2"
    );
    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a],
        "network must remain unchanged after quiet tick 2 (reader was drained)"
    );
    assert!(
        collected_echo_strike_damage(&app).is_empty(),
        "no echo damage emitted on quiet tick 2"
    );
}

// ── Behavior 17 — empty network: register impact; zero echo damage ─────────-

#[test]
fn primed_bolt_empty_network_registers_impact_no_damage() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 10.0, vec![]);
    let a = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, a, bolt);
    tick(&mut app);

    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a],
        "impacted cell should be registered as the first echo"
    );
    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "EchoPrimed should be removed after primed impact"
    );
    assert_eq!(
        collected_echo_strike_damage(&app).len(),
        0,
        "empty network emits zero echo-damage messages"
    );
}

// ── Behavior 17 edge (17a) — primed bolt with no EchoNetwork component ─────-

#[test]
fn primed_bolt_with_no_network_component_gets_network_on_demand() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    // Bolt has EchoPrimed + BoltBaseDamage but NO EchoNetwork.
    let bolt = spawn_bolt_primed(&mut app, 10.0);
    let a = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, a, bolt);
    tick(&mut app);

    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a],
        "protocol should attach EchoNetwork on demand with impacted cell as first echo"
    );
    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "EchoPrimed should be removed after primed impact"
    );
    assert_eq!(
        collected_echo_strike_damage(&app).len(),
        0,
        "first echo (no prior echoes) emits zero echo-damage messages"
    );
}

// ── Behavior 18 — 1-echo network: newest_fraction damage; push impact to back

#[test]
fn primed_impact_one_existing_echo_deals_newest_fraction_pushes_back() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 10.0, vec![a]);
    let b = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, b, bolt);
    tick(&mut app);

    let msgs = collected_echo_strike_damage(&app);
    assert_eq!(msgs.len(), 1, "expected exactly one echo-damage message");
    let msg = &msgs[0];
    assert_eq!(msg.dealer, Some(bolt), "dealer must be the impacting bolt");
    assert_eq!(msg.target, a, "target must be the sole echo A");
    assert!(
        (msg.amount - 5.0).abs() < 1e-4,
        "amount expected 5.0 (= 10.0 * 0.5 newest_fraction), got {}",
        msg.amount
    );
    assert_eq!(
        msg.source_chip.as_deref(),
        Some(ECHO_STRIKE_SENTINEL),
        "source_chip must be the Echo Strike sentinel"
    );

    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a, b],
        "impacted cell B should be pushed to the back"
    );
    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "EchoPrimed should be removed after primed impact"
    );
}

// ── Behavior 18 edge (18a) — newly-impacted cell is NOT echo-damaged ───────-

#[test]
fn primed_impact_newly_impacted_cell_not_in_echo_damage() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 10.0, vec![a]);
    let b = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, b, bolt);
    tick(&mut app);

    let msgs = collected_echo_strike_damage(&app);
    assert!(
        find_amount_for(&msgs, b).is_none(),
        "no DamageDealt<Cell> should target the impacted cell B this frame"
    );
}

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

// ── Behavior 21 — FIFO eviction fires only when exceeding max_echoes ───────-

#[test]
fn fifo_eviction_does_not_fire_when_below_max_echoes() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app); // max_echoes: 3
    let a = spawn_cell_empty(&mut app);
    let b = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 10.0, vec![a, b]);
    let c = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, c, bolt);
    tick(&mut app);

    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a, b, c],
        "network expected [A, B, C] — NO eviction because we are at max now, not over it"
    );
}

// ── Behavior 22 — non-primed bolt: no echo, no network mutation ────────────-

#[test]
fn non_primed_bolt_impact_no_echo_no_network_change() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let b = spawn_cell_empty(&mut app);
    // bolt has no EchoPrimed.
    let bolt = spawn_bolt_with_echo_network(&mut app, 10.0, vec![a, b]);
    let c = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, c, bolt);
    tick(&mut app);

    assert!(
        collected_echo_strike_damage(&app).is_empty(),
        "non-primed bolt must emit zero echo damage"
    );
    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a, b],
        "non-primed bolt's network must be unchanged"
    );
    assert!(
        !read_echo_network(&app, bolt).contains(&c),
        "impacted cell C must NOT be added to non-primed bolt's network"
    );
}

// ── Behavior 22 edge (22a) — non-primed with empty network ─────────────────-

#[test]
fn non_primed_bolt_impact_with_empty_network_remains_empty() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_echo_network(&mut app, 10.0, vec![]);
    let c = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, c, bolt);
    tick(&mut app);

    assert_eq!(
        read_echo_network(&app, bolt),
        Vec::<Entity>::new(),
        "non-primed bolt with empty network must remain empty"
    );
}

// ── Behavior 23 — zero base damage → zero echo-damage messages ─────────────-

#[test]
fn primed_impact_zero_base_damage_emits_zero_messages_but_registers_echo() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let b = spawn_cell_empty(&mut app);
    let c = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 0.0, vec![a, b, c]);
    let d = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, d, bolt);
    tick(&mut app);

    assert!(
        collected_echo_strike_damage(&app).is_empty(),
        "zero base damage must not emit any DamageDealt<Cell> messages"
    );
    // Echo registration and FIFO still happen regardless of damage amount.
    assert_eq!(
        read_echo_network(&app, bolt),
        vec![b, c, d],
        "echo registration + FIFO happens even with zero damage"
    );
    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "EchoPrimed removed regardless of damage amount"
    );
}

// ── Behavior 24 — bolt without BoltBaseDamage uses DEFAULT_BOLT_BASE_DAMAGE -

#[test]
fn primed_impact_without_bolt_base_damage_uses_default() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    // Spawn primed bolt WITHOUT BoltBaseDamage.
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            EchoPrimed,
            EchoNetwork {
                echoes: std::iter::once(a).collect(),
            },
        ))
        .id();
    let b = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, b, bolt);
    tick(&mut app);

    let msgs = collected_echo_strike_damage(&app);
    assert_eq!(msgs.len(), 1, "one echo message expected (1-echo network)");
    let expected = DEFAULT_BOLT_BASE_DAMAGE * 0.5; // = 10.0 * 0.5 = 5.0
    let amt = find_amount_for(&msgs, a).expect("echo message for A");
    assert!(
        (amt - expected).abs() < 1e-4,
        "amount expected {expected} (= DEFAULT_BOLT_BASE_DAMAGE * 0.5), got {amt}"
    );
}

// ── Behavior 25 — source_chip sentinel on every emitted message ────────────-

#[test]
fn every_emitted_echo_damage_message_has_echo_strike_sentinel() {
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
    assert_eq!(msgs.len(), 3, "3 messages expected from 3-echo network");
    for msg in &msgs {
        assert_eq!(
            msg.source_chip.as_deref(),
            Some(ECHO_STRIKE_SENTINEL),
            "every emitted echo-damage message must carry ECHO_STRIKE_SENTINEL"
        );
    }
}

// ── Behavior 26 — dealer is Some(bolt) on every emitted message ────────────-

#[test]
fn every_emitted_echo_damage_message_has_dealer_set_to_bolt() {
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
    assert_eq!(msgs.len(), 3);
    for msg in &msgs {
        assert_eq!(
            msg.dealer,
            Some(bolt),
            "every echo-damage message must have dealer = Some(bolt)"
        );
    }
}

// ── Behavior 27 — impact for despawned bolt is tolerated ───────────────────-

#[test]
fn impact_for_despawned_bolt_is_tolerated() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 10.0, vec![a]);
    app.world_mut().entity_mut(bolt).despawn();
    let b = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, b, bolt);
    tick(&mut app); // must not panic

    assert!(
        collected_echo_strike_damage(&app).is_empty(),
        "no echo damage for a despawned bolt (network gone)"
    );
}

// ── Behavior 28 — multi-bolt isolation ─────────────────────────────────────-

#[test]
fn primed_impact_does_not_touch_other_bolts_networks() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let cell_a = spawn_cell_empty(&mut app);
    let cell_p = spawn_cell_empty(&mut app);
    let cell_q = spawn_cell_empty(&mut app);
    let primed_bolt = spawn_bolt_primed_with_network(&mut app, 10.0, vec![cell_a]);
    let other_bolt = spawn_bolt_with_echo_network(&mut app, 10.0, vec![cell_p, cell_q]);
    let cell_b = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell_b, primed_bolt);
    tick(&mut app);

    assert_eq!(
        read_echo_network(&app, primed_bolt),
        vec![cell_a, cell_b],
        "primed_bolt's network should have A + B"
    );
    assert_eq!(
        read_echo_network(&app, other_bolt),
        vec![cell_p, cell_q],
        "other_bolt's network must be untouched"
    );

    let msgs = collected_echo_strike_damage(&app);
    assert_eq!(msgs.len(), 1, "exactly one echo-damage message");
    assert_eq!(
        msgs[0].target, cell_a,
        "target should be A (primed_bolt's sole echo)"
    );
    assert_eq!(
        msgs[0].dealer,
        Some(primed_bolt),
        "dealer should be primed_bolt"
    );
    assert!(
        find_amount_for(&msgs, cell_p).is_none() && find_amount_for(&msgs, cell_q).is_none(),
        "no echo-damage should target P or Q (other_bolt's echoes)"
    );

    assert!(
        app.world().get::<EchoPrimed>(primed_bolt).is_none(),
        "primed_bolt no longer primed"
    );
    assert!(
        app.world().get::<EchoPrimed>(other_bolt).is_none(),
        "other_bolt still not primed"
    );
}

// ── Behavior 29 — same cell re-hit (dedup + move to back) ──────────────────-

#[test]
fn primed_impact_on_existing_middle_echo_dedups_and_moves_to_back() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let b = spawn_cell_empty(&mut app);
    let c = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 20.0, vec![a, b, c]);

    // Re-hit the MIDDLE echo B.
    write_bolt_impact_cell(&mut app, b, bolt);
    tick(&mut app);

    let msgs = collected_echo_strike_damage(&app);
    assert_eq!(
        msgs.len(),
        3,
        "pre-move damage emitted for all three positions"
    );
    assert!(
        (find_amount_for(&msgs, a).expect("msg for A") - 2.0).abs() < 1e-4,
        "A (oldest) = 20.0 * 0.1 = 2.0"
    );
    assert!(
        (find_amount_for(&msgs, b).expect("msg for B") - 5.0).abs() < 1e-4,
        "B (middle) = 20.0 * 0.25 = 5.0"
    );
    assert!(
        (find_amount_for(&msgs, c).expect("msg for C") - 10.0).abs() < 1e-4,
        "C (newest) = 20.0 * 0.5 = 10.0"
    );

    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a, c, b],
        "network after: B removed from middle, C stays, B pushed to back"
    );
    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "EchoPrimed removed after primed impact"
    );
}

// ── Behavior 29 edge (29a) — re-hit newest echo ────────────────────────────-

#[test]
fn primed_impact_on_existing_newest_echo_is_ordering_no_op() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let b = spawn_cell_empty(&mut app);
    let c = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 20.0, vec![a, b, c]);

    write_bolt_impact_cell(&mut app, c, bolt);
    tick(&mut app);

    let msgs = collected_echo_strike_damage(&app);
    assert_eq!(msgs.len(), 3, "pre-move damage for all three positions");
    assert!(
        (find_amount_for(&msgs, a).expect("A") - 2.0).abs() < 1e-4
            && (find_amount_for(&msgs, b).expect("B") - 5.0).abs() < 1e-4
            && (find_amount_for(&msgs, c).expect("C") - 10.0).abs() < 1e-4
    );

    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a, b, c],
        "network after: [A, B, C] — dedup+push at the back is a no-op for newest"
    );
}

// ── Behavior 29 edge (29b) — re-hit oldest echo ────────────────────────────-

#[test]
fn primed_impact_on_existing_oldest_echo_moves_it_to_back() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let b = spawn_cell_empty(&mut app);
    let c = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 20.0, vec![a, b, c]);

    write_bolt_impact_cell(&mut app, a, bolt);
    tick(&mut app);

    let msgs = collected_echo_strike_damage(&app);
    assert_eq!(msgs.len(), 3);
    assert!(
        (find_amount_for(&msgs, a).expect("A") - 2.0).abs() < 1e-4
            && (find_amount_for(&msgs, b).expect("B") - 5.0).abs() < 1e-4
            && (find_amount_for(&msgs, c).expect("C") - 10.0).abs() < 1e-4
    );

    assert_eq!(
        read_echo_network(&app, bolt),
        vec![b, c, a],
        "A removed from front, pushed to back; B/C retained in order"
    );
}

// ── Behavior 31 — on_impact with Echo Strike inactive does nothing ─────────-

#[test]
fn on_impact_with_echo_strike_inactive_does_nothing() {
    let mut app = build_echo_strike_app();
    // Intentionally DO NOT seed ActiveProtocols.
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 10.0, vec![a]);
    let b = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, b, bolt);
    tick(&mut app);

    assert!(
        collected_echo_strike_damage(&app).is_empty(),
        "no echo damage when Echo Strike inactive"
    );
    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a],
        "network unchanged when system gated off"
    );
    assert!(
        app.world().get::<EchoPrimed>(bolt).is_some(),
        "EchoPrimed must remain — system never ran"
    );
}

// ── Behavior 32 — on_impact with NodeState != Playing does nothing ─────────-

#[test]
fn on_impact_with_node_state_not_playing_does_nothing() {
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
    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a],
        "network unchanged in ChipSelecting"
    );
    assert!(
        app.world().get::<EchoPrimed>(bolt).is_some(),
        "EchoPrimed must remain in ChipSelecting"
    );
}

// ── Behavior 33 — same-tick pierce single-shot guard ───────────────────────-

#[test]
fn primed_bolt_receives_two_impacts_same_tick_single_shot() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 10.0, vec![a]);
    let c = spawn_cell_empty(&mut app);
    let d = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, c, bolt);
    write_bolt_impact_cell(&mut app, d, bolt);
    tick(&mut app);

    let msgs = collected_echo_strike_damage(&app);
    assert_eq!(
        msgs.len(),
        1,
        "exactly one echo-damage message — pierce guard prevents double emit"
    );
    let msg = &msgs[0];
    assert_eq!(msg.target, a, "target is the only existing echo (A)");
    assert!(
        (msg.amount - 5.0).abs() < 1e-4,
        "amount = 10.0 * 0.5 = 5.0, got {}",
        msg.amount
    );
    assert_eq!(msg.dealer, Some(bolt));
    assert_eq!(msg.source_chip.as_deref(), Some(ECHO_STRIKE_SENTINEL));

    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a, c],
        "only the first impact (C) registered; D ignored"
    );
    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "EchoPrimed removed after the first impact"
    );
}

// ── Behavior 33 edge (33a) — same-tick pierce with 2 existing echoes ───────-

#[test]
fn primed_bolt_two_impacts_same_tick_with_two_existing_echoes() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let b_echo = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 20.0, vec![a, b_echo]);
    let c = spawn_cell_empty(&mut app);
    let d = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, c, bolt);
    write_bolt_impact_cell(&mut app, d, bolt);
    tick(&mut app);

    let msgs = collected_echo_strike_damage(&app);
    assert_eq!(msgs.len(), 2, "two echo messages for 2-echo network");
    let amt_b = find_amount_for(&msgs, b_echo).expect("msg for B_echo");
    assert!(
        (amt_b - 10.0).abs() < 1e-4,
        "B_echo (newest) amount = 20.0 * 0.5 = 10.0, got {amt_b}"
    );
    let amt_a = find_amount_for(&msgs, a).expect("msg for A");
    assert!(
        (amt_a - 2.0).abs() < 1e-4,
        "A (oldest) amount = 20.0 * 0.1 = 2.0, got {amt_a}"
    );
    assert!(
        find_amount_for(&msgs, c).is_none() && find_amount_for(&msgs, d).is_none(),
        "neither C nor D should be echo-damaged this frame"
    );

    assert_eq!(
        read_echo_network(&app, bolt),
        vec![a, b_echo, c],
        "only the first impact (C) registered; D ignored"
    );
    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "EchoPrimed removed after the first impact"
    );
}
