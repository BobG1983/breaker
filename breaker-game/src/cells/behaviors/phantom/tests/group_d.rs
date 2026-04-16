//! Group D — Timer Decrement
//!
//! Verifies that `tick_phantom_phase` decrements `PhantomTimer` each tick
//! when in `NodeState::Playing`.

use std::time::Duration;

use super::helpers::*;
use crate::{
    cells::behaviors::phantom::components::{PhantomConfig, PhantomPhase, PhantomTimer},
    prelude::*,
};

// Behavior 22: Timer decrements by dt each tick
#[test]
fn timer_decrements_by_dt_each_tick() {
    let mut app = build_phantom_test_app();
    let config = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: 0.5,
    };
    let entity = spawn_phantom_cell_raw(
        &mut app,
        PhantomPhase::Solid,
        2.5,
        config,
        CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
    );

    advance_to_playing(&mut app);

    // dt = 1/60s = ~0.016667s
    let dt = Duration::from_nanos(16_666_667); // 1/60 second
    tick_with_dt(&mut app, dt);

    let timer = app
        .world()
        .get::<PhantomTimer>(entity)
        .expect("entity should have PhantomTimer");
    let expected = 2.5 - (1.0 / 60.0);
    assert!(
        (timer.0 - expected).abs() < 0.001,
        "timer should be approximately {expected} after one tick, got {}",
        timer.0
    );
}

// Behavior 22 edge: two consecutive ticks reduce timer by 2 * dt
#[test]
fn timer_decrements_across_two_ticks() {
    let mut app = build_phantom_test_app();
    let config = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: 0.5,
    };
    let entity = spawn_phantom_cell_raw(
        &mut app,
        PhantomPhase::Solid,
        2.5,
        config,
        CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
    );

    advance_to_playing(&mut app);

    let dt = Duration::from_nanos(16_666_667);
    tick_with_dt(&mut app, dt);
    tick_with_dt(&mut app, dt);

    let timer = app
        .world()
        .get::<PhantomTimer>(entity)
        .expect("entity should have PhantomTimer");
    let expected = 2.0f32.mul_add(-(1.0 / 60.0), 2.5);
    assert!(
        (timer.0 - expected).abs() < 0.001,
        "timer should be approximately {expected} after two ticks, got {}",
        timer.0
    );
}

// Behavior 23: Timer decrements with large dt
#[test]
fn timer_decrements_with_large_dt() {
    let mut app = build_phantom_test_app();
    let config = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: 0.5,
    };
    let entity = spawn_phantom_cell_raw(
        &mut app,
        PhantomPhase::Ghost,
        3.0,
        config,
        CollisionLayers::new(0, 0),
    );

    advance_to_playing(&mut app);

    tick_with_dt(&mut app, Duration::from_secs(1));

    let timer = app
        .world()
        .get::<PhantomTimer>(entity)
        .expect("entity should have PhantomTimer");
    assert!(
        (timer.0 - 2.0).abs() < 0.001,
        "timer should be approximately 2.0 after 1.0s tick, got {}",
        timer.0
    );
}

// Behavior 24: Timer does not decrement when not in NodeState::Playing
#[test]
fn timer_does_not_decrement_when_not_playing() {
    let mut app = build_phantom_test_app();
    let config = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: 0.5,
    };
    let entity = spawn_phantom_cell_raw(
        &mut app,
        PhantomPhase::Solid,
        2.5,
        config,
        CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
    );

    // Do NOT advance to playing — stay in Loading
    tick_with_dt(&mut app, Duration::from_nanos(16_666_667));

    let timer = app
        .world()
        .get::<PhantomTimer>(entity)
        .expect("entity should have PhantomTimer");
    assert!(
        (timer.0 - 2.5).abs() < f32::EPSILON,
        "timer should remain 2.5 when not in Playing state, got {}",
        timer.0
    );
}

// Behavior 24 edge: transitioning to Playing then ticking causes decrement
#[test]
fn timer_decrements_after_transition_to_playing() {
    let mut app = build_phantom_test_app();
    let config = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: 0.5,
    };
    let entity = spawn_phantom_cell_raw(
        &mut app,
        PhantomPhase::Solid,
        2.5,
        config,
        CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
    );

    // Not playing yet — tick should not decrement
    tick_with_dt(&mut app, Duration::from_nanos(16_666_667));
    let timer = app.world().get::<PhantomTimer>(entity).unwrap();
    assert!(
        (timer.0 - 2.5).abs() < f32::EPSILON,
        "timer should remain 2.5 before Playing, got {}",
        timer.0
    );

    // Now advance to Playing
    advance_to_playing(&mut app);

    tick_with_dt(&mut app, Duration::from_nanos(16_666_667));
    let timer = app.world().get::<PhantomTimer>(entity).unwrap();
    let expected = 2.5 - (1.0 / 60.0);
    assert!(
        (timer.0 - expected).abs() < 0.001,
        "timer should decrement after entering Playing, expected {expected}, got {}",
        timer.0
    );
}

// Post-Review Correction 5: negative test — non-PhantomCell entity is not ticked
#[test]
fn non_phantom_cell_entity_is_not_ticked() {
    let mut app = build_phantom_test_app();

    // Spawn an entity with PhantomPhase/PhantomTimer/PhantomConfig but WITHOUT
    // PhantomCell marker — the With<PhantomCell> filter should exclude it.
    let entity = app
        .world_mut()
        .spawn((
            Cell,
            // NO PhantomCell marker
            PhantomPhase::Solid,
            PhantomTimer(2.5),
            PhantomConfig {
                cycle_secs:     3.0,
                telegraph_secs: 0.5,
            },
            CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
            Hp::new(20.0),
            KilledBy::default(),
        ))
        .id();

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_secs(1));

    let timer = app
        .world()
        .get::<PhantomTimer>(entity)
        .expect("entity should still have PhantomTimer");
    assert!(
        (timer.0 - 2.5).abs() < f32::EPSILON,
        "entity without PhantomCell marker should NOT be ticked, timer should remain 2.5, got {}",
        timer.0
    );
}
