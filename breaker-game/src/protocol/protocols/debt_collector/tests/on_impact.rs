//! Group D — `debt_collector_on_impact` (Behaviors 16–23).
//!
//! Pins the `BoltImpactCell` consumer:
//! - Emits bonus `DamageDealt<Cell>` with `amount = base_damage * stack`.
//! - Falls back to `DEFAULT_BOLT_BASE_DAMAGE` when `BoltBaseDamage` absent.
//! - One-shot cross-frame + same-frame (pierce guard).
//! - No cash-out → no bonus.
//! - Zero-stack cash-out emits a zero-amount bonus.
//! - Multi-bolt isolation.
//! - Untracked bolts tolerated.

use super::{
    super::system::{DEBT_COLLECTOR_SENTINEL, DebtCashOut, DebtStack},
    helpers::{
        build_debt_collector_app, collected_bonus_damage, install_debt_cash_out,
        install_debt_stack, seed_active_protocols_with_debt_collector,
        spawn_bolt_with_base_damage_and_cashout, write_bolt_impact_cell,
    },
};
use crate::{
    bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE},
    prelude::*,
};

// ── Behavior 16 — cash-out bolt emits bonus on next impact ──────────────────

#[test]
fn cash_out_bolt_emits_bonus_damage_dealt_on_impact() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_base_damage_and_cashout(&mut app, 10.0, 1.5);
    let cell = app.world_mut().spawn_empty().id();

    write_bolt_impact_cell(&mut app, bolt, cell);
    tick(&mut app);

    let bonuses = collected_bonus_damage(&app);
    assert_eq!(
        bonuses.len(),
        1,
        "expected exactly ONE bonus DamageDealt<Cell>, got {}",
        bonuses.len()
    );
    let msg = &bonuses[0];
    assert_eq!(
        msg.dealer,
        Some(bolt),
        "dealer should be the impacting bolt"
    );
    assert_eq!(msg.target, cell, "target should be the impacted cell");
    assert!(
        (msg.amount - 15.0).abs() < 1e-4,
        "amount = 10.0 * 1.5 = 15.0, got {}",
        msg.amount
    );
    assert_eq!(
        msg.source_chip.as_deref(),
        Some(DEBT_COLLECTOR_SENTINEL),
        "source_chip drift guard: expected \"protocol:debt_collector\""
    );

    // DebtCashOut removed; DebtStack unaffected (it wasn't present anyway).
    assert!(
        app.world().get::<DebtCashOut>(bolt).is_none(),
        "DebtCashOut should be removed after cash-out"
    );
}

// ── Behavior 17 — absent BoltBaseDamage falls back to DEFAULT ──────────────-

#[test]
fn bonus_uses_default_base_damage_when_bolt_base_damage_absent() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    // Spawn a bolt WITHOUT BoltBaseDamage but WITH DebtCashOut.
    let bolt = app.world_mut().spawn(Bolt).id();
    install_debt_cash_out(&mut app, bolt, 2.0);
    let cell = app.world_mut().spawn_empty().id();

    write_bolt_impact_cell(&mut app, bolt, cell);
    tick(&mut app);

    let bonuses = collected_bonus_damage(&app);
    assert_eq!(bonuses.len(), 1, "expected exactly one bonus");
    let expected = DEFAULT_BOLT_BASE_DAMAGE * 2.0; // 10.0 * 2.0 = 20.0
    assert!(
        (bonuses[0].amount - expected).abs() < 1e-4,
        "amount expected {} (default 10.0 × stack 2.0), got {}",
        expected,
        bonuses[0].amount
    );
    assert_eq!(
        bonuses[0].source_chip.as_deref(),
        Some(DEBT_COLLECTOR_SENTINEL)
    );
}

// ── Behavior 18 — cash-out is one-shot; second impact produces no bonus ────-

#[test]
fn cash_out_is_one_shot_second_impact_produces_no_bonus() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_base_damage_and_cashout(&mut app, 10.0, 1.5);
    let cell = app.world_mut().spawn_empty().id();

    // First impact frame.
    write_bolt_impact_cell(&mut app, bolt, cell);
    tick(&mut app);
    let bonuses = collected_bonus_damage(&app);
    assert_eq!(bonuses.len(), 1, "frame 1 must emit exactly one bonus");

    // DebtCashOut removed.
    assert!(app.world().get::<DebtCashOut>(bolt).is_none());

    // Second impact frame — no new cash-out, no new bonus.
    write_bolt_impact_cell(&mut app, bolt, cell);
    tick(&mut app);
    let bonuses = collected_bonus_damage(&app);
    assert_eq!(
        bonuses.len(),
        0,
        "second impact frame must emit zero bonuses (cash-out was removed after frame 1); got {}",
        bonuses.len()
    );
}

// ── Behavior 18 (edge case) — same-frame pierce emits exactly one bonus ────-

#[test]
fn same_frame_pierce_emits_exactly_one_bonus() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_base_damage_and_cashout(&mut app, 10.0, 1.5);
    let cell_a = app.world_mut().spawn_empty().id();
    let cell_b = app.world_mut().spawn_empty().id();

    // Two BoltImpactCell messages in the same frame for the same bolt.
    write_bolt_impact_cell(&mut app, bolt, cell_a);
    write_bolt_impact_cell(&mut app, bolt, cell_b);
    tick(&mut app);

    let bonuses = collected_bonus_damage(&app);
    assert_eq!(
        bonuses.len(),
        1,
        "two same-frame impacts must produce exactly ONE bonus (pierce guard); got {}",
        bonuses.len()
    );
}

// ── Behavior 19 — bolt without DebtCashOut produces no bonus ───────────────-

