//! Group E — Phase Transitions
//!
//! Verifies that when the timer reaches zero, the phase transitions to the
//! next and the timer resets to the correct duration.

use std::time::Duration;

use super::helpers::*;
use crate::{
    cells::behaviors::phantom::components::{PhantomConfig, PhantomPhase, PhantomTimer},
    prelude::*,
};

// Behavior 25: Solid to Telegraph transition when timer expires
#[test]
fn solid_to_telegraph_transition_when_timer_expires() {
    let mut app = build_phantom_test_app();
    let config = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: 0.5,
    };
    let entity = spawn_phantom_cell_raw(
        &mut app,
        PhantomPhase::Solid,
        0.01,
        config,
        CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
    );

    advance_to_playing(&mut app);
    // dt = 1/60s (~0.0167), exceeds timer 0.01
    tick_with_dt(&mut app, Duration::from_nanos(16_666_667));

    let phase = app
        .world()
        .get::<PhantomPhase>(entity)
        .expect("entity should have PhantomPhase");
    assert_eq!(
        *phase,
        PhantomPhase::Telegraph,
        "phase should transition from Solid to Telegraph"
    );

    let timer = app.world().get::<PhantomTimer>(entity).unwrap();
    assert!(
        (timer.0 - 0.5).abs() < f32::EPSILON,
        "timer should reset to telegraph duration 0.5, got {}",
        timer.0
    );

    // Edge: CollisionLayers should remain standard (no layer change on Solid->Telegraph)
    let layers = app.world().get::<CollisionLayers>(entity).unwrap();
    assert_eq!(
        layers.membership, CELL_LAYER,
        "layers should remain standard on Solid->Telegraph, got 0x{:02X}",
        layers.membership
    );
    assert_eq!(layers.mask, BOLT_LAYER);
}

// Behavior 26: Telegraph to Ghost transition when timer expires: layers zeroed
#[test]
fn telegraph_to_ghost_transition_zeroes_layers() {
    let mut app = build_phantom_test_app();
    let config = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: 0.5,
    };
    let entity = spawn_phantom_cell_raw(
        &mut app,
        PhantomPhase::Telegraph,
        0.01,
        config,
        CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
    );

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_nanos(16_666_667));

    let phase = app.world().get::<PhantomPhase>(entity).unwrap();
    assert_eq!(
        *phase,
        PhantomPhase::Ghost,
        "phase should transition from Telegraph to Ghost"
    );

    let timer = app.world().get::<PhantomTimer>(entity).unwrap();
    assert!(
        (timer.0 - 3.0).abs() < f32::EPSILON,
        "timer should reset to ghost duration (cycle_secs) = 3.0, got {}",
        timer.0
    );

    let layers = app.world().get::<CollisionLayers>(entity).unwrap();
    assert_eq!(
        layers.membership, 0,
        "Ghost phase should have zeroed membership, got 0x{:02X}",
        layers.membership
    );
    assert_eq!(
        layers.mask, 0,
        "Ghost phase should have zeroed mask, got 0x{:02X}",
        layers.mask
    );
}

// Behavior 27: Ghost to Solid transition when timer expires: layers restored
#[test]
fn ghost_to_solid_transition_restores_layers() {
    let mut app = build_phantom_test_app();
    let config = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: 0.5,
    };
    let entity = spawn_phantom_cell_raw(
        &mut app,
        PhantomPhase::Ghost,
        0.01,
        config,
        CollisionLayers::new(0, 0), // start with zeroed layers
    );

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_nanos(16_666_667));

    let phase = app.world().get::<PhantomPhase>(entity).unwrap();
    assert_eq!(
        *phase,
        PhantomPhase::Solid,
        "phase should transition from Ghost to Solid"
    );

    let timer = app.world().get::<PhantomTimer>(entity).unwrap();
    assert!(
        (timer.0 - 2.5).abs() < f32::EPSILON,
        "timer should reset to solid duration (cycle_secs - telegraph_secs) = 2.5, got {}",
        timer.0
    );

    let layers = app.world().get::<CollisionLayers>(entity).unwrap();
    assert_eq!(
        layers.membership, CELL_LAYER,
        "Solid phase should restore CELL_LAYER membership, got 0x{:02X}",
        layers.membership
    );
    assert_eq!(
        layers.mask, BOLT_LAYER,
        "Solid phase should restore BOLT_LAYER mask, got 0x{:02X}",
        layers.mask
    );
}

