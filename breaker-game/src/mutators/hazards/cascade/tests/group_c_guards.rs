use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::{
    cells::components::Cell,
    mutators::hazards::{definition::HazardKind, resources::ActiveHazards},
    prelude::*,
};

// ════════════════════════════════════════════════════════════════════════════
// Group C — No-op guards
// ════════════════════════════════════════════════════════════════════════════

// Behavior 6: no message emitted when CascadeConfig resource is absent.
#[test]
fn no_message_when_config_absent() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 5.0, 10.0);
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(
        cascade_heal_collector_len(&app),
        0,
        "no HealDealt<Cell> should be emitted when CascadeConfig is absent"
    );
    // Neighbour's Hp must not have been mutated either — cascade_heal_on_death
    // has no business touching Hp, ever.
    let hp = app.world().get::<Hp>(neighbour).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "neighbour Hp must remain 5.0 when no config present, got {}",
        hp.current
    );
}

// Behavior 7: no message emitted when ActiveHazards.stacks(Cascade) == 0.
#[test]
fn no_message_when_zero_cascade_stacks() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 5.0,
        },
    );
    // No stacks added — heal_per_neighbour(0) == 0.0 triggers the early return.

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let _neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(
        cascade_heal_collector_len(&app),
        0,
        "no HealDealt<Cell> should be emitted when Cascade has zero stacks"
    );
}

// Behavior 7 edge: zero stacks + zero config → still no messages.
#[test]
fn no_message_when_zero_stacks_and_zero_config() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      0.0,
            per_level_heal: 0.0,
        },
    );

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let _neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 0);
}

// Behavior 8: no message emitted when there are no Destroyed<Cell> messages.
#[test]
fn no_message_when_no_destroyed_messages() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 5.0,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let _a = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 5.0, 10.0);
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    // No send_cell_destroyed call.

    run_fixed_update(&mut app);

    assert_eq!(
        cascade_heal_collector_len(&app),
        0,
        "no HealDealt<Cell> without any Destroyed<Cell> this tick"
    );
    let hp = app.world().get::<Hp>(neighbour).unwrap();
    assert!((hp.current - 5.0).abs() < f32::EPSILON);
}

// Behavior 9: run-condition gate — system skipped when Cascade inactive.
#[test]
fn system_skipped_when_cascade_run_condition_false() {
    let mut app = test_app_playing();
    // Use register(app) so the hazard_active + in_state run-conditions apply.
    register(&mut app);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 5.0,
        },
    );
    // Stack a different hazard (Volatility) — Cascade remains inactive.
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Volatility);
    assert_eq!(
        app.world()
            .resource::<ActiveHazards>()
            .stacks(HazardKind::Cascade),
        0
    );

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let _neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(
        cascade_heal_collector_len(&app),
        0,
        "hazard_active(Cascade) run-condition must gate off the system"
    );
}

// Behavior 10: run-condition gate — system skipped when NOT in NodeState::Playing.
#[test]
fn system_skipped_when_not_in_node_playing() {
    // Build app with state hierarchy but DO NOT drive into NodeState::Playing.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveHazards>()
        .with_message::<Destroyed<Cell>>()
        .with_message_capture::<HealDealt<Cell>>()
        .build();
    register(&mut app);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 5.0,
        },
    );
    add_cascade_stacks(&mut app, 3);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let _neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(
        cascade_heal_collector_len(&app),
        0,
        "in_state(NodeState::Playing) run-condition must gate off the system"
    );
}
