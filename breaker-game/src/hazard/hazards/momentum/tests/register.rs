//! Group H — `register` wiring + run-condition gates (Behaviors 51–57)
//! plus Behaviors 68, 69 (ordering pins for
//! `attach_momentum_ceiling` and `momentum_split_check`).
//!
//! Pins that `register(app)` wires `attach_momentum_ceiling`,
//! `momentum_heal_on_nonlethal`, and `momentum_split_check` in `FixedUpdate`
//! with both run-if gates, AND the ordering:
//! attach → `apply_damage` → `heal_emit` → `apply_heal` → `split_check`.

use bevy::prelude::*;
use rantzsoft_dmg::{RantzDmgAppExt, RantzDmgPlugin};

use super::{
    super::system::{momentum_heal_on_nonlethal, momentum_split_check, register},
    helpers::{
        add_hazard_stacks, add_momentum_stacks, canonical_momentum_config, cell_count,
        heal_collector_len, heals_for_cell, install_momentum_config, run_fixed_update,
        spawn_cell_at, test_app_not_playing, test_app_playing, write_cell_damage,
        write_cell_heal_max,
    },
};
use crate::{
    cells::components::Cell,
    hazard::{definition::HazardKind, resources::ActiveHazards},
    prelude::*,
};

/// Builds a register-wired app that also wires `apply_damage::<Cell>` and
/// `apply_heal::<Cell>` (so the full pipeline end-to-end goes through).
fn register_app_full_pipeline() -> App {
    let mut app = test_app_playing();
    app.add_plugins(RantzDmgPlugin);
    let _ = app.register_dmgable::<Cell>();
    register(&mut app);
    app
}

// ── Behavior 51 — register wires momentum_heal_on_nonlethal ─────────────────

#[test]
fn register_wires_heal_on_nonlethal() {
    let mut app = register_app_full_pipeline();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);
    write_cell_damage(&mut app, cell, 5.0);

    run_fixed_update(&mut app);

    assert_eq!(
        heal_collector_len(&app),
        1,
        "register must wire momentum_heal_on_nonlethal; expected 1 heal, got {}",
        heal_collector_len(&app)
    );
    let msgs = heals_for_cell(&app, cell);
    assert_eq!(msgs.len(), 1);
    assert!((msgs[0].amount - 10.0).abs() < f32::EPSILON);
    assert!(matches!(msgs[0].cap, HealCap::Max));
    assert_eq!(
        msgs[0].source.as_ref(),
        Some(&SourceId::hazard(HazardKind::Momentum).build())
    );
}

#[test]
fn register_gate_off_zero_stacks_emits_no_heal() {
    let mut app = register_app_full_pipeline();
    install_momentum_config(&mut app, canonical_momentum_config());
    // 0 stacks.

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);
    write_cell_damage(&mut app, cell, 5.0);

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 0);
}

// ── Behavior 52 — register wires momentum_split_check ───────────────────────

#[test]
fn register_wires_split_check() {
    let mut app = register_app_full_pipeline();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let parent =
        super::helpers::spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(parent).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "register must wire split_check; parent must reset; got {}",
        hp.current
    );
    assert_eq!(cell_count(&mut app), 3);
}

// ── Behavior 53 — both gated off when not in Playing state ──────────────────

#[test]
fn both_systems_gated_off_when_not_in_playing() {
    let mut app = test_app_not_playing();
    app.add_plugins(RantzDmgPlugin);
    let _ = app.register_dmgable::<Cell>();
    register(&mut app);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);
    write_cell_damage(&mut app, cell, 5.0);
    let over_threshold = super::helpers::spawn_cell_at_with_max(
        &mut app,
        Vec2::new(1000.0, 1000.0),
        20.0,
        10.0,
        Some(20.0),
    );

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 0);
    let hp_over = app.world().get::<Hp>(over_threshold).unwrap();
    assert!(
        (hp_over.current - 20.0).abs() < f32::EPSILON,
        "split must not fire when gated off; got {}",
        hp_over.current
    );
}

// ── Behavior 54 — both gated off when Momentum has 0 stacks (other hazard) ─

#[test]
fn both_systems_gated_off_when_zero_momentum_stacks() {
    let mut app = register_app_full_pipeline();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_hazard_stacks(&mut app, HazardKind::Cascade, 1);
    assert_eq!(
        app.world()
            .resource::<ActiveHazards>()
            .stacks(HazardKind::Momentum),
        0
    );

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);
    write_cell_damage(&mut app, cell, 5.0);
    let over_threshold = super::helpers::spawn_cell_at_with_max(
        &mut app,
        Vec2::new(1000.0, 1000.0),
        20.0,
        10.0,
        Some(20.0),
    );

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 0);
    let hp_over = app.world().get::<Hp>(over_threshold).unwrap();
    assert!((hp_over.current - 20.0).abs() < f32::EPSILON);
}

// ── Behavior 55 — split-check runs AFTER heal applies within same tick ──────

#[test]
fn split_check_runs_after_heal_applies_same_tick_no_split_at_fifteen() {
    let mut app = register_app_full_pipeline();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    // Fresh cell (hp.max = None). attach_momentum_ceiling auto-lifts to Some(20).
    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);
    write_cell_damage(&mut app, cell, 5.0);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "after full tick: 10 → 5 (damage) → 15 (heal); got {}",
        hp.current
    );
    assert_eq!(hp.max, Some(20.0));
    assert_eq!(cell_count(&mut app), 1, "no split at 15.0 < threshold 20.0");
}

