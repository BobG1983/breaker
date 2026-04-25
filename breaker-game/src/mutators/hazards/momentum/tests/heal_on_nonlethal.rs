//! Groups C + D — `momentum_heal_on_nonlethal` no-op guards and positive
//! message shape (Behaviors 11–27).
//!
//! Pins that damage messages against live cells produce `HealDealt<Cell>` with
//! the stack-scaled amount, `HealCap::Max`, `healer: None`, and
//! `source: Some("hazard:momentum".into())`. Guards: missing config, zero
//! stacks, no damage messages, run-if gated on `hazard_active(Momentum)` AND
//! `in_state(NodeState::Playing)`, lethal hits → no heal,
//! `Dead`/`Invulnerable` → no heal, zero-amount damage → no heal.

use bevy::prelude::*;
use rantzsoft_dmg::{RantzDmgAppExt, RantzDmgPlugin};

use super::{
    super::system::{momentum_heal_on_nonlethal, register},
    helpers::{
        add_hazard_stacks, add_momentum_stacks, canonical_momentum_config, heal_collector_len,
        heals_for_cell, install_momentum_config, run_fixed_update, spawn_cell_at,
        spawn_cell_dead_at, spawn_cell_invulnerable_at, test_app_not_playing, test_app_playing,
        write_cell_damage,
    },
};
use crate::{
    cells::components::Cell,
    mutators::hazards::{definition::HazardKind, resources::ActiveHazards},
    prelude::*,
};

// ════════════════════════════════════════════════════════════════════════════
// Group C — No-op guards
// ════════════════════════════════════════════════════════════════════════════

// ── Behavior 11 — no message when MomentumConfig absent ─────────────────────

#[test]
fn no_heal_when_momentum_config_absent() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, momentum_heal_on_nonlethal);
    add_momentum_stacks(&mut app, 1);
    // NO MomentumConfig installed.

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 5.0, 10.0);
    write_cell_damage(&mut app, cell, 0.0);

    run_fixed_update(&mut app);

    assert_eq!(
        heal_collector_len(&app),
        0,
        "no HealDealt<Cell> when MomentumConfig is absent"
    );
    // Hp must not be mutated directly by the heal system.
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "cell Hp.current must remain 5.0 (heal system must never write Hp); got {}",
        hp.current
    );
}

// ── Behavior 12 — no message when stacks == 0 ───────────────────────────────

#[test]
fn no_heal_when_zero_momentum_stacks() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, momentum_heal_on_nonlethal);
    install_momentum_config(&mut app, canonical_momentum_config());
    // 0 stacks.

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 5.0, 10.0);
    write_cell_damage(&mut app, cell, 5.0);

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 0);
}

// ── Behavior 13 — no message when no DamageDealt<Cell> present ──────────────

#[test]
fn no_heal_when_no_damage_messages() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, momentum_heal_on_nonlethal);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let _cell = spawn_cell_at(&mut app, Vec2::ZERO, 5.0, 10.0);
    // No damage message written.

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 0);
}

// ── Behavior 14 — hazard_active run-if gate blocks (different hazard) ───────

#[test]
fn gate_blocks_when_different_hazard_stacked() {
    let mut app = test_app_playing();
    register(&mut app);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_hazard_stacks(&mut app, HazardKind::Volatility, 1);
    assert_eq!(
        app.world()
            .resource::<ActiveHazards>()
            .stacks(HazardKind::Momentum),
        0
    );

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 5.0, 10.0);
    write_cell_damage(&mut app, cell, 2.0);

    run_fixed_update(&mut app);

    assert_eq!(
        heal_collector_len(&app),
        0,
        "hazard_active(Momentum) run-if gate must block the system"
    );
}

// ── Behavior 15 — in_state(Playing) run-if gate blocks ──────────────────────

#[test]
fn gate_blocks_when_not_in_playing_state() {
    let mut app = test_app_not_playing();
    register(&mut app);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 5.0, 10.0);
    write_cell_damage(&mut app, cell, 2.0);

    run_fixed_update(&mut app);

    assert_eq!(
        heal_collector_len(&app),
        0,
        "in_state(NodeState::Playing) gate must block the system"
    );
}