// Behavior 28: Timer exactly at zero triggers transition
#[test]
fn timer_exactly_at_zero_triggers_transition() {
    let mut app = build_phantom_test_app();
    let config = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: 0.5,
    };
    let entity = spawn_phantom_cell_raw(
        &mut app,
        PhantomPhase::Solid,
        0.0,
        config,
        CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
    );

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_nanos(16_666_667));

    let phase = app.world().get::<PhantomPhase>(entity).unwrap();
    assert_eq!(
        *phase,
        PhantomPhase::Telegraph,
        "timer at exactly 0.0 should trigger Solid->Telegraph transition"
    );

    let timer = app.world().get::<PhantomTimer>(entity).unwrap();
    assert!(
        (timer.0 - 0.5).abs() < f32::EPSILON,
        "timer should reset to telegraph duration 0.5, got {}",
        timer.0
    );
}

// Behavior 28 edge: timer already slightly negative also transitions
#[test]
fn timer_slightly_negative_triggers_transition() {
    let mut app = build_phantom_test_app();
    let config = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: 0.5,
    };
    let entity = spawn_phantom_cell_raw(
        &mut app,
        PhantomPhase::Solid,
        -0.001,
        config,
        CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
    );

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_nanos(16_666_667));

    let phase = app.world().get::<PhantomPhase>(entity).unwrap();
    assert_eq!(
        *phase,
        PhantomPhase::Telegraph,
        "timer at -0.001 should trigger transition"
    );
}

// Behavior 29: Timer resets to correct duration for Solid phase
#[test]
fn timer_resets_to_solid_duration_on_ghost_to_solid() {
    let mut app = build_phantom_test_app();
    let config = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: 0.5,
    };
    let entity = spawn_phantom_cell_raw(
        &mut app,
        PhantomPhase::Ghost,
        0.01,
        config,
        CollisionLayers::new(0, 0),
    );

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_nanos(16_666_667));

    let timer = app.world().get::<PhantomTimer>(entity).unwrap();
    assert!(
        (timer.0 - 2.5).abs() < f32::EPSILON,
        "timer should reset to Solid duration = cycle_secs - telegraph_secs = 2.5, got {}",
        timer.0
    );
}

// Behavior 30: Timer resets to correct duration for Telegraph phase
#[test]
fn timer_resets_to_telegraph_duration_on_solid_to_telegraph() {
    let mut app = build_phantom_test_app();
    let config = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: 0.5,
    };
    let entity = spawn_phantom_cell_raw(
        &mut app,
        PhantomPhase::Solid,
        0.01,
        config,
        CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
    );

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_nanos(16_666_667));

    let timer = app.world().get::<PhantomTimer>(entity).unwrap();
    assert!(
        (timer.0 - 0.5).abs() < f32::EPSILON,
        "timer should reset to Telegraph duration = telegraph_secs = 0.5, got {}",
        timer.0
    );
}

// Behavior 31: Timer resets to correct duration for Ghost phase
#[test]
fn timer_resets_to_ghost_duration_on_telegraph_to_ghost() {
    let mut app = build_phantom_test_app();
    let config = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: 0.5,
    };
    let entity = spawn_phantom_cell_raw(
        &mut app,
        PhantomPhase::Telegraph,
        0.01,
        config,
        CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
    );

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_nanos(16_666_667));

    let timer = app.world().get::<PhantomTimer>(entity).unwrap();
    assert!(
        (timer.0 - 3.0).abs() < f32::EPSILON,
        "timer should reset to Ghost duration = cycle_secs = 3.0, got {}",
        timer.0
    );
}
