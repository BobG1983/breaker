use std::time::Duration;

use super::{super::system::*, helpers::*};
use crate::{prelude::*, shared::death_pipeline::HealCap};

// ══════════════════════════════════════════════════════════════════════
// Group B — stacking and interval floor
// ══════════════════════════════════════════════════════════════════════

// Behavior 6 — stack=3, interval ≈ 3.333s
#[test]
fn stack_3_emits_heal_after_effective_interval() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 3);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(3.3334));

    let heals = heals_for(&app, cell);
    assert_eq!(
        heals.len(),
        1,
        "expected one HealDealt<Cell> at stack=3 after 3.3334s"
    );
    assert!((heals[0].amount - 1.0).abs() < f32::EPSILON);
    assert_eq!(heals[0].cap, HealCap::Max);
    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    assert!(
        timer.elapsed.abs() < 1e-3,
        "timer.elapsed should roll over to ~0.0, got {}",
        timer.elapsed
    );
}

// Behavior 6 edge — just below interval at stack 3
#[test]
fn stack_3_just_below_interval_emits_zero() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 3);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(3.333));

    assert_eq!(heal_collector_len(&app), 0);
    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    assert!(
        (timer.elapsed - 3.333).abs() < 1e-3,
        "timer.elapsed should be ~3.333, got {}",
        timer.elapsed
    );
}

// Behavior 6 edge — two intervals exactly at stack 3
#[test]
fn stack_3_two_intervals_emits_two_heals() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 3);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(6.667));

    let heals = heals_for(&app, cell);
    assert_eq!(
        heals.len(),
        2,
        "two intervals at stack=3 should emit two heals"
    );
    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    assert!(
        timer.elapsed.abs() < 1e-3,
        "timer.elapsed should be ~0.0, got {}",
        timer.elapsed
    );
}

// Behavior 7 — stack=2, interval = 4.0s
#[test]
fn stack_2_emits_heal_after_4_seconds() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 2);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(4.0));

    let heals = heals_for(&app, cell);
    assert_eq!(heals.len(), 1);
    assert!((heals[0].amount - 1.0).abs() < f32::EPSILON);
    assert_eq!(heals[0].cap, HealCap::Max);
}

// Behavior 7 edge — just below 4.0s
#[test]
fn stack_2_just_below_interval_emits_zero() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 2);
    register_volatility_systems(&mut app);
    spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(3.999));

    assert_eq!(heal_collector_len(&app), 0);
}

// Behavior 7 edge — just above 4.0s
#[test]
fn stack_2_just_above_interval_emits_one() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 2);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(4.001));

    let heals = heals_for(&app, cell);
    assert_eq!(heals.len(), 1);
}

// Behavior 8 — stack=100 floored at 1.0s, 0.999s emits zero
#[test]
fn stack_100_floor_0_999s_emits_zero() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 100);
    register_volatility_systems(&mut app);
    spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.999));

    assert_eq!(heal_collector_len(&app), 0);
}

// Behavior 8 edge — exactly 1.0s emits one heal
#[test]
fn stack_100_floor_1_0s_emits_one() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 100);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let heals = heals_for(&app, cell);
    assert_eq!(heals.len(), 1);
    assert!((heals[0].amount - 1.0).abs() < f32::EPSILON);
    assert_eq!(heals[0].cap, HealCap::Max);
}

// Behavior 8 edge — 0.5s tick still no heal
#[test]
fn stack_100_half_second_no_heal() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 100);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.5));

    assert_eq!(heal_collector_len(&app), 0);
    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    assert!(
        (timer.elapsed - 0.5).abs() < 1e-3,
        "timer.elapsed should be ~0.5, got {}",
        timer.elapsed
    );
}

// Behavior 8 edge — stack=1000 also floors at 1.0s
#[test]
fn stack_1000_floor_is_exactly_1_0s() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1000);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let heals = heals_for(&app, cell);
    assert_eq!(heals.len(), 1, "stack=1000 should still use the 1.0s floor");
}

// Behavior 9 — stack=0 (hazard inactive)
#[test]
fn stack_0_inactive_hazard_no_timer_no_heal_no_max_change() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    // ActiveHazards::default() — no Volatility stacks.
    register_volatility_systems(&mut app);
    let cell = spawn_cell_no_timer(&mut app, 5.0, 5.0, None);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    assert!(
        app.world().get::<VolatilityTimer>(cell).is_none(),
        "no VolatilityTimer should be attached when hazard inactive"
    );
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!((hp.current - 5.0).abs() < f32::EPSILON);
    assert!(
        hp.max.is_none(),
        "Hp.max should remain None when hazard inactive"
    );
    assert_eq!(heal_collector_len(&app), 0);
}
