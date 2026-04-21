use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::shared::death_pipeline::{HealCap, Hp};

// ════════════════════════════════════════════════════════════════════════════
// Group H — Retrofit-specific regression guards
// ════════════════════════════════════════════════════════════════════════════

// Behavior 30: regression — cascade_heal_on_death does NOT mutate Hp directly.
#[test]
fn regression_cascade_does_not_mutate_hp_directly() {
    let mut app = test_app_playing();
    // ONLY cascade_heal_on_death wired — apply_heal::<Cell> is deliberately
    // absent. Any Hp change observed here is a direct mutation.
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    // 1. Message WAS emitted.
    assert_eq!(
        cascade_heal_collector_len(&app),
        1,
        "cascade must emit exactly one HealDealt<Cell> message"
    );
    // 2. Hp was NOT directly mutated — apply_heal isn't wired here.
    let hp = app.world().get::<Hp>(neighbour).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "neighbour Hp must remain 5.0 (apply_heal isn't wired); got {}. \
         A change here proves cascade_heal_on_death is directly mutating Hp.",
        hp.current
    );
}

// Behavior 32: regression — no pre-clamp in cascade. amount is raw
// heal_per_neighbour(stacks), not min'd against hp.max or hp.starting.
#[test]
fn regression_cascade_emits_raw_amount_no_pre_clamp() {
    let mut app = test_app_playing();
    // ONLY cascade wired.
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      100.0,
            per_level_heal: 0.0,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    // Neighbour has starting = 10.0, max = Some(20.0). A buggy implementation
    // that pre-clamps at min(max, starting) or similar would emit amount = 15.0
    // or 5.0. We expect the raw 100.0 — the cap is declarative.
    let neighbour = spawn_cell_at_with_max(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0, Some(20.0));
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let msgs = heals_for_cell(&app, neighbour);
    assert_eq!(msgs.len(), 1);
    assert!(
        (msgs[0].amount - 100.0).abs() < f32::EPSILON,
        "amount must be the raw heal (100.0), not pre-clamped against Hp \
         ceilings; got {}. Pre-clamp is apply_heal's job via HealCap.",
        msgs[0].amount
    );
    assert!(matches!(msgs[0].cap, HealCap::Starting));
}
