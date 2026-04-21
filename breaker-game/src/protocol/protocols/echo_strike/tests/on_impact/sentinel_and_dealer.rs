use super::{
    super::{
        super::system::ECHO_STRIKE_SENTINEL,
        helpers::{
            build_echo_strike_app, collected_echo_strike_damage, spawn_bolt_primed_with_network,
            spawn_cell_empty, write_bolt_impact_cell,
        },
    },
    helpers::seed_canonical,
};
use crate::prelude::*;

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
