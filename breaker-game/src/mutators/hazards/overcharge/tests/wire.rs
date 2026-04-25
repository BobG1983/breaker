//! Group F — `wire`-wired integration and gating.
//!
//! These use `wire(&mut app)` to test the wiring end-to-end, NOT
//! hand-wired systems. They exercise the full three-system chain
//! (`count_kills → reset_on_bump → apply_speed`) in one `FixedUpdate` tick.

use ordered_float::OrderedFloat;

use super::{
    super::system::{OverchargeKillCount, wire},
    helpers::{
        add_overcharge_stacks, canonical_config, install_overcharge_config, overcharge_entries,
        run_fixed_update, spawn_bolt, spawn_bolt_with_stack, spawn_cell, test_app_not_playing,
        test_app_playing, write_bump, write_cell_destroyed,
    },
};
use crate::{
    chips::definition::Rarity,
    effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack},
    mutators::hazards::{definition::HazardKind, resources::ActiveHazards},
    prelude::*,
};

fn hazard_overcharge() -> SourceId {
    SourceId::hazard(HazardKind::Overcharge).build()
}

fn chip_overclock() -> SourceId {
    SourceId::chip("Overclock").rarity(Rarity::Common).build()
}

// ── Behavior 39 — full chain: kill → count → apply speed in one tick ─────

#[test]
fn full_chain_kill_counts_and_applies_speed_in_one_tick() {
    let mut app = test_app_playing();
    wire(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);
    let cell = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell, Some(bolt));

    run_fixed_update(&mut app);

    let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
    assert_eq!(count.0, 1);
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("Overcharge entry must be attached after the chain runs");
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 1.05).abs() < 1e-6);
    let entries = overcharge_entries(stack);
    assert_eq!(entries[0].0, hazard_overcharge());
}

#[test]
fn full_chain_is_idempotent_across_quiescent_tick() {
    // Edge: a second tick with NO new messages leaves count at 1,
    // stack len at 1, and aggregate at 1.05.
    let mut app = test_app_playing();
    wire(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);
    let cell = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell, Some(bolt));

    run_fixed_update(&mut app);
    run_fixed_update(&mut app);

    let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
    assert_eq!(count.0, 1);
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 1.05).abs() < 1e-6);
}

// ── Behavior 40 — full chain: bump after kills resets count + removes entry ─

#[test]
fn full_chain_bump_after_kill_resets_count_and_removes_entry() {
    let mut app = test_app_playing();
    wire(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);
    let cell = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell, Some(bolt));

    run_fixed_update(&mut app);
    // After first tick: count == 1, aggregate == 1.05. Now bump.
    write_bump(&mut app, Some(bolt));
    run_fixed_update(&mut app);

    let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
    assert_eq!(count.0, 0);
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 0, "Overcharge entry removed when kills = 0");
}

#[test]
fn full_chain_intra_tick_ordering_kill_reset_apply() {
    // Edge: in one tick, a Destroyed<Cell> + a BumpPerformed queue up.
    // Per `wire` ordering count → reset → apply: count becomes 1,
    // reset sets to 0, apply sees zero kills and inserts no entry.
    // Final: count == 0, stack len == 0.
    let mut app = test_app_playing();
    wire(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);
    let cell = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell, Some(bolt));
    write_bump(&mut app, Some(bolt));

    run_fixed_update(&mut app);

    let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
    assert_eq!(count.0, 0);
    // EffectStack may or may not exist — what matters is no Overcharge entry.
    let stack_opt = app.world().get::<EffectStack<SpeedBoostConfig>>(bolt);
    if let Some(stack) = stack_opt {
        let entries = overcharge_entries(stack);
        assert!(
            entries.iter().all(|(s, _)| s != &hazard_overcharge()),
            "intra-tick: reset must fire before apply → no Overcharge entry"
        );
    }
}

// ── Behavior 41 — NOT in NodeState::Playing → none of three run ──────────

#[test]
fn state_gate_suppresses_all_three_systems() {
    let mut app = test_app_not_playing();
    wire(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);
    let cell = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell, Some(bolt));

    run_fixed_update(&mut app);

    assert!(app.world().get::<OverchargeKillCount>(bolt).is_none());
    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none()
    );
}

#[test]
fn state_gate_also_suppresses_reset_on_bump() {
    // Edge: with a bump in the same suppressed tick, still no components
    // attached — the reset_on_bump system is also gated off.
    let mut app = test_app_not_playing();
    wire(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);
    let cell = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell, Some(bolt));
    write_bump(&mut app, Some(bolt));

    run_fixed_update(&mut app);

    assert!(app.world().get::<OverchargeKillCount>(bolt).is_none());
    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none()
    );
}

// ── Behavior 42 — hazard_active(Overcharge) false → none of three run ───

#[test]
fn hazard_inactive_gate_suppresses_all_three_systems() {
    let mut app = test_app_playing();
    wire(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    // Stack Decay instead, to confirm it's the per-kind gate that fires.
    for _ in 0..3 {
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Decay);
    }
    assert_eq!(
        app.world()
            .resource::<ActiveHazards>()
            .stacks(HazardKind::Overcharge),
        0
    );
    let bolt = spawn_bolt(&mut app);
    let cell = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell, Some(bolt));

    run_fixed_update(&mut app);

    assert!(app.world().get::<OverchargeKillCount>(bolt).is_none());
    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none()
    );
}

