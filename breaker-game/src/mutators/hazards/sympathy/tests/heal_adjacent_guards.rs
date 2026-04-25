//! Group C — `sympathy_heal_adjacent` no-op guards (Behaviors 26–33).
//!
//! Pins that the heal-emit system is inert when the config, stacks, or damage
//! input preconditions are not met. Run-if gates (`hazard_active(Sympathy)` +
//! `in_state(NodeState::Playing)`) are covered via `wire`.

use bevy::prelude::*;

use super::{
    super::system::{sympathy_heal_adjacent, wire},
    helpers::{
        add_hazard_stacks, add_sympathy_stacks, canonical_sympathy_config, heal_collector_len,
        install_sympathy_config, run_fixed_update, spawn_cell_at_default, test_app_not_playing,
        test_app_playing, write_cell_damage,
    },
};
use crate::mutators::hazards::{definition::HazardKind, resources::ActiveHazards};

// ── Behavior 26 — no heal when SympathyConfig absent ────────────────────────

#[test]
fn no_heal_when_sympathy_config_absent() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    add_sympathy_stacks(&mut app, 1);
    // NO SympathyConfig installed.

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let _b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    assert_eq!(
        heal_collector_len(&app),
        0,
        "no HealDealt<Cell> must be emitted when SympathyConfig is absent"
    );
}

// ── Behavior 27 — no heal when zero Sympathy stacks ─────────────────────────

#[test]
fn no_heal_when_zero_sympathy_stacks() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    // 0 stacks — heal_percent(0) == 0.0 guard must fire.

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let _b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 0);
}

// ── Behavior 28 — no heal when no DamageDealt<Cell> messages ────────────────

#[test]
fn no_heal_when_no_damage_messages() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 1);

    let _a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let _b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    // No damage message written.

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 0);
}

// ── Behavior 29 — hazard_active gate blocks when a different hazard stacked ─

#[test]
fn hazard_active_gate_blocks_when_different_hazard_stacked() {
    let mut app = test_app_playing();
    wire(&mut app);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_hazard_stacks(&mut app, HazardKind::Volatility, 1);
    assert_eq!(
        app.world()
            .resource::<ActiveHazards>()
            .stacks(HazardKind::Sympathy),
        0,
        "sanity: zero Sympathy stacks"
    );

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let _b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    assert_eq!(
        heal_collector_len(&app),
        0,
        "hazard_active(Sympathy) run-if gate must block"
    );
}

// ── Behavior 30 — in_state(NodeState::Playing) gate blocks when not Playing ─

#[test]
fn in_state_playing_gate_blocks_when_not_playing() {
    let mut app = test_app_not_playing();
    wire(&mut app);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 1);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let _b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    assert_eq!(
        heal_collector_len(&app),
        0,
        "in_state(NodeState::Playing) run-if gate must block"
    );
}

// ── Behavior 31 — zero-amount DamageDealt<Cell> emits no heal ───────────────

#[test]
fn zero_amount_damage_emits_no_heal() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 1);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let _b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    write_cell_damage(&mut app, a, 0.0);

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 0);
}

// ── Behavior 32 — negative-amount DamageDealt<Cell> emits no heal ───────────

#[test]
fn negative_amount_damage_emits_no_heal() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 1);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let _b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    write_cell_damage(&mut app, a, -5.0);

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 0);
}

// ── Behavior 33 — damage targeting a despawned entity does not panic ────────

#[test]
fn despawned_target_does_not_panic() {
    // Spec-level guarantee: no panic / no unwrap on a despawned entity.
    // Heal count is intentionally NOT asserted — some implementations may
    // still emit a heal at a cached position, others may short-circuit.
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 1);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let _b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    write_cell_damage(&mut app, a, 100.0);
    app.world_mut().despawn(a);

    run_fixed_update(&mut app);
    // (no panic) — assert nothing on heal count.
}