// ── Behavior 16 — zero-amount damage message produces no heal ───────────────

#[test]
fn zero_damage_amount_emits_no_heal() {
    let mut app = test_app_playing();
    app.add_plugins(RantzDmgPlugin);
    let _ = app.register_dmgable::<Cell>();
    // Wire apply_damage first so the damage lands (at zero). Then Momentum observes.
    app.add_systems(
        FixedUpdate,
        momentum_heal_on_nonlethal.after(DmgSystems::ApplyDamage),
    );
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);
    write_cell_damage(&mut app, cell, 0.0);

    run_fixed_update(&mut app);

    assert_eq!(
        heal_collector_len(&app),
        0,
        "zero-amount damage is a no-op for Momentum's non-lethal trigger"
    );
}

// ── Behavior 17 — despawned target does not panic ───────────────────────────

#[test]
fn damage_against_despawned_target_does_not_panic() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, momentum_heal_on_nonlethal);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 5.0, 10.0);
    // Write damage targeting the cell, then despawn it pre-tick.
    write_cell_damage(&mut app, cell, 2.0);
    app.world_mut().despawn(cell);

    run_fixed_update(&mut app);

    // No panic. No heal emitted for a stale target either.
    assert_eq!(heal_collector_len(&app), 0);
}

// ════════════════════════════════════════════════════════════════════════════
// Group D — Positive message shape
// ════════════════════════════════════════════════════════════════════════════

/// Builder for Group D: wires `apply_damage` → `momentum_heal_on_nonlethal` in
/// order so that heal observes post-damage `hp.current`.
fn test_app_with_damage_and_heal_emit() -> App {
    let mut app = test_app_playing();
    app.add_plugins(RantzDmgPlugin);
    let _ = app.register_dmgable::<Cell>();
    app.add_systems(
        FixedUpdate,
        momentum_heal_on_nonlethal.after(DmgSystems::ApplyDamage),
    );
    app
}

// ── Behavior 18 — non-lethal at stack 1 emits HealDealt with correct fields ─

#[test]
fn nonlethal_hit_stack_one_emits_one_heal_with_correct_fields() {
    let mut app = test_app_with_damage_and_heal_emit();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    // Pre-seed hp.max so apply_heal wouldn't clamp at starting — but we're not
    // wiring apply_heal here. This test only asserts the emit shape.
    let cell = super::helpers::spawn_cell_at_with_max(&mut app, Vec2::ZERO, 10.0, 10.0, Some(20.0));

    write_cell_damage(&mut app, cell, 5.0);

    run_fixed_update(&mut app);

    let msgs = heals_for_cell(&app, cell);
    assert_eq!(
        msgs.len(),
        1,
        "expected exactly 1 HealDealt<Cell>, got {}",
        msgs.len()
    );
    let msg = &msgs[0];
    assert_eq!(msg.target, cell);
    assert!(
        (msg.amount - 10.0).abs() < f32::EPSILON,
        "amount must equal heal_per_hit(1) == 10.0, got {}",
        msg.amount
    );
    assert!(matches!(msg.cap, HealCap::Max));
    assert_eq!(msg.healer, None);
    assert_eq!(
        msg.source.as_ref(),
        Some(&SourceId::hazard(HazardKind::Momentum).build())
    );
}

// ── Behavior 19 — stack 3 scales amount to 30.0 ─────────────────────────────

#[test]
fn nonlethal_hit_stack_three_scales_heal_to_thirty() {
    let mut app = test_app_with_damage_and_heal_emit();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 3);

    let cell =
        super::helpers::spawn_cell_at_with_max(&mut app, Vec2::ZERO, 50.0, 50.0, Some(100.0));
    write_cell_damage(&mut app, cell, 10.0);

    run_fixed_update(&mut app);

    let msgs = heals_for_cell(&app, cell);
    assert_eq!(msgs.len(), 1);
    assert!(
        (msgs[0].amount - 30.0).abs() < f32::EPSILON,
        "stack-3 heal must be 30.0, got {}",
        msgs[0].amount
    );
    assert!(matches!(msgs[0].cap, HealCap::Max));
    assert_eq!(
        msgs[0].source.as_ref(),
        Some(&SourceId::hazard(HazardKind::Momentum).build())
    );
}

