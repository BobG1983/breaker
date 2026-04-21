use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::shared::death_pipeline::Hp;

// ════════════════════════════════════════════════════════════════════════════
// Group G — Full-pipeline integration with apply_heal::<Cell>
// ════════════════════════════════════════════════════════════════════════════

// Behavior 24: neighbour within radius gets healed end-to-end.
#[test]
fn pipeline_neighbour_within_radius_gets_healed() {
    let mut app = test_app_pipeline();
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

    let hp = app.world().get::<Hp>(neighbour).unwrap();
    assert!(
        (hp.current - 6.0).abs() < f32::EPSILON,
        "neighbour should heal 5.0 → 6.0 end-to-end, got {}",
        hp.current
    );
}

// Behavior 24 edge: at boundary distance 70.0 still heals.
#[test]
fn pipeline_neighbour_at_boundary_heals() {
    let mut app = test_app_pipeline();
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let neighbour = spawn_cell_at(&mut app, Vec2::new(70.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(neighbour).unwrap();
    assert!(
        (hp.current - 6.0).abs() < f32::EPSILON,
        "boundary neighbour (dist 70) heals, got {}",
        hp.current
    );
}

// Behavior 25: heal clamps at starting via HealCap::Starting.
#[test]
fn pipeline_heal_clamps_to_starting() {
    let mut app = test_app_pipeline();
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 0.0,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 8.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(neighbour).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "heal must clamp to starting (10.0), got {}",
        hp.current
    );
}

// Behavior 25 edge: at starting already — heal is a no-op.
#[test]
fn pipeline_heal_at_starting_is_noop() {
    let mut app = test_app_pipeline();
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 0.0,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 10.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(neighbour).unwrap();
    assert!((hp.current - 10.0).abs() < f32::EPSILON);
}

// Behavior 26: HealCap::Starting honoured even when hp.max > starting
// (Cascade + Volatility synergy).
#[test]
fn pipeline_heal_cap_is_starting_not_max_with_elevated_max() {
    let mut app = test_app_pipeline();
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 0.0,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    // Volatility has lifted hp.max to 20.0 on this neighbour. HealCap::Starting
    // still clamps at hp.starting = 10.0, NOT 20.0.
    let neighbour = spawn_cell_at_with_max(&mut app, Vec2::new(50.0, 0.0), 8.0, 10.0, Some(20.0));
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(neighbour).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "HealCap::Starting must clamp at starting (10.0), not max (20.0); got {}",
        hp.current
    );
}

// Behavior 27: multiple deaths, shared neighbour → Hp sums (no clamp needed).
#[test]
fn pipeline_multiple_deaths_shared_neighbour_sums_hp() {
    let mut app = test_app_pipeline();
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let n = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 5.0, 20.0);
    let va = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 0.0, 10.0);
    let vb = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 0.0, 10.0);
    send_cell_destroyed(&mut app, va, Vec2::new(50.0, 0.0));
    send_cell_destroyed(&mut app, vb, Vec2::new(-50.0, 0.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(n).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "two heals of 1.0 each → 5.0 + 2.0 = 7.0, got {}",
        hp.current
    );
}

// Behavior 27 edge: shared neighbour with starting = 6.0 clamps at 6.0.
#[test]
fn pipeline_multiple_deaths_clamp_at_starting() {
    let mut app = test_app_pipeline();
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let n = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 5.0, 6.0);
    let va = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 0.0, 10.0);
    let vb = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 0.0, 10.0);
    send_cell_destroyed(&mut app, va, Vec2::new(50.0, 0.0));
    send_cell_destroyed(&mut app, vb, Vec2::new(-50.0, 0.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(n).unwrap();
    assert!(
        (hp.current - 6.0).abs() < f32::EPSILON,
        "shared-neighbour heals clamp at starting = 6.0, got {}",
        hp.current
    );
}

// Behavior 28: cell outside radius — Hp unchanged end-to-end.
#[test]
fn pipeline_cell_outside_radius_unchanged() {
    let mut app = test_app_pipeline();
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let far = spawn_cell_at(&mut app, Vec2::new(200.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(far).unwrap();
    assert!((hp.current - 5.0).abs() < f32::EPSILON);
}

// Behavior 29: dead cell is not self-healed end-to-end.
#[test]
fn pipeline_dead_cell_is_not_self_healed() {
    let mut app = test_app_pipeline();
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let dead_cell = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    send_cell_destroyed(&mut app, dead_cell, Vec2::ZERO);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(dead_cell).unwrap();
    assert!(
        hp.current.abs() < f32::EPSILON,
        "dead cell must not self-heal end-to-end, got {}",
        hp.current
    );
}
