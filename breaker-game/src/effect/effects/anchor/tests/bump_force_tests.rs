//! Tests for planted breaker bump force interactions.

use super::helpers::*;

// ── Behavior 9: Planted breaker has anchor bump_force_multiplier in ActiveBumpForces ──

#[test]
fn planted_breaker_has_anchor_force_in_active_bump_forces() {
    // Given: AnchorActive with bump_force_multiplier 2.0, ActiveBumpForces empty, stationary.
    // When: tick_anchor runs through timer countdown until planted.
    // Then: ActiveBumpForces contains [2.0].
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
            ActiveBumpForces(vec![]),
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    // First tick: inserts AnchorTimer(0.3)
    tick(&mut app);
    assert!(
        app.world().get::<AnchorTimer>(entity).is_some(),
        "first tick should insert AnchorTimer"
    );

    // Tick enough frames to exhaust the timer (~20 ticks at dt=1/64)
    for _ in 0..20 {
        tick(&mut app);
    }

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_some(),
        "breaker should be planted after timer expires"
    );
    assert!(
        app.world().get::<AnchorTimer>(entity).is_none(),
        "AnchorTimer should be removed after planting"
    );

    let forces = app.world().get::<ActiveBumpForces>(entity).unwrap();
    assert_eq!(
        forces.0,
        vec![2.0],
        "ActiveBumpForces should contain [2.0] after planting, got {:?}",
        forces.0
    );
}

#[test]
fn planted_breaker_appends_force_to_existing_entries() {
    // Edge case: ActiveBumpForces already has [1.5] from other effects.
    // After planting, should be [1.5, 2.0].
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
            ActiveBumpForces(vec![1.5]),
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    // First tick: inserts AnchorTimer(0.3)
    tick(&mut app);

    // Tick enough frames for timer to expire
    for _ in 0..20 {
        tick(&mut app);
    }

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_some(),
        "breaker should be planted"
    );

    let forces = app.world().get::<ActiveBumpForces>(entity).unwrap();
    assert_eq!(
        forces.0,
        vec![1.5, 2.0],
        "ActiveBumpForces should append anchor's 2.0 to existing [1.5], got {:?}",
        forces.0
    );
}

// ── Behavior 10: Movement removes AnchorPlanted and pops bump_force_multiplier ──

#[test]
fn movement_removes_planted_and_pops_force_multiplier() {
    // Given: Planted with ActiveBumpForces [1.5, 2.0] (2.0 from anchor).
    // When: BreakerVelocity.x set to 200.0, tick_anchor runs.
    // Then: AnchorPlanted removed. ActiveBumpForces is [1.5].
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorPlanted,
            ActiveBumpForces(vec![1.5, 2.0]),
            BreakerVelocity { x: 200.0 },
            BreakerState::Idle,
        ))
        .id();

    app.update();

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_none(),
        "AnchorPlanted should be removed on movement"
    );

    let forces = app.world().get::<ActiveBumpForces>(entity).unwrap();
    assert_eq!(
        forces.0,
        vec![1.5],
        "anchor's 2.0 should be popped from ActiveBumpForces, got {:?}",
        forces.0
    );
}

#[test]
fn movement_pops_force_from_single_entry_forces() {
    // Edge case: ActiveBumpForces only contained [2.0], becomes [] after pop.
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorPlanted,
            ActiveBumpForces(vec![2.0]),
            BreakerVelocity { x: 200.0 },
            BreakerState::Idle,
        ))
        .id();

    app.update();

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_none(),
        "AnchorPlanted should be removed on movement"
    );

    let forces = app.world().get::<ActiveBumpForces>(entity).unwrap();
    assert!(
        forces.0.is_empty(),
        "ActiveBumpForces should be empty after popping sole entry, got {:?}",
        forces.0
    );
}

// ── Behavior 11: No AnchorPlanted means ActiveBumpForces unchanged ──

#[test]
fn no_planted_means_active_bump_forces_unchanged() {
    // AnchorActive present, moving (no timer/planted). ActiveBumpForces [1.5] stays.
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            ActiveBumpForces(vec![1.5]),
            BreakerVelocity { x: 200.0 },
            BreakerState::Idle,
        ))
        .id();

    app.update();

    let forces = app.world().get::<ActiveBumpForces>(entity).unwrap();
    assert_eq!(
        forces.0,
        vec![1.5],
        "ActiveBumpForces should remain [1.5] when not planted, got {:?}",
        forces.0
    );
}

#[test]
fn no_anchor_active_means_active_bump_forces_unchanged() {
    // Edge case: No AnchorActive at all. ActiveBumpForces [1.5] stays.
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            ActiveBumpForces(vec![1.5]),
            BreakerVelocity { x: 200.0 },
            BreakerState::Idle,
        ))
        .id();

    app.update();

    let forces = app.world().get::<ActiveBumpForces>(entity).unwrap();
    assert_eq!(
        forces.0,
        vec![1.5],
        "ActiveBumpForces should remain [1.5] without AnchorActive, got {:?}",
        forces.0
    );
}

// ── Behavior 12: Re-planting after cancellation re-adds force entry ──

#[test]
fn replanting_after_cancellation_readds_single_force_entry() {
    // Full cycle: stationary -> timer -> planted (force pushed) -> movement (force popped)
    // -> stationary again -> new timer -> new planted (force pushed).
    // After final planting, ActiveBumpForces contains exactly one anchor entry [2.0].
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
            ActiveBumpForces(vec![]),
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    // Phase 1: stationary -> timer starts -> timer expires -> planted (force pushed)
    // First tick inserts timer
    tick(&mut app);
    // Tick enough for timer to expire (~20 ticks for 0.3s at 1/64)
    for _ in 0..20 {
        tick(&mut app);
    }

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_some(),
        "should be planted after first cycle"
    );
    let forces = app.world().get::<ActiveBumpForces>(entity).unwrap();
    assert_eq!(
        forces.0,
        vec![2.0],
        "ActiveBumpForces should be [2.0] after first planting"
    );

    // Phase 2: movement -> unplanted (force popped)
    app.world_mut()
        .get_mut::<BreakerVelocity>(entity)
        .unwrap()
        .x = 200.0;
    tick(&mut app);

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_none(),
        "should be unplanted after movement"
    );
    let forces = app.world().get::<ActiveBumpForces>(entity).unwrap();
    assert!(
        forces.0.is_empty(),
        "ActiveBumpForces should be empty after unplanting"
    );

    // Phase 3: stationary again -> new timer -> new planted (force pushed)
    app.world_mut()
        .get_mut::<BreakerVelocity>(entity)
        .unwrap()
        .x = 0.0;

    // First tick: new timer starts
    tick(&mut app);
    assert!(
        app.world().get::<AnchorTimer>(entity).is_some(),
        "new timer should start after stopping"
    );

    // Edge case verification: timer starts fresh from plant_delay, not residual
    let timer = app.world().get::<AnchorTimer>(entity).unwrap();
    assert!(
        (timer.0 - 0.3).abs() < 0.02,
        "new timer should start fresh from plant_delay (0.3), got {}",
        timer.0
    );

    // Tick enough for timer to expire again
    for _ in 0..20 {
        tick(&mut app);
    }

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_some(),
        "should be planted after second cycle"
    );
    let forces = app.world().get::<ActiveBumpForces>(entity).unwrap();
    assert_eq!(
        forces.0,
        vec![2.0],
        "ActiveBumpForces should contain exactly one [2.0] after replanting (not cumulative)"
    );
}
