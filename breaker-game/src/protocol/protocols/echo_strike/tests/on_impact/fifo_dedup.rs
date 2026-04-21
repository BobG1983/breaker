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
