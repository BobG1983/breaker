use bevy::prelude::*;

use super::{
    super::{
        super::system::{EchoNetwork, EchoPrimed},
        helpers::{
            build_echo_strike_app, collected_echo_strike_damage, find_amount_for,
            read_echo_network, spawn_bolt_primed_with_network, spawn_bolt_with_echo_network,
            spawn_cell_empty, write_bolt_impact_cell,
        },
    },
    helpers::seed_canonical,
};
use crate::{bolt::resources::DEFAULT_BOLT_BASE_DAMAGE, prelude::*};

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
