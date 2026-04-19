//! Group F — `register` wiring, ordering, and run-if gates (Behaviors 53–60).
//!
//! Pins that `register(&mut app)` places `sympathy_heal_adjacent` in
//! `FixedUpdate` inside `DeathPipelineSystems::ApplyHeal`, BEFORE
//! `apply_heal::<Cell>`, with both `hazard_active(Sympathy)` and
//! `in_state(NodeState::Playing)` run-if gates. Pins that the emitted
//! `HealDealt<Cell>` reaches `apply_heal::<Cell>` and lands on the target
//! within the same tick.

use bevy::prelude::*;

use super::{
    super::system::register,
    helpers::{
        add_hazard_stacks, add_sympathy_stacks, all_heals, canonical_sympathy_config,
        heal_collector_len, heals_for_cell, install_sympathy_config, run_fixed_update,
        spawn_cell_at, spawn_cell_at_default, test_app_not_playing, test_app_playing,
        write_cell_damage,
    },
};
use crate::{
    hazard::{definition::HazardKind, hazards::momentum, resources::ActiveHazards},
    prelude::*,
    shared::death_pipeline::{HealCap, sets::DeathPipelineSystems, systems::apply_heal},
};

/// Sets up death-pipeline set ordering + `apply_heal::<Cell>`, then calls
/// `register(&mut app)` so the full Sympathy plumbing is in place.
fn register_app_with_apply_heal() -> App {
    let mut app = test_app_playing();
    app.configure_sets(
        FixedUpdate,
        (
            DeathPipelineSystems::ApplyDamage,
            DeathPipelineSystems::DetectDeaths.after(DeathPipelineSystems::ApplyDamage),
            DeathPipelineSystems::HandleKill.after(DeathPipelineSystems::DetectDeaths),
            DeathPipelineSystems::ApplyHeal.after(DeathPipelineSystems::HandleKill),
        ),
    );
    app.add_systems(
        FixedUpdate,
        apply_heal::<Cell>.in_set(DeathPipelineSystems::ApplyHeal),
    );
    register(&mut app);
    app
}

// ── Behavior 53 — register wires sympathy_heal_adjacent in FixedUpdate ──────

#[test]
fn register_wires_sympathy_heal_adjacent_into_fixed_update() {
    let mut app = register_app_with_apply_heal();
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 1);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    // B at (50,0) with Hp::new(50.0) — already at starting; HealCap::Starting clamps.
    let b = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 50.0, 50.0);
    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    assert_eq!(
        heal_collector_len(&app),
        1,
        "register must wire sympathy_heal_adjacent; expected 1 heal, got {}",
        heal_collector_len(&app)
    );

    let msgs = heals_for_cell(&app, b);
    assert_eq!(msgs.len(), 1);
    assert!(
        (msgs[0].amount - 25.0).abs() < 1e-4,
        "heal amount must be 25.0, got {}",
        msgs[0].amount
    );
    assert!(matches!(msgs[0].cap, HealCap::Starting));
    assert_eq!(msgs[0].source.as_deref(), Some("hazard:sympathy"));
}

// ── Behavior 54 — emitted heal reaches apply_heal::<Cell> and lands same tick

#[test]
fn register_emitted_heal_reaches_apply_heal_and_restores_hp_same_tick() {
    let mut app = register_app_with_apply_heal();
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 1);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    // B at (50,0) with Hp { current=20, starting=50 }. Heal 25 → current becomes 45.
    let b = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 20.0, 50.0);
    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    let hp_b = app.world().get::<Hp>(b).expect("B must still be alive");
    assert!(
        (hp_b.current - 45.0).abs() < 1e-4,
        "B.Hp.current must become 45.0 (20 + 25 heal) same tick; got {}",
        hp_b.current
    );
}

// ── Behavior 55 — ordering places sympathy_heal_adjacent BEFORE apply_heal ──

