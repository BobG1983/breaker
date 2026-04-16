//! Group B — Builder Integration
//!
//! Tests for the `.phantom()` and `.phantom_config()` builder methods and the
//! `spawn_inner()` `CellBehavior::Phantom` match arm.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    cells::{
        behaviors::phantom::components::{PhantomCell, PhantomConfig, PhantomPhase, PhantomTimer},
        definition::CellBehavior,
        test_utils::test_cell_definition,
    },
    prelude::*,
};

// Behavior 8: Builder .phantom(Solid) inserts PhantomCell marker
#[test]
fn phantom_builder_inserts_phantom_cell_marker() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .phantom(PhantomPhase::Solid)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<PhantomCell>(entity).is_some(),
        "entity should have PhantomCell marker"
    );

    // Edge case: entity also has Cell and Hp
    assert!(
        world.get::<Cell>(entity).is_some(),
        "entity should still have Cell component"
    );
    let hp = world.get::<Hp>(entity).expect("entity should have Hp");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "entity should have hp.current == 20.0, got {}",
        hp.current
    );
}

// Behavior 9: Builder .phantom(Solid) inserts PhantomPhase(Solid)
#[test]
fn phantom_builder_inserts_phantom_phase_solid() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .phantom(PhantomPhase::Solid)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let phase = world
        .get::<PhantomPhase>(entity)
        .expect("entity should have PhantomPhase");
    assert_eq!(*phase, PhantomPhase::Solid, "PhantomPhase should be Solid");
}

// Behavior 10: Builder .phantom(Solid) inserts PhantomTimer with Solid duration
#[test]
fn phantom_builder_inserts_phantom_timer_with_solid_duration() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .phantom(PhantomPhase::Solid)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let timer = world
        .get::<PhantomTimer>(entity)
        .expect("entity should have PhantomTimer");
    // Default config: cycle_secs=3.0, telegraph_secs=0.5 -> Solid duration = 2.5
    assert!(
        (timer.0 - 2.5).abs() < f32::EPSILON,
        "PhantomTimer should be 2.5 (Solid duration for default config), got {}",
        timer.0
    );
}

// Behavior 11: Builder .phantom(Solid) inserts PhantomConfig with defaults
#[test]
fn phantom_builder_inserts_phantom_config_with_defaults() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .phantom(PhantomPhase::Solid)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let config = world
        .get::<PhantomConfig>(entity)
        .expect("entity should have PhantomConfig");
    assert!(
        (config.cycle_secs - 3.0).abs() < f32::EPSILON,
        "cycle_secs should be 3.0, got {}",
        config.cycle_secs
    );
    assert!(
        (config.telegraph_secs - 0.5).abs() < f32::EPSILON,
        "telegraph_secs should be 0.5, got {}",
        config.telegraph_secs
    );
}

// Behavior 12: Builder .phantom(Ghost) starts in Ghost phase with Ghost duration timer
#[test]
fn phantom_builder_ghost_phase_has_ghost_duration_timer() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .phantom(PhantomPhase::Ghost)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let phase = world
        .get::<PhantomPhase>(entity)
        .expect("entity should have PhantomPhase");
    assert_eq!(*phase, PhantomPhase::Ghost, "phase should be Ghost");

    let timer = world
        .get::<PhantomTimer>(entity)
        .expect("entity should have PhantomTimer");
    // Ghost duration = cycle_secs = 3.0
    assert!(
        (timer.0 - 3.0).abs() < f32::EPSILON,
        "Ghost timer should be 3.0 (cycle_secs), got {}",
        timer.0
    );

    // Edge case: marker present
    assert!(
        world.get::<PhantomCell>(entity).is_some(),
        "entity should have PhantomCell marker"
    );
}

// Behavior 13: Builder .phantom(Telegraph) starts in Telegraph phase
#[test]
fn phantom_builder_telegraph_phase_has_telegraph_duration_timer() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .phantom(PhantomPhase::Telegraph)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let phase = world
        .get::<PhantomPhase>(entity)
        .expect("entity should have PhantomPhase");
    assert_eq!(*phase, PhantomPhase::Telegraph, "phase should be Telegraph");

    let timer = world
        .get::<PhantomTimer>(entity)
        .expect("entity should have PhantomTimer");
    // Telegraph duration = telegraph_secs = 0.5
    assert!(
        (timer.0 - 0.5).abs() < f32::EPSILON,
        "Telegraph timer should be 0.5 (telegraph_secs), got {}",
        timer.0
    );
}

