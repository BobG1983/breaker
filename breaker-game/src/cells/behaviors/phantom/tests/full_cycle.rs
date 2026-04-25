//! Group F — Full Cycle and Starting Phase Variations
//!
//! Integration tests verifying complete phase cycles and non-default
//! starting phases.

use std::time::Duration;

use super::helpers::*;
use crate::{
    cells::behaviors::phantom::components::{PhantomConfig, PhantomPhase, PhantomTimer},
    prelude::*,
};

// Behavior 32: Full cycle Solid -> Telegraph -> Ghost -> Solid round-trip
#[test]
fn full_cycle_solid_telegraph_ghost_solid_round_trip() {
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

    // Solid phase: timer = 2.5, dt = 1.0s
    // After 3 ticks (3.0s), timer goes 2.5 -> 1.5 -> 0.5 -> transition at tick 3
    for _ in 0..3 {
        tick_with_dt(&mut app, Duration::from_secs(1));
    }
    let phase = app.world().get::<PhantomPhase>(entity).unwrap();
    assert_eq!(
        *phase,
        PhantomPhase::Telegraph,
        "after 3s, should have transitioned from Solid to Telegraph"
    );

    // Telegraph phase: timer = 0.5, dt = 1.0s
    // After 1 tick, timer goes 0.5 -> transition
    tick_with_dt(&mut app, Duration::from_secs(1));
    let phase = app.world().get::<PhantomPhase>(entity).unwrap();
    assert_eq!(
        *phase,
        PhantomPhase::Ghost,
        "after Telegraph timer expires, should be Ghost"
    );

    // Verify layers zeroed during Ghost
    let layers = app.world().get::<CollisionLayers>(entity).unwrap();
    assert_eq!(
        layers.membership, 0,
        "Ghost phase should have zeroed membership"
    );
    assert_eq!(layers.mask, 0, "Ghost phase should have zeroed mask");

    // Ghost phase: timer = 3.0, dt = 1.0s
    // After 3 ticks, timer goes 3.0 -> 2.0 -> 1.0 -> transition at tick 3
    for _ in 0..3 {
        tick_with_dt(&mut app, Duration::from_secs(1));
    }

    let phase = app.world().get::<PhantomPhase>(entity).unwrap();
    assert_eq!(
        *phase,
        PhantomPhase::Solid,
        "after full cycle, should be back to Solid"
    );

    let timer = app.world().get::<PhantomTimer>(entity).unwrap();
    assert!(
        (timer.0 - 2.5).abs() < 0.01,
        "timer should reset to Solid duration ~2.5, got {}",
        timer.0
    );

    // Verify layers restored on return to Solid
    let layers = app.world().get::<CollisionLayers>(entity).unwrap();
    assert_eq!(
        layers.membership, CELL_LAYER,
        "Solid phase should restore CELL_LAYER membership"
    );
    assert_eq!(
        layers.mask, BOLT_LAYER,
        "Solid phase should restore BOLT_LAYER mask"
    );
}

// Behavior 33: Starting in Ghost transitions to Solid when timer expires
#[test]
fn starting_in_ghost_transitions_to_solid() {
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

    let phase = app.world().get::<PhantomPhase>(entity).unwrap();
    assert_eq!(
        *phase,
        PhantomPhase::Solid,
        "Ghost should transition to Solid"
    );

    let layers = app.world().get::<CollisionLayers>(entity).unwrap();
    assert_eq!(layers.membership, CELL_LAYER);
    assert_eq!(layers.mask, BOLT_LAYER);

    let timer = app.world().get::<PhantomTimer>(entity).unwrap();
    assert!(
        (timer.0 - 2.5).abs() < f32::EPSILON,
        "timer should reset to Solid duration 2.5, got {}",
        timer.0
    );
}

