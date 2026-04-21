use super::{
    super::{
        super::system::{ECHO_STRIKE_SENTINEL, EchoPrimed},
        helpers::{
            build_echo_strike_app, build_echo_strike_app_no_config, collected_echo_strike_damage,
            find_amount_for, read_echo_network, spawn_bolt_primed, spawn_bolt_primed_with_network,
            spawn_cell_empty, write_bolt_impact_cell,
        },
    },
    helpers::seed_canonical,
};
use crate::prelude::*;

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