// ── Behavior 20 — lethal hit emits no heal (post-hp <= 0) ───────────────────

#[test]
fn lethal_hit_emits_no_heal() {
    let mut app = test_app_with_damage_and_heal_emit();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);
    // 15.0 damage on 10.0 HP → post-damage hp.current == -5.0 (lethal).
    write_cell_damage(&mut app, cell, 15.0);

    run_fixed_update(&mut app);

    assert_eq!(
        heal_collector_len(&app),
        0,
        "lethal hit must not emit a heal message"
    );
}

#[test]
fn exactly_lethal_hit_emits_no_heal() {
    let mut app = test_app_with_damage_and_heal_emit();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);
    // Exactly-lethal: post-damage hp.current == 0.0. Boundary is strict `> 0.0`.
    write_cell_damage(&mut app, cell, 10.0);

    run_fixed_update(&mut app);

    assert_eq!(
        heal_collector_len(&app),
        0,
        "exactly-lethal hit (post-hp == 0.0) must not emit a heal"
    );
}

// ── Behavior 21 — already-Dead cell gets no heal ────────────────────────────

#[test]
fn dead_marked_cell_gets_no_heal_from_stale_damage() {
    let mut app = test_app_with_damage_and_heal_emit();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_dead_at(&mut app, Vec2::ZERO, 5.0, 10.0);
    write_cell_damage(&mut app, cell, 2.0);

    run_fixed_update(&mut app);

    assert_eq!(
        heal_collector_len(&app),
        0,
        "Dead-marked cell must not receive a heal"
    );
}

// ── Behavior 22 — Invulnerable cell gets no heal ────────────────────────────

#[test]
fn invulnerable_cell_gets_no_heal() {
    let mut app = test_app_with_damage_and_heal_emit();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_invulnerable_at(&mut app, Vec2::ZERO, 10.0, 10.0);
    write_cell_damage(&mut app, cell, 2.0);

    run_fixed_update(&mut app);

    assert_eq!(
        heal_collector_len(&app),
        0,
        "Invulnerable cell must not receive a heal"
    );
}

// ── Behavior 23 — multiple damage messages → one heal each ──────────────────

#[test]
fn multiple_damage_messages_produce_one_heal_each() {
    let mut app = test_app_with_damage_and_heal_emit();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let a = super::helpers::spawn_cell_at_with_max(
        &mut app,
        Vec2::new(100.0, 0.0),
        10.0,
        10.0,
        Some(20.0),
    );
    let b = super::helpers::spawn_cell_at_with_max(
        &mut app,
        Vec2::new(-100.0, 0.0),
        10.0,
        10.0,
        Some(20.0),
    );
    let c = super::helpers::spawn_cell_at_with_max(
        &mut app,
        Vec2::new(0.0, 100.0),
        10.0,
        10.0,
        Some(20.0),
    );
    write_cell_damage(&mut app, a, 5.0);
    write_cell_damage(&mut app, b, 5.0);
    write_cell_damage(&mut app, c, 5.0);

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 3);
    for entity in [a, b, c] {
        let msgs = heals_for_cell(&app, entity);
        assert_eq!(msgs.len(), 1, "each cell should receive exactly one heal");
        assert!((msgs[0].amount - 10.0).abs() < f32::EPSILON);
        assert!(matches!(msgs[0].cap, HealCap::Max));
    }
}

// ── Behavior 24 — every message carries cap == HealCap::Max ─────────────────

#[test]
fn every_emitted_message_has_cap_max() {
    let mut app = test_app_with_damage_and_heal_emit();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let a = super::helpers::spawn_cell_at_with_max(
        &mut app,
        Vec2::new(100.0, 0.0),
        10.0,
        10.0,
        Some(20.0),
    );
    let b = super::helpers::spawn_cell_at_with_max(
        &mut app,
        Vec2::new(-100.0, 0.0),
        10.0,
        10.0,
        Some(20.0),
    );
    write_cell_damage(&mut app, a, 5.0);
    write_cell_damage(&mut app, b, 5.0);

    run_fixed_update(&mut app);

    let all = &app
        .world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0;
    assert_eq!(all.len(), 2);
    assert!(
        all.iter().all(|m| matches!(m.cap, HealCap::Max)),
        "every message must carry HealCap::Max"
    );
}