// Behavior 14: Builder .phantom_config(5.0, 1.0, Ghost) sets explicit timing
#[test]
fn phantom_config_builder_sets_explicit_timing() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .phantom_config(5.0, 1.0, PhantomPhase::Ghost)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let config = world
        .get::<PhantomConfig>(entity)
        .expect("entity should have PhantomConfig");
    assert!(
        (config.cycle_secs - 5.0).abs() < f32::EPSILON,
        "cycle_secs should be 5.0, got {}",
        config.cycle_secs
    );
    assert!(
        (config.telegraph_secs - 1.0).abs() < f32::EPSILON,
        "telegraph_secs should be 1.0, got {}",
        config.telegraph_secs
    );

    let phase = world.get::<PhantomPhase>(entity).unwrap();
    assert_eq!(*phase, PhantomPhase::Ghost);

    let timer = world.get::<PhantomTimer>(entity).unwrap();
    // Ghost duration = cycle_secs = 5.0
    assert!(
        (timer.0 - 5.0).abs() < f32::EPSILON,
        "Ghost timer should be 5.0, got {}",
        timer.0
    );
}

// Behavior 15: Builder .phantom_config(2.0, 0.0, Solid) with zero telegraph
#[test]
fn phantom_config_builder_with_zero_telegraph() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .phantom_config(2.0, 0.0, PhantomPhase::Solid)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let config = world.get::<PhantomConfig>(entity).unwrap();
    assert!((config.cycle_secs - 2.0).abs() < f32::EPSILON);
    assert!((config.telegraph_secs - 0.0).abs() < f32::EPSILON);

    let phase = world.get::<PhantomPhase>(entity).unwrap();
    assert_eq!(*phase, PhantomPhase::Solid);

    let timer = world.get::<PhantomTimer>(entity).unwrap();
    // Solid duration = 2.0 - 0.0 = 2.0
    assert!(
        (timer.0 - 2.0).abs() < f32::EPSILON,
        "Solid timer with zero telegraph should be 2.0, got {}",
        timer.0
    );
}

// Behavior 16: Plain cell has no phantom components (negative test)
#[test]
fn plain_cell_has_no_phantom_components() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<PhantomCell>(entity).is_none(),
        "plain cell should not have PhantomCell"
    );
    assert!(
        world.get::<PhantomPhase>(entity).is_none(),
        "plain cell should not have PhantomPhase"
    );
    assert!(
        world.get::<PhantomTimer>(entity).is_none(),
        "plain cell should not have PhantomTimer"
    );
    assert!(
        world.get::<PhantomConfig>(entity).is_none(),
        "plain cell should not have PhantomConfig"
    );

    // Edge case: entity still has Hp (proves builder ran)
    let hp = world.get::<Hp>(entity).expect("entity should have Hp");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "plain cell should have hp.current == 20.0"
    );
}

// Behavior 17: Builder .with_behavior(CellBehavior::Phantom { .. }) matches .phantom() sugar
#[test]
fn with_behavior_phantom_matches_phantom_sugar() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_behavior(CellBehavior::Phantom {
                cycle_secs:     3.0,
                telegraph_secs: 0.5,
                starting_phase: PhantomPhase::Solid,
            })
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<PhantomCell>(entity).is_some());

    let phase = world.get::<PhantomPhase>(entity).unwrap();
    assert_eq!(*phase, PhantomPhase::Solid);

    let config = world.get::<PhantomConfig>(entity).unwrap();
    assert!((config.cycle_secs - 3.0).abs() < f32::EPSILON);
    assert!((config.telegraph_secs - 0.5).abs() < f32::EPSILON);

    let timer = world.get::<PhantomTimer>(entity).unwrap();
    assert!(
        (timer.0 - 2.5).abs() < f32::EPSILON,
        "timer should be 2.5, got {}",
        timer.0
    );
}

// Behavior 18: Builder phantom via .definition(&def) inserts phantom components
#[test]
fn definition_with_phantom_behavior_inserts_phantom_components() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![CellBehavior::Phantom {
        cycle_secs:     4.0,
        telegraph_secs: 0.8,
        starting_phase: PhantomPhase::Telegraph,
    }]);

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<PhantomCell>(entity).is_some(),
        "definition-sourced phantom should have PhantomCell marker"
    );

    let phase = world.get::<PhantomPhase>(entity).unwrap();
    assert_eq!(*phase, PhantomPhase::Telegraph);

    let config = world.get::<PhantomConfig>(entity).unwrap();
    assert!((config.cycle_secs - 4.0).abs() < f32::EPSILON);
    assert!((config.telegraph_secs - 0.8).abs() < f32::EPSILON);

    let timer = world.get::<PhantomTimer>(entity).unwrap();
    // Telegraph duration = 0.8
    assert!(
        (timer.0 - 0.8).abs() < f32::EPSILON,
        "timer should be 0.8 (Telegraph duration), got {}",
        timer.0
    );
}