#[test]
fn register_orders_sympathy_heal_adjacent_before_apply_heal_starting_clamp() {
    let mut app = register_app_with_apply_heal();
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 1);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    // B at (50,0) with Hp { current=20, starting=30 }. Heal 25 → clamped to 30.
    let b = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 20.0, 30.0);
    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    let hp_b = app.world().get::<Hp>(b).expect("B must still be alive");
    assert!(
        (hp_b.current - 30.0).abs() < 1e-4,
        "B.Hp.current must be clamped to starting 30.0 same tick; got {}",
        hp_b.current
    );
}

// ── Behavior 56 — register's in_state(Playing) gate blocks when not Playing ─

#[test]
fn register_in_state_playing_gate_blocks_when_not_playing() {
    let mut app = test_app_not_playing();
    app.configure_sets(
        FixedUpdate,
        (
            DeathPipelineSystems::ApplyDamage,
            DeathPipelineSystems::DetectDeaths.after(DeathPipelineSystems::ApplyDamage),
            DeathPipelineSystems::HandleKill.after(DeathPipelineSystems::DetectDeaths),
            DeathPipelineSystems::ApplyHeal.after(DeathPipelineSystems::HandleKill),
        ),
    );
    app.add_systems(
        FixedUpdate,
        apply_heal::<Cell>.in_set(DeathPipelineSystems::ApplyHeal),
    );
    register(&mut app);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 1);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let _b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 0);
}

// ── Behavior 57 — register's hazard_active gate blocks under different hazard

#[test]
fn register_hazard_active_gate_blocks_under_different_hazard() {
    let mut app = register_app_with_apply_heal();
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_hazard_stacks(&mut app, HazardKind::Volatility, 1);
    assert_eq!(
        app.world()
            .resource::<ActiveHazards>()
            .stacks(HazardKind::Sympathy),
        0
    );

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let _b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 0);
}

// ── Behavior 58 — register does not panic when SympathyConfig is absent ─────

#[test]
fn register_does_not_panic_when_sympathy_config_absent() {
    let mut app = register_app_with_apply_heal();
    // NO SympathyConfig.
    add_sympathy_stacks(&mut app, 1);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let _b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);
    // No panic. No heal emitted either.
    assert_eq!(heal_collector_len(&app), 0);
}

// ── Behavior 59 — plugin_builds — Sympathy schedules tick with no damage ────

#[test]
fn plugin_builds_sympathy_schedules_tick_with_no_messages() {
    let mut app = register_app_with_apply_heal();
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 1);

    // No cells, no damage messages — just prove the schedule builds and ticks.
    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 0);
}

// ── Behavior 60 — schedule does not panic with both Sympathy and Momentum ───

#[test]
fn schedule_does_not_panic_with_both_sympathy_and_momentum_stacked() {
    let mut app = register_app_with_apply_heal();
    // Register Momentum too — both systems read DamageDealt<Cell> + write HealDealt<Cell>.
    momentum::register(&mut app);

    // Both configs + 1 stack each.
    install_sympathy_config(&mut app, canonical_sympathy_config());
    app.world_mut()
        .insert_resource(momentum::system::MomentumConfig {
            base_hp_per_hit:      10.0,
            per_level_hp_per_hit: 10.0,
        });
    add_sympathy_stacks(&mut app, 1);
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Momentum);

    // Single cell A at (0,0) with Hp::new(50.0); one damage of 10.0 — non-lethal.
    let a = spawn_cell_at(&mut app, Vec2::ZERO, 50.0, 50.0);
    write_cell_damage(&mut app, a, 10.0);

    run_fixed_update(&mut app);

    // Momentum emits 1 heal (self-heal to A). Sympathy emits 0 (A has no neighbours).
    let all = all_heals(&app);
    assert_eq!(
        all.len(),
        1,
        "with both hazards active but A alone, only Momentum's self-heal \
         should appear; got {}",
        all.len()
    );
    assert_eq!(all[0].source.as_deref(), Some("hazard:momentum"));
}
