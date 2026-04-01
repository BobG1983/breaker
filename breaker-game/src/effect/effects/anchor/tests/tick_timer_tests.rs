//! Tests for `tick_anchor` timer behavior: insert, decrement, expire, cancel.

use super::helpers::*;

// ── Behavior 7: tick_anchor inserts AnchorTimer on stationary breaker ──

#[test]
fn tick_anchor_inserts_timer_on_stationary_idle_breaker() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    app.update();

    let timer = app
        .world()
        .get::<AnchorTimer>(entity)
        .expect("tick_anchor should insert AnchorTimer on stationary idle breaker");
    assert!(
        (timer.0 - 0.3).abs() < f32::EPSILON,
        "AnchorTimer should be initialized to plant_delay (0.3), got {}",
        timer.0
    );
}

#[test]
fn tick_anchor_inserts_timer_on_stationary_settling_breaker() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            BreakerVelocity { x: 0.0 },
            BreakerState::Settling,
        ))
        .id();

    app.update();

    let timer = app
        .world()
        .get::<AnchorTimer>(entity)
        .expect("tick_anchor should insert AnchorTimer on stationary settling breaker");
    assert!(
        (timer.0 - 0.3).abs() < f32::EPSILON,
        "AnchorTimer should be initialized to plant_delay (0.3), got {}",
        timer.0
    );
}

// ── Behavior 8: AnchorTimer ticks down by dt each frame ───────────

#[test]
fn tick_anchor_timer_decrements_by_dt_while_stationary() {
    let mut app = test_app_fixed();
    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorTimer(0.3),
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    tick(&mut app);

    let timer = app
        .world()
        .get::<AnchorTimer>(entity)
        .expect("AnchorTimer should still exist after one tick");
    // Fixed timestep is 1/64 by default in Bevy
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();
    let expected = 0.3 - dt;
    assert!(
        (timer.0 - expected).abs() < 0.001,
        "AnchorTimer should be approximately {expected}, got {}",
        timer.0
    );
}

#[test]
fn tick_anchor_timer_accumulates_over_multiple_ticks() {
    let mut app = test_app_fixed();
    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorTimer(0.3),
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    // Run 6 fixed ticks
    for _ in 0..6 {
        tick(&mut app);
    }

    let timer = app
        .world()
        .get::<AnchorTimer>(entity)
        .expect("AnchorTimer should still exist after 6 ticks");
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();
    let expected = (-6.0f32).mul_add(dt, 0.3);
    assert!(
        (timer.0 - expected).abs() < 0.001,
        "AnchorTimer should be approximately {expected} after 6 ticks, got {}",
        timer.0
    );
}

// ── Behavior 9: AnchorTimer reaches zero -> AnchorPlanted inserted ──

#[test]
fn tick_anchor_timer_reaching_zero_inserts_planted_and_removes_timer() {
    let mut app = test_app_fixed();
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();

    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            // Timer at 0.01 -- one tick at dt ~= 0.015625 will push it below zero.
            AnchorTimer(0.01),
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    tick(&mut app);

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_some(),
        "AnchorPlanted should be inserted when timer reaches zero (dt = {dt})"
    );
    assert!(
        app.world().get::<AnchorTimer>(entity).is_none(),
        "AnchorTimer should be removed when timer reaches zero"
    );
}

#[test]
fn tick_anchor_timer_exactly_one_dt_triggers_planted() {
    let mut app = test_app_fixed();
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();

    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            // Timer is exactly one dt -- after one tick it should be at or below zero.
            AnchorTimer(dt),
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    tick(&mut app);

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_some(),
        "AnchorPlanted should be inserted when timer was exactly one dt"
    );
    assert!(
        app.world().get::<AnchorTimer>(entity).is_none(),
        "AnchorTimer should be removed when timer was exactly one dt"
    );
}

// ── Behavior 10: breaker velocity cancels timer and planted ───────

#[test]
fn tick_anchor_velocity_cancels_timer_and_planted() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorTimer(0.15),
            AnchorPlanted,
            BreakerVelocity { x: 200.0 },
            BreakerState::Idle,
        ))
        .id();

    app.update();

    assert!(
        app.world().get::<AnchorTimer>(entity).is_none(),
        "AnchorTimer should be removed when breaker has velocity"
    );
    assert!(
        app.world().get::<AnchorPlanted>(entity).is_none(),
        "AnchorPlanted should be removed when breaker has velocity"
    );
    assert!(
        app.world().get::<AnchorActive>(entity).is_some(),
        "AnchorActive should still be present after movement cancel"
    );
}

// ── Behavior 11: dash state cancels timer and planted regardless of velocity ──