#[test]
fn bolt_without_debt_cash_out_produces_no_bonus() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    // Has BoltBaseDamage + DebtStack but NO DebtCashOut.
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltBaseDamage(10.0), DebtStack(0.5)))
        .id();
    let cell = app.world_mut().spawn_empty().id();

    write_bolt_impact_cell(&mut app, bolt, cell);
    tick(&mut app);

    let bonuses = collected_bonus_damage(&app);
    assert!(
        bonuses.is_empty(),
        "no DebtCashOut → no Debt Collector bonus emitted; got {}",
        bonuses.len()
    );

    let stack = app.world().get::<DebtStack>(bolt).expect("stack retained");
    assert!(
        (stack.0 - 0.5).abs() < f32::EPSILON,
        "DebtStack unchanged by on_impact, got {}",
        stack.0
    );
}

// ── Behavior 20 — zero-stack cash-out emits bonus with amount 0.0 ──────────-

#[test]
fn zero_stack_cash_out_emits_bonus_with_amount_zero() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_base_damage_and_cashout(&mut app, 10.0, 0.0);
    let cell = app.world_mut().spawn_empty().id();

    write_bolt_impact_cell(&mut app, bolt, cell);
    tick(&mut app);

    let bonuses = collected_bonus_damage(&app);
    assert_eq!(
        bonuses.len(),
        1,
        "zero-stack cash-out still emits one bonus"
    );
    let msg = &bonuses[0];
    assert!(
        (msg.amount - 0.0).abs() < f32::EPSILON,
        "amount should be 0.0 for zero-stack cash-out, got {}",
        msg.amount
    );
    assert_eq!(msg.dealer, Some(bolt));
    assert_eq!(msg.target, cell);
    assert_eq!(msg.source_chip.as_deref(), Some(DEBT_COLLECTOR_SENTINEL));
    assert!(
        app.world().get::<DebtCashOut>(bolt).is_none(),
        "DebtCashOut removed even when amount is zero"
    );
}

// ── Behavior 21 — multiple bolts each emit their own bonus ─────────────────-

#[test]
fn multiple_bolts_each_emit_their_own_bonus() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let a = spawn_bolt_with_base_damage_and_cashout(&mut app, 10.0, 1.5);
    let b = spawn_bolt_with_base_damage_and_cashout(&mut app, 25.0, 0.4);
    let cell_a = app.world_mut().spawn_empty().id();
    let cell_b = app.world_mut().spawn_empty().id();

    write_bolt_impact_cell(&mut app, a, cell_a);
    write_bolt_impact_cell(&mut app, b, cell_b);
    tick(&mut app);

    let bonuses = collected_bonus_damage(&app);
    assert_eq!(
        bonuses.len(),
        2,
        "two cash-out bolts must produce 2 bonuses, got {}",
        bonuses.len()
    );

    let msg_a = bonuses
        .iter()
        .find(|m| m.dealer == Some(a))
        .expect("bonus for bolt a should exist");
    assert_eq!(msg_a.target, cell_a);
    assert!(
        (msg_a.amount - 15.0).abs() < 1e-4,
        "bolt A amount = 10.0 × 1.5 = 15.0, got {}",
        msg_a.amount
    );
    assert_eq!(msg_a.source_chip.as_deref(), Some(DEBT_COLLECTOR_SENTINEL));

    let msg_b = bonuses
        .iter()
        .find(|m| m.dealer == Some(b))
        .expect("bonus for bolt b should exist");
    assert_eq!(msg_b.target, cell_b);
    assert!(
        (msg_b.amount - 10.0).abs() < 1e-4,
        "bolt B amount = 25.0 × 0.4 = 10.0, got {}",
        msg_b.amount
    );
    assert_eq!(msg_b.source_chip.as_deref(), Some(DEBT_COLLECTOR_SENTINEL));
}

// ── Behavior 22 — untracked bolt is tolerated ──────────────────────────────-

#[test]
fn untracked_bolt_impact_is_tolerated() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    // Bare bolt with no DebtStack, no DebtCashOut, no BoltBaseDamage.
    let bolt = app.world_mut().spawn(Bolt).id();
    let cell = app.world_mut().spawn_empty().id();

    write_bolt_impact_cell(&mut app, bolt, cell);
    tick(&mut app); // must not panic

    let bonuses = collected_bonus_damage(&app);
    assert!(
        bonuses.is_empty(),
        "untracked bolt impact must not emit a bonus, got {}",
        bonuses.len()
    );
}

// ── Behavior 23 — on_impact no-op when no bolt has DebtCashOut ─────────────-

#[test]
fn on_impact_is_no_op_when_no_bolt_has_debt_cash_out() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    // Bolt with base damage but NO DebtCashOut.
    let bolt = app.world_mut().spawn((Bolt, BoltBaseDamage(10.0))).id();
    install_debt_stack(&mut app, bolt, 0.0);
    let cell = app.world_mut().spawn_empty().id();

    write_bolt_impact_cell(&mut app, bolt, cell);
    tick(&mut app);

    assert!(
        collected_bonus_damage(&app).is_empty(),
        "no DebtCashOut on any bolt → zero bonuses"
    );
    assert!(
        app.world().get::<DebtCashOut>(bolt).is_none(),
        "bolt should still have no DebtCashOut"
    );
    let base = app
        .world()
        .get::<BoltBaseDamage>(bolt)
        .expect("base damage retained");
    assert!(
        (base.0 - 10.0).abs() < f32::EPSILON,
        "BoltBaseDamage unchanged, got {}",
        base.0
    );

    // Edge case: second quiet frame — still no-op.
    tick(&mut app);
    assert!(collected_bonus_damage(&app).is_empty());
}
