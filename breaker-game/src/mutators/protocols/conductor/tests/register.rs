//! Group G — `register` wiring + run-condition gates (Behaviors 14–19).
//!
//! Pins that the Conductor swap system runs only when Conductor is in
//! `ActiveProtocols` AND the node state is `Playing`; that a `BumpPerformed`
//! message written in the same tick within `BreakerSystems::GradeBump` is
//! visible to the swap system via `.after(BreakerSystems::GradeBump)`; that
//! quiet ticks do not panic or disturb state; and that pre-gate buffered
//! `BumpPerformed` messages ARE replayed when the gate opens (baseline Bevy
//! retained-reader behavior — see todo
//! `docs/todos/detail/2026-04-20-protocol-pregate-message-drain.md` for the
//! cross-cutting fix tracking this class).

use bevy::prelude::*;

use super::{
    super::system::ConductorConfig,
    helpers::{
        bound_fingerprints, build_conductor_app, build_conductor_app_in_chip_selecting, has_extra,
        has_primary, make_distinct_bound, seed_active_protocols_with_conductor,
        seed_active_protocols_with_greed, spawn_dummy_breaker, spawn_extra_bolt_with_bound,
        spawn_primary_bolt_with_bound, write_bump_performed,
    },
};
use crate::{
    breaker::{messages::BumpGrade, sets::BreakerSystems},
    prelude::*,
};

// ── Behavior 14 — swap wired + gated on active + Playing ────────────────────

#[test]
fn register_wires_swap_gated_on_active_and_playing() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));

    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    assert!(has_primary(&app, extra), "swap ran — extra promoted");
    assert!(has_extra(&app, primary), "swap ran — primary demoted");
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["PRIMARY_BOUND".to_string()],
        "swap ran — extra received primary's bound"
    );
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["EXTRA_BOUND".to_string()],
        "swap ran — primary received extra's bound"
    );
}

// ── Behavior 15 — swap gated OFF when Conductor NOT in ActiveProtocols ──────

#[test]
fn swap_gated_off_when_conductor_not_active() {
    let mut app = build_conductor_app();
    // Do NOT seed ActiveProtocols.
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));

    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        has_primary(&app, primary),
        "primary unchanged — swap gated off"
    );
    assert!(has_extra(&app, extra), "extra unchanged — swap gated off");
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["PRIMARY_BOUND".to_string()],
        "primary's bound unchanged"
    );
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["EXTRA_BOUND".to_string()],
        "extra's bound unchanged"
    );
}

// ── Behavior 15 (edge case) — different protocol active does NOT open gate ──

#[test]
fn swap_gated_off_when_only_another_protocol_is_active() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_greed(&mut app);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));

    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        has_primary(&app, primary),
        "primary unchanged — not Conductor"
    );
    assert!(has_extra(&app, extra), "extra unchanged — not Conductor");
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["PRIMARY_BOUND".to_string()]
    );
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["EXTRA_BOUND".to_string()]
    );
}

// ── Behavior 16 — swap gated OFF when NodeState != Playing (ChipSelecting) ──

#[test]
fn swap_gated_off_when_node_state_is_chip_selecting() {
    let mut app = build_conductor_app_in_chip_selecting();
    seed_active_protocols_with_conductor(&mut app);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));

    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        has_primary(&app, primary),
        "primary unchanged in ChipSelecting"
    );
    assert!(has_extra(&app, extra), "extra unchanged in ChipSelecting");
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["PRIMARY_BOUND".to_string()]
    );
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["EXTRA_BOUND".to_string()]
    );
}

// ── Behavior 17 — swap consumes BumpPerformed same tick via .after(GradeBump)