// Behavior 34: Starting in Telegraph transitions to Ghost when timer expires
#[test]
fn starting_in_telegraph_transitions_to_ghost() {
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
        "Telegraph should transition to Ghost"
    );

    let layers = app.world().get::<CollisionLayers>(entity).unwrap();
    assert_eq!(
        layers.membership, 0,
        "Ghost phase should have zeroed layers"
    );
    assert_eq!(layers.mask, 0);

    let timer = app.world().get::<PhantomTimer>(entity).unwrap();
    assert!(
        (timer.0 - 3.0).abs() < f32::EPSILON,
        "timer should reset to Ghost duration 3.0, got {}",
        timer.0
    );
}

// Behavior 35: Multiple phantom cells cycle independently
#[test]
fn multiple_phantom_cells_cycle_independently() {
    let mut app = build_phantom_test_app();

    // Cell A: Solid, timer=0.01, config(3.0, 0.5)
    let config_a = PhantomConfig {
        cycle_secs:     3.0,
        telegraph_secs: 0.5,
    };
    let entity_a = spawn_phantom_cell_raw(
        &mut app,
        PhantomPhase::Solid,
        0.01,
        config_a,
        CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
    );

    // Cell B: Ghost, timer=0.01, config(2.0, 0.3)
    let config_b = PhantomConfig {
        cycle_secs:     2.0,
        telegraph_secs: 0.3,
    };
    let entity_b = spawn_phantom_cell_raw(
        &mut app,
        PhantomPhase::Ghost,
        0.01,
        config_b,
        CollisionLayers::new(0, 0),
    );

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_nanos(16_666_667));

    // Cell A: Solid -> Telegraph, timer = 0.5
    let phase_a = app.world().get::<PhantomPhase>(entity_a).unwrap();
    assert_eq!(
        *phase_a,
        PhantomPhase::Telegraph,
        "cell A should transition to Telegraph"
    );
    let timer_a = app.world().get::<PhantomTimer>(entity_a).unwrap();
    assert!(
        (timer_a.0 - 0.5).abs() < f32::EPSILON,
        "cell A timer should be 0.5, got {}",
        timer_a.0
    );

    // Cell B: Ghost -> Solid, timer = 2.0 - 0.3 = 1.7
    let phase_b = app.world().get::<PhantomPhase>(entity_b).unwrap();
    assert_eq!(
        *phase_b,
        PhantomPhase::Solid,
        "cell B should transition to Solid"
    );
    let timer_b = app.world().get::<PhantomTimer>(entity_b).unwrap();
    assert!(
        (timer_b.0 - 1.7).abs() < f32::EPSILON,
        "cell B timer should be 1.7 (2.0 - 0.3), got {}",
        timer_b.0
    );
}

// Behavior 36: Custom config with large cycle full Solid phase
#[test]
fn custom_config_large_cycle_full_solid_phase() {
    let mut app = build_phantom_test_app();
    let config = PhantomConfig {
        cycle_secs:     10.0,
        telegraph_secs: 2.0,
    };
    let entity = spawn_phantom_cell_raw(
        &mut app,
        PhantomPhase::Solid,
        8.0,
        config,
        CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
    );

    advance_to_playing(&mut app);

    // 7 ticks at dt = 1.0s: timer goes 8.0 -> 1.0
    for _ in 0..7 {
        tick_with_dt(&mut app, Duration::from_secs(1));
    }

    let phase = app.world().get::<PhantomPhase>(entity).unwrap();
    assert_eq!(
        *phase,
        PhantomPhase::Solid,
        "after 7 ticks, should still be Solid (timer ~1.0)"
    );

    let timer = app.world().get::<PhantomTimer>(entity).unwrap();
    assert!(
        (timer.0 - 1.0).abs() < 0.01,
        "timer should be approximately 1.0 after 7 ticks, got {}",
        timer.0
    );

    // 8th tick: timer -> ~0.0, triggers transition to Telegraph
    tick_with_dt(&mut app, Duration::from_secs(1));

    let phase = app.world().get::<PhantomPhase>(entity).unwrap();
    assert_eq!(
        *phase,
        PhantomPhase::Telegraph,
        "8th tick should trigger transition to Telegraph"
    );
}