#[test]
fn hazard_inactive_gate_toggles_on_once_overcharge_stack_added() {
    // Edge: add 1 Overcharge stack and a fresh Destroyed<Cell> → next
    // tick the bolt has count(1) and an Overcharge entry at 1.05.
    //
    // Note: we deliberately do NOT write a `Destroyed<Cell>` during the
    // gated-off tick. The retrofit drains the reader in-body when the
    // gate is closed, so pre-gate messages do not accumulate — pinned
    // by `pregate_messages_drain_cleanly_before_gate_opens` below.
    // Here we isolate the gate toggle with no pre-gate writes.
    let mut app = test_app_playing();
    wire(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    for _ in 0..3 {
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Decay);
    }
    let bolt = spawn_bolt(&mut app);

    run_fixed_update(&mut app);
    // Gate off: no components attached.
    assert!(app.world().get::<OverchargeKillCount>(bolt).is_none());

    // Toggle gate on, then write one fresh message.
    add_overcharge_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell, Some(bolt));
    run_fixed_update(&mut app);

    let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
    assert_eq!(count.0, 1);
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert!((stack.aggregate() - 1.05).abs() < 1e-6);
}

#[test]
fn pregate_messages_drain_cleanly_before_gate_opens() {
    // Regression pin against accidental `.run_if` reintroduction on the
    // reader systems: `overcharge_count_kills` / `overcharge_reset_on_bump`
    // now enforce their gate in-body via `reader.clear()` so pre-gate
    // messages drain cleanly instead of accumulating and replaying on
    // gate open.
    //
    // Setup: gate starts CLOSED (Decay stacks only, no Overcharge stack).
    // Write one Destroyed<Cell>. Tick (gate off → drain). Toggle gate ON
    // without writing a new message. Tick again. The pre-gate message
    // must NOT be consumed → no OverchargeKillCount component inserted,
    // and no Overcharge speed-stack entry appears.
    let mut app = test_app_playing();
    wire(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    for _ in 0..3 {
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Decay);
    }
    let bolt = spawn_bolt(&mut app);

    // Tick 1 — gate off, pre-gate kill written. Retrofit drains reader.
    let cell = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell, Some(bolt));
    run_fixed_update(&mut app);
    assert!(
        app.world().get::<OverchargeKillCount>(bolt).is_none(),
        "gate closed → no component inserted this tick"
    );

    // Tick 2 — gate opens; no new message written. The pre-gate message
    // was drained on tick 1; nothing remains to replay.
    add_overcharge_stacks(&mut app, 1);
    run_fixed_update(&mut app);

    assert!(
        app.world().get::<OverchargeKillCount>(bolt).is_none(),
        "pre-gate Destroyed<Cell> was drained on tick 1 — no \
         OverchargeKillCount may be inserted retroactively"
    );
    // No speed-stack entry either — without kills, `overcharge_apply_speed`
    // has nothing to push.
    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(bolt);
    if let Some(stack) = stack {
        assert!(
            stack.is_empty() || !stack.iter().any(|(s, _)| s == &hazard_overcharge()),
            "no 'hazard:overcharge' speed-stack entry may appear from a \
             pre-gate drained message"
        );
    }
}

// ── Behavior 43 — ActiveHazards totally empty → none of three run ────────

#[test]
fn empty_active_hazards_suppresses_all_three_systems() {
    let mut app = test_app_playing();
    wire(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    // ActiveHazards left untouched — no stacks of any kind.
    let bolt = spawn_bolt(&mut app);
    let cell = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell, Some(bolt));
    write_bump(&mut app, Some(bolt));

    run_fixed_update(&mut app);

    assert!(app.world().get::<OverchargeKillCount>(bolt).is_none());
    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none()
    );
}

#[test]
fn empty_active_hazards_never_inserts_across_five_ticks() {
    // Edge: five consecutive empty-gate ticks still insert nothing.
    let mut app = test_app_playing();
    wire(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    let bolt = spawn_bolt(&mut app);

    for _ in 0..5 {
        run_fixed_update(&mut app);
    }

    assert!(app.world().get::<OverchargeKillCount>(bolt).is_none());
    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none()
    );
}

// ── Behavior 44 — stack-0 PRESERVE: stale Overcharge entry untouched ────
//
// REGRESSION PIN: with 0 Overcharge stacks, `hazard_active` is false and
// the chain never runs. A pre-existing `"hazard:overcharge"` entry on a
// bolt's stack persists indefinitely. Flagged as a design concern in the
// test spec (mirrors the Haste stack-0 persistence pin).

#[test]
fn stack_zero_preserves_pre_existing_overcharge_entry() {
    let mut app = test_app_playing();
    wire(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    // Zero Overcharge stacks.
    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        hazard_overcharge(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.40),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, 0, seed);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("seeded stack persists");
    assert_eq!(stack.len(), 1);
    assert!(
        (stack.aggregate() - 1.40).abs() < 1e-6,
        "stack-0 with gate off leaves the stale Overcharge entry untouched"
    );
}

#[test]
fn stack_zero_preserves_chip_and_pre_existing_overcharge_entries() {
    // Edge: chip entry + stale Overcharge entry both persist.
    let mut app = test_app_playing();
    wire(&mut app);
    install_overcharge_config(&mut app, canonical_config());

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        hazard_overcharge(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.40),
        },
    );
    seed.push(
        chip_overclock(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, 0, seed);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 2);
    // 1.40 * 1.5 = 2.10
    assert!((stack.aggregate() - 2.10).abs() < 1e-5);
}
