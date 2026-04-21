use std::time::Duration;

use super::{super::system::*, helpers::*};
use crate::prelude::*;

// ══════════════════════════════════════════════════════════════════════
// Group D — attach_volatility_timers
// ══════════════════════════════════════════════════════════════════════

// Behavior 15 — attach inserts VolatilityTimer when hazard active
#[test]
fn attach_inserts_timer_on_cells_when_hazard_active() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_no_timer(&mut app, 10.0, 10.0, None);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    // After an attach in the same tick, grow advances elapsed by dt.
    // Tolerate up to one fixed-update tick's worth of accumulation.
    assert!(
        timer.elapsed < 0.02,
        "timer.elapsed should be near 0.0 after attach (≤ one tick's dt), got {}",
        timer.elapsed
    );
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max,
        Some(20.0),
        "Hp.max should be lifted to Some(20.0), got {:?}",
        hp.max
    );
}

// Behavior 15 edge — idempotent: pre-existing timer not overwritten,
// but Hp.max is still lifted.
#[test]
fn attach_idempotent_preserves_existing_timer_but_lifts_hp_max() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    // Cell with pre-existing timer at elapsed=2.0 and Hp.max=None.
    let cell = app
        .world_mut()
        .spawn((
            Cell,
            Hp {
                current:  10.0,
                starting: 10.0,
                max:      None,
            },
            VolatilityTimer { elapsed: 2.0 },
        ))
        .id();

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    // Timer insert was skipped; elapsed may advance by dt if grow ran,
    // but must start from the pre-existing 2.0.
    assert!(
        timer.elapsed >= 2.0 - 1e-5,
        "existing timer must not be overwritten to 0.0, got {}",
        timer.elapsed
    );
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max,
        Some(20.0),
        "Hp.max should still be lifted to Some(20.0)"
    );
}

// Behavior 16 — attach inactive hazard does nothing
#[test]
fn attach_inactive_hazard_does_not_insert_timer_or_lift_max() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    // No Volatility stacks.
    register_volatility_systems(&mut app);
    let cell = spawn_cell_no_timer(&mut app, 10.0, 10.0, None);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(app.world().get::<VolatilityTimer>(cell).is_none());
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        hp.max.is_none(),
        "Hp.max should remain None, got {:?}",
        hp.max
    );
    assert_eq!(heal_collector_len(&app), 0);
}

// Behavior 16 edge — hazard becomes active mid-run
#[test]
fn attach_reacts_when_hazard_becomes_active_mid_run() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_no_timer(&mut app, 10.0, 10.0, None);

    // Tick 1 — no timer, no lift.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    assert!(app.world().get::<VolatilityTimer>(cell).is_none());
    assert!(app.world().get::<Hp>(cell).unwrap().max.is_none());

    // Activate mid-run.
    add_volatility_stacks(&mut app, 1);

    // Tick 2.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    // After an attach in the same tick, grow advances elapsed by dt.
    // Tolerate up to one fixed-update tick's worth of accumulation.
    assert!(
        timer.elapsed < 0.02,
        "timer.elapsed should be near 0.0 after attach (≤ one tick's dt), got {}",
        timer.elapsed
    );
    assert_eq!(app.world().get::<Hp>(cell).unwrap().max, Some(20.0));
}

