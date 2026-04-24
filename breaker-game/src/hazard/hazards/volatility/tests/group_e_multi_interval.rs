use std::time::Duration;

use super::{super::system::*, helpers::*};
use crate::prelude::HealCap;

// ══════════════════════════════════════════════════════════════════════
// Group E — multi-interval accumulation (pause/resume)
// ══════════════════════════════════════════════════════════════════════

// Behavior 21 — 12.5s tick emits two heals, elapsed ≈ 2.5
#[test]
fn tick_12_5s_emits_two_heals_and_remainder() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(12.5));

    let heals = heals_for(&app, cell);
    assert_eq!(heals.len(), 2);
    for msg in &heals {
        assert!((msg.amount - 1.0).abs() < f32::EPSILON);
        assert_eq!(msg.cap, HealCap::Max);
    }
    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    assert!(
        (timer.elapsed - 2.5).abs() < 1e-5,
        "timer.elapsed should be ~2.5, got {}",
        timer.elapsed
    );
}

// Behavior 21 edge — exactly three intervals
#[test]
fn tick_15_0s_emits_three_heals_and_rolls_over() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(15.0));

    let heals = heals_for(&app, cell);
    assert_eq!(heals.len(), 3);
    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    assert!(timer.elapsed.abs() < 1e-5);
}

// Behavior 21 edge — 5.0 + EPSILON emits exactly one
#[test]
fn tick_5_0_plus_epsilon_emits_one() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(5.0 + f32::EPSILON));

    let heals = heals_for(&app, cell);
    assert_eq!(heals.len(), 1, "5.0 + EPSILON should emit exactly one heal");
}

// Behavior 21 edge — 50.0s emits ten heals
#[test]
fn tick_50s_emits_ten_heals() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(50.0));

    let heals = heals_for(&app, cell);
    assert_eq!(heals.len(), 10);
    for msg in &heals {
        assert!((msg.amount - 1.0).abs() < f32::EPSILON);
        assert_eq!(msg.cap, HealCap::Max);
    }
}

// Behavior 22 — per-iteration cap check from live Hp; two emissions at current=19.0
#[test]
fn per_iteration_cap_check_reads_live_hp_current_19() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 19.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(10.0));

    let heals = heals_for(&app, cell);
    assert_eq!(heals.len(), 2);
    for msg in &heals {
        assert!((msg.amount - 1.0).abs() < f32::EPSILON);
        assert_eq!(msg.cap, HealCap::Max);
    }
}

// Behavior 22 edge — at cap, two intervals emit zero
#[test]
fn per_iteration_at_cap_two_intervals_emit_zero() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer_max(&mut app, 20.0, 10.0, Some(20.0), 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(10.0));

    assert_eq!(heal_collector_len(&app), 0);
    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    assert!(timer.elapsed.abs() < 1e-5);
}

// Behavior 22 edge — base case at current=19.5
#[test]
fn per_iteration_current_19_5_two_emissions() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 19.5, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(10.0));

    let heals = heals_for(&app, cell);
    assert_eq!(heals.len(), 2);
}