#[test]
fn tick_anchor_dashing_cancels_timer_and_planted_even_with_zero_velocity() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorTimer(0.15),
            AnchorPlanted,
            BreakerVelocity { x: 0.0 },
            BreakerState::Dashing,
        ))
        .id();

    app.update();

    assert!(
        app.world().get::<AnchorTimer>(entity).is_none(),
        "AnchorTimer should be removed during Dashing state"
    );
    assert!(
        app.world().get::<AnchorPlanted>(entity).is_none(),
        "AnchorPlanted should be removed during Dashing state"
    );
    assert!(
        app.world().get::<AnchorActive>(entity).is_some(),
        "AnchorActive should still be present after dash cancel"
    );
}

// ── Behavior 12: entity without AnchorActive is ignored ───────────

#[test]
fn tick_anchor_ignores_entity_without_anchor_active() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((BreakerVelocity { x: 0.0 }, BreakerState::Idle))
        .id();

    app.update();

    assert!(
        app.world().get::<AnchorTimer>(entity).is_none(),
        "AnchorTimer should not be inserted on entity without AnchorActive"
    );
    assert!(
        app.world().get::<AnchorPlanted>(entity).is_none(),
        "AnchorPlanted should not be inserted on entity without AnchorActive"
    );
}

// ── Behavior 13: steady-state planted breaker stays planted ───────

#[test]
fn tick_anchor_steady_state_planted_remains_planted_no_timer() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorPlanted,
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    app.update();

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_some(),
        "AnchorPlanted should remain on a stationary planted breaker"
    );
    assert!(
        app.world().get::<AnchorTimer>(entity).is_none(),
        "AnchorTimer should not be re-inserted on an already planted breaker"
    );
}

// ── Behavior 14: movement cancel then re-stop inserts full delay timer ──

#[test]
fn tick_anchor_movement_cancel_then_restop_inserts_full_delay_timer() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorTimer(0.15),
            BreakerVelocity { x: 200.0 },
            BreakerState::Idle,
        ))
        .id();

    // First tick: breaker is moving, timer should be cancelled.
    app.update();

    assert!(
        app.world().get::<AnchorTimer>(entity).is_none(),
        "AnchorTimer should be cancelled on first tick due to movement"
    );

    // Now stop the breaker.
    app.world_mut()
        .get_mut::<BreakerVelocity>(entity)
        .unwrap()
        .x = 0.0;

    // Second tick: breaker is stationary again, should get full plant_delay timer.
    app.update();

    let timer = app
        .world()
        .get::<AnchorTimer>(entity)
        .expect("AnchorTimer should be re-inserted when breaker stops");
    assert!(
        (timer.0 - 0.3).abs() < f32::EPSILON,
        "AnchorTimer should be full plant_delay (0.3), not residual. Got {}",
        timer.0
    );
}

// ── Behavior 15: Settling state with zero velocity allows timer tick ──

#[test]
fn tick_anchor_settling_state_allows_timer_tick() {
    let mut app = test_app_fixed();
    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorTimer(0.3),
            BreakerVelocity { x: 0.0 },
            BreakerState::Settling,
        ))
        .id();

    tick(&mut app);

    let timer = app
        .world()
        .get::<AnchorTimer>(entity)
        .expect("AnchorTimer should still exist after tick in Settling state");
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();
    let expected = 0.3 - dt;
    assert!(
        (timer.0 - expected).abs() < 0.001,
        "AnchorTimer should tick down in Settling state. Expected {expected}, got {}",
        timer.0
    );
}

// ── Behavior 13: tick_anchor handles missing ActiveBumpForces on plant ──

#[test]
fn tick_anchor_inserts_active_bump_forces_on_plant_when_missing() {
    // Given: No ActiveBumpForces. AnchorTimer about to expire.
    // When: tick_anchor runs, timer expires, AnchorPlanted inserted.
    // Then: ActiveBumpForces inserted with [2.0].
    let mut app = test_app_fixed();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorTimer(0.001), // about to expire
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
            // NO ActiveBumpForces
        ))
        .id();

    tick(&mut app);

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_some(),
        "AnchorPlanted should be inserted when timer expires"
    );

    let forces = app
        .world()
        .get::<ActiveBumpForces>(entity)
        .expect("ActiveBumpForces should be inserted when missing");
    assert_eq!(
        forces.0,
        vec![2.0],
        "ActiveBumpForces should contain [2.0], got {:?}",
        forces.0
    );
}

#[test]
fn tick_anchor_inserts_active_bump_forces_on_plant_when_no_active_forces() {
    // Edge case: no ActiveBumpForces on the entity before planting.
    // Only ActiveBumpForces should be inserted.
    let mut app = test_app_fixed();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorTimer(0.001), // about to expire
            // NO ActiveBumpForces
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    tick(&mut app);

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_some(),
        "AnchorPlanted should be inserted"
    );

    let forces = app
        .world()
        .get::<ActiveBumpForces>(entity)
        .expect("ActiveBumpForces should be inserted");
    assert_eq!(forces.0, vec![2.0], "ActiveBumpForces should contain [2.0]");
}