// Behavior 17 — cells spawned mid-node receive timer on next tick
#[test]
fn cells_spawned_mid_node_receive_timer_on_next_tick() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let a = spawn_cell_no_timer(&mut app, 10.0, 10.0, None);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    assert!(app.world().get::<VolatilityTimer>(a).is_some());
    assert_eq!(app.world().get::<Hp>(a).unwrap().max, Some(20.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let b = spawn_cell_no_timer(&mut app, 5.0, 5.0, None);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let timer_b = app.world().get::<VolatilityTimer>(b).unwrap();
    // After an attach in the same tick, grow advances elapsed by dt.
    // Tolerate up to one fixed-update tick's worth of accumulation.
    assert!(
        timer_b.elapsed < 0.02,
        "timer_b.elapsed should be near 0.0 after attach (≤ one tick's dt), got {}",
        timer_b.elapsed
    );
    assert_eq!(app.world().get::<Hp>(b).unwrap().max, Some(10.0));
    // A retains its timer and max.
    assert!(app.world().get::<VolatilityTimer>(a).is_some());
    assert_eq!(app.world().get::<Hp>(a).unwrap().max, Some(20.0));
}

// Behavior 17 edge — cell B spawned with pre-existing timer
#[test]
fn mid_node_cell_with_existing_timer_keeps_it_but_gets_max_lifted() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let b = app
        .world_mut()
        .spawn((
            Cell,
            Hp {
                current:  5.0,
                starting: 5.0,
                max:      None,
            },
            VolatilityTimer { elapsed: 2.0 },
        ))
        .id();

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let timer_b = app.world().get::<VolatilityTimer>(b).unwrap();
    assert!(
        timer_b.elapsed >= 2.0 - 1e-5,
        "existing timer must not be overwritten to 0.0, got {}",
        timer_b.elapsed
    );
    assert_eq!(app.world().get::<Hp>(b).unwrap().max, Some(10.0));
}

// Behavior 18 — Hp.max lift from None to Some(2 * starting)
#[test]
fn attach_lifts_none_max_to_2x_starting() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_no_timer(&mut app, 5.0, 10.0, None);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(hp.max, Some(20.0));
    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    // After an attach in the same tick, grow advances elapsed by dt.
    // Tolerate up to one fixed-update tick's worth of accumulation.
    assert!(
        timer.elapsed < 0.02,
        "timer.elapsed should be near 0.0 after attach (≤ one tick's dt), got {}",
        timer.elapsed
    );
}

// Behavior 18 edge — starting = 0.0 yields Hp.max = Some(0.0)
#[test]
fn attach_zero_starting_yields_zero_max() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_no_timer(&mut app, 0.0, 0.0, None);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(hp.max, Some(0.0));
}

// Behavior 18 edge — max_multiplier = 1.0 yields Hp.max = Some(starting)
#[test]
fn attach_multiplier_one_yields_starting_max() {
    let mut app = test_app_playing();
    app.world_mut().insert_resource(VolatilityConfig {
        hp_per_interval: 1.0,
        interval_secs:   5.0,
        max_multiplier:  1.0,
    });
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_no_timer(&mut app, 5.0, 10.0, None);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(hp.max, Some(10.0));
}

// Behavior 19 — higher existing max preserved
#[test]
fn attach_preserves_higher_existing_max() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_no_timer(&mut app, 5.0, 10.0, Some(30.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max,
        Some(30.0),
        "existing higher max must not be lowered"
    );
    assert!(app.world().get::<VolatilityTimer>(cell).is_some());
}

// Behavior 19 edge — existing max equals 2x starting
#[test]
fn attach_preserves_max_equal_to_2x_starting() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_no_timer(&mut app, 5.0, 10.0, Some(20.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(hp.max, Some(20.0));
}

// Behavior 19 edge — existing max is infinity
#[test]
fn attach_preserves_infinite_existing_max() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_no_timer(&mut app, 5.0, 10.0, Some(f32::INFINITY));

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(hp.max, Some(f32::INFINITY));
}

// Behavior 20 — lower existing max raised to 2x starting
#[test]
fn attach_raises_lower_existing_max_to_2x_starting() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_no_timer(&mut app, 5.0, 10.0, Some(12.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(hp.max, Some(20.0));
    assert!(app.world().get::<VolatilityTimer>(cell).is_some());
}

// Behavior 20 edge — slightly below 2x starting
#[test]
fn attach_raises_19_999_to_20() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_no_timer(&mut app, 5.0, 10.0, Some(19.999));

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(hp.max, Some(20.0));
}

// Behavior 20 edge — existing max = 0.0 is raised to 2x starting
#[test]
fn attach_raises_zero_existing_max_to_2x_starting() {
    let mut app = test_app_playing();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_no_timer(&mut app, 5.0, 10.0, Some(0.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(hp.max, Some(20.0));
}