#[test]
fn register_wires_swap_to_consume_bump_performed_same_tick() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));

    // Ghost producer in BreakerSystems::GradeBump writes BumpPerformed once.
    app.add_systems(
        FixedUpdate,
        (move |mut w: MessageWriter<BumpPerformed>, mut done: Local<bool>| {
            if *done {
                return;
            }
            w.write(BumpPerformed {
                grade: BumpGrade::Perfect,
                bolt: Some(extra),
                breaker,
            });
            *done = true;
        })
        .in_set(BreakerSystems::GradeBump)
        .run_if(in_state(NodeState::Playing)),
    );

    tick(&mut app);

    assert!(
        has_primary(&app, extra),
        "BumpPerformed written inside BreakerSystems::GradeBump must be visible \
         to conductor_swap_on_perfect_bump on the same FixedUpdate tick"
    );
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["PRIMARY_BOUND".to_string()],
        "swap ran on same tick — bound moved to bumped bolt"
    );
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["EXTRA_BOUND".to_string()],
        "swap ran on same tick — bound moved to old primary"
    );
}

// ── Behavior 18 — quiet-tick safety ─────────────────────────────────────────

#[test]
fn register_schedule_ticks_cleanly_with_no_bolts_and_no_messages() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app);

    for _ in 0..3 {
        tick(&mut app);
    }

    assert_eq!(
        *app.world().resource::<ConductorConfig>(),
        ConductorConfig,
        "ConductorConfig must remain canonical across quiet ticks"
    );
}

// ── Behavior 18 (edge case) — quiet ticks with a single primary bolt ────────

#[test]
fn register_schedule_ticks_cleanly_with_one_primary_and_no_messages() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));

    for _ in 0..3 {
        tick(&mut app);
    }

    assert!(has_primary(&app, primary), "primary marker unchanged");
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["PRIMARY_BOUND".to_string()],
        "primary's bound unchanged"
    );
}

// ── Behavior 19 — pre-gate BumpPerformed drains cleanly before activation
//    Regression pin against accidental `.run_if` reintroduction on the
//    reader system: `conductor_swap_on_perfect_bump` now enforces its
//    gate in-body via `reader.clear()` so pre-gate `BumpPerformed`
//    messages drain cleanly instead of replaying on gate open.

#[test]
fn pregate_bump_performed_drains_cleanly_before_conductor_activates() {
    let mut app = build_conductor_app();
    // Gate closed — ActiveProtocols empty (Conductor not active).
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));

    // Tick 1 — buffer a Perfect bump with gate closed. Retrofit drains reader.
    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);
    assert!(has_primary(&app, primary), "tick 1: no swap (gate closed)");
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["PRIMARY_BOUND".to_string()]
    );
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["EXTRA_BOUND".to_string()]
    );

    // Tick 2 — open the gate, no new messages. The pre-gate message was
    // drained on tick 1; nothing remains to replay → no swap.
    seed_active_protocols_with_conductor(&mut app);
    tick(&mut app);

    assert!(
        has_primary(&app, primary),
        "tick 2: pre-gate message already drained → primary unchanged"
    );
    assert!(
        has_extra(&app, extra),
        "tick 2: pre-gate message already drained → extra unchanged"
    );
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["PRIMARY_BOUND".to_string()],
        "tick 2: primary's bound unchanged — no retroactive swap"
    );
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["EXTRA_BOUND".to_string()],
        "tick 2: extra's bound unchanged — no retroactive swap"
    );
}

// ── Behavior 19 (edge case) — multiple pre-gate messages all drain cleanly.
//    Regression pin against accidental `.run_if` reintroduction.

#[test]
fn multiple_pregate_bump_performed_drain_cleanly_no_swap_on_activation() {
    let mut app = build_conductor_app();
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));

    // Tick 1 — three buffered Perfect bumps, gate closed. Retrofit drains reader.
    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    // Tick 2 — open the gate, no new messages. All three pre-gate messages
    // were drained on tick 1; nothing remains to replay → no swap.
    seed_active_protocols_with_conductor(&mut app);
    tick(&mut app);

    assert!(
        has_primary(&app, primary),
        "primary unchanged — pre-gate messages already drained"
    );
    assert!(
        has_extra(&app, extra),
        "extra unchanged — pre-gate messages already drained"
    );
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["PRIMARY_BOUND".to_string()],
        "primary's bound unchanged — no retroactive swap"
    );
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["EXTRA_BOUND".to_string()],
        "extra's bound unchanged — no retroactive swap"
    );
}
