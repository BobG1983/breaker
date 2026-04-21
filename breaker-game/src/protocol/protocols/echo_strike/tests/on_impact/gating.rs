use super::{
    super::{
        super::system::{ECHO_STRIKE_SENTINEL, EchoPrimed},
        helpers::{
            build_echo_strike_app, build_echo_strike_app_in_chip_selecting,
            collected_echo_strike_damage, find_amount_for, read_echo_network,
            spawn_bolt_primed_with_network, spawn_cell_empty, write_bolt_impact_cell,
        },
    },
    helpers::seed_canonical,
};
use crate::prelude::*;

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