// ── Behavior 25 — every message carries source == Some("hazard:momentum") ───

#[test]
fn every_emitted_message_has_momentum_sentinel_source() {
    let mut app = test_app_with_damage_and_heal_emit();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let a = super::helpers::spawn_cell_at_with_max(
        &mut app,
        Vec2::new(100.0, 0.0),
        10.0,
        10.0,
        Some(20.0),
    );
    let b = super::helpers::spawn_cell_at_with_max(
        &mut app,
        Vec2::new(-100.0, 0.0),
        10.0,
        10.0,
        Some(20.0),
    );
    write_cell_damage(&mut app, a, 5.0);
    write_cell_damage(&mut app, b, 5.0);

    run_fixed_update(&mut app);

    let all = &app
        .world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0;
    assert_eq!(all.len(), 2);
    assert!(
        all.iter()
            .all(|m| m.source == Some(SourceId::hazard(HazardKind::Momentum).build())),
        "every message's source must be Some(\"hazard:momentum\")"
    );
}

// ── Behavior 26 — every message carries healer == None ──────────────────────

#[test]
fn every_emitted_message_has_healer_none() {
    let mut app = test_app_with_damage_and_heal_emit();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let a = super::helpers::spawn_cell_at_with_max(
        &mut app,
        Vec2::new(100.0, 0.0),
        10.0,
        10.0,
        Some(20.0),
    );
    let b = super::helpers::spawn_cell_at_with_max(
        &mut app,
        Vec2::new(-100.0, 0.0),
        10.0,
        10.0,
        Some(20.0),
    );
    write_cell_damage(&mut app, a, 5.0);
    write_cell_damage(&mut app, b, 5.0);

    run_fixed_update(&mut app);

    let all = &app
        .world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0;
    assert_eq!(all.len(), 2);
    assert!(all.iter().all(|m| m.healer.is_none()));
}

// ── Behavior 27 — regression: momentum_heal_on_nonlethal does not mutate Hp ─

#[test]
fn regression_heal_system_does_not_mutate_hp_directly() {
    let mut app = test_app_playing();
    // ONLY momentum_heal_on_nonlethal wired (no apply_damage, no apply_heal).
    // The cell is pre-seeded at the post-damage state: hp.current = 5.0,
    // starting = 10.0. The damage message is stale (damage-apply is absent).
    // momentum_heal_on_nonlethal should observe hp.current > 0.0 and emit a
    // heal — but must NEVER mutate Hp directly.
    app.add_systems(FixedUpdate, momentum_heal_on_nonlethal);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 5.0, 10.0);
    write_cell_damage(&mut app, cell, 5.0);

    run_fixed_update(&mut app);

    // (a) Hp.current must be untouched by momentum_heal_on_nonlethal.
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Hp.current must remain 5.0 — heal system must NEVER mutate Hp; got {}",
        hp.current
    );

    // (b) The heal message WAS emitted (one per non-lethal damage message).
    assert_eq!(
        heal_collector_len(&app),
        1,
        "exactly 1 HealDealt<Cell> must be emitted for the non-lethal damage; got {}",
        heal_collector_len(&app)
    );
}

// ── B32: source matches builder-produced hazard:momentum ──

#[test]
fn momentum_heal_source_equals_builder_hazard_momentum() {
    use crate::{mutators::hazards::definition::HazardKind, prelude::SourceIdExt};
    let mut app = test_app_with_damage_and_heal_emit();
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell =
        super::helpers::spawn_cell_at_with_max(&mut app, Vec2::ZERO, 50.0, 50.0, Some(100.0));
    write_cell_damage(&mut app, cell, 10.0);
    run_fixed_update(&mut app);

    let msgs = heals_for_cell(&app, cell);
    assert!(!msgs.is_empty());
    let expected = SourceId::hazard(HazardKind::Momentum).build();
    for m in &msgs {
        assert_eq!(m.source.as_ref(), Some(&expected));
    }
}