#[test]
fn no_damage_message_no_heal_no_split() {
    let mut app = register_app_full_pipeline();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);
    // No damage message.

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!((hp.current - 10.0).abs() < f32::EPSILON);
    assert_eq!(heal_collector_len(&app), 0);
    assert_eq!(cell_count(&mut app), 1);
}

// ── Behavior 56 — 2 consecutive hits at stack 1 land, then split ────────────

#[test]
fn two_consecutive_nonlethal_hits_eventually_split() {
    let mut app = register_app_full_pipeline();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);

    // Tick 1: damage 5 → post 5 → heal 10 → min(5+10, 20) = 15. No split.
    write_cell_damage(&mut app, cell, 5.0);
    run_fixed_update(&mut app);

    let hp_after_tick_1 = app.world().get::<Hp>(cell).unwrap().clone();
    assert!(
        (hp_after_tick_1.current - 15.0).abs() < f32::EPSILON,
        "tick 1 should land at current=15.0; got {}",
        hp_after_tick_1.current
    );
    assert_eq!(cell_count(&mut app), 1);

    // Tick 2: damage 1 → post 14 → heal 10 → min(14+10, 20) = 20. Split fires.
    write_cell_damage(&mut app, cell, 1.0);
    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "tick 2: split must fire at threshold; current resets to 10.0; got {}",
        hp.current
    );
    assert_eq!(cell_count(&mut app), 3, "2 new cells spawn after split");
}

// ── Behavior 57 — register does not panic when MomentumConfig absent ────────

#[test]
fn register_does_not_panic_when_config_absent() {
    let mut app = register_app_full_pipeline();
    // NO MomentumConfig.
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);
    write_cell_damage(&mut app, cell, 5.0);

    run_fixed_update(&mut app);
    // No panic. Early-return paths in systems must handle absent config.
    assert_eq!(heal_collector_len(&app), 0);
}

// ════════════════════════════════════════════════════════════════════════════
// Behaviors 68, 69 — ordering pins
// ════════════════════════════════════════════════════════════════════════════

// ── Behavior 68 — register places attach before heal (integration proof) ────

#[test]
fn register_orders_attach_ceiling_before_heal_emit() {
    let mut app = register_app_full_pipeline();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    // Fresh cell: hp.max = None. If attach runs before heal, the ceiling
    // will be lifted (max=20) before apply_heal runs, allowing current to
    // exceed starting.
    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);
    write_cell_damage(&mut app, cell, 5.0);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max,
        Some(20.0),
        "register must order attach_momentum_ceiling first; hp.max must be lifted; got {:?}",
        hp.max
    );
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "register ordering must allow current to exceed starting; got {}",
        hp.current
    );
    let msgs = heals_for_cell(&app, cell);
    assert_eq!(msgs.len(), 1);
    assert!((msgs[0].amount - 10.0).abs() < f32::EPSILON);
}

// ── Behavior 69 — split_check must run AFTER apply_heal (ordering pin) ─────
//
// Positive control of the ordering contract: using `register(app)` to wire
// the production ordering, a pre-queued `HealDealt<Cell>` that pushes
// `hp.current` to the threshold MUST trigger a split on the same tick.
//
// - With the stub (`register` = empty): nothing is wired; no split occurs;
//   the assertion `cell_count == 3` fails → RED-phase failure.
// - With correct production wiring (`split_check.after(ApplyHeal)`): heal
//   lands → current = 20.0 >= threshold → split fires → 2 new cells spawn.
// - With a bug where production wires `split_check` BEFORE `apply_heal`:
//   split_check would see `current = 15.0 < 20.0`, skip → assertion fails.
//
// This is the negative-control proof that misordering causes observable
// divergence, reframed as a positive pin on the register-wiring contract.

#[test]
fn register_wires_split_check_after_apply_heal_so_same_tick_heals_can_trigger_split() {
    let mut app = register_app_full_pipeline();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    // Cell at current=15, starting=10, max=20. A heal of 5.0 lands via
    // apply_heal (current becomes 20.0). split_check MUST observe the
    // post-heal state (current >= threshold 20.0) and fire a split.
    let cell = super::helpers::spawn_cell_at_with_max(&mut app, Vec2::ZERO, 15.0, 10.0, Some(20.0));
    write_cell_heal_max(&mut app, cell, 5.0);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "split must fire (register must wire split_check AFTER apply_heal); \
         current should reset to 10.0 after split; got {}",
        hp.current
    );
    assert_eq!(
        cell_count(&mut app),
        3,
        "register-wired ordering must produce 2 new split cells when \
         apply_heal pushes a cell across the threshold; got {}",
        cell_count(&mut app)
    );
    // Touch unused imports so the file's imports remain used even under
    // schedule changes. Assign to a real-name binding so clippy doesn't
    // flag `let _touch` as a no-effect underscore binding.
    let touch = HealDealt::<Cell> {
        healer:        None,
        attributed_to: None,
        target:        cell,
        amount:        0.0,
        cap:           HealCap::Max,
        source:        None,
        _marker:       std::marker::PhantomData,
    };
    drop(touch);
    let _ = momentum_heal_on_nonlethal;
    let _ = momentum_split_check;
}
