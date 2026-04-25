use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::{cells::components::Cell, mutators::hazards::definition::HazardKind, prelude::*};

// ════════════════════════════════════════════════════════════════════════════
// Group F — Message-field invariants
// ════════════════════════════════════════════════════════════════════════════

// Behavior 20: every emitted message has cap == HealCap::Starting.
#[test]
fn every_message_cap_is_starting() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let _n1 = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    let _n2 = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 5.0, 10.0);
    let _n3 = spawn_cell_at(&mut app, Vec2::new(0.0, 50.0), 5.0, 10.0);
    let _n4 = spawn_cell_at(&mut app, Vec2::new(0.0, -50.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let all = &app
        .world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0;
    assert_eq!(all.len(), 4);
    assert!(all.iter().all(|m| matches!(m.cap, HealCap::Starting)));
}

// Behavior 20 edge: cap is HealCap::Starting even at stack 5 with elevated
// amounts.
#[test]
fn cap_is_starting_even_at_stack_five() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 5.0,
        },
    );
    add_cascade_stacks(&mut app, 5);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let n = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 100.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let msgs = heals_for_cell(&app, n);
    assert_eq!(msgs.len(), 1);
    assert!((msgs[0].amount - 30.0).abs() < f32::EPSILON);
    assert!(matches!(msgs[0].cap, HealCap::Starting));
}

// Behavior 21: every emitted message has source == Some("hazard:cascade").
#[test]
fn every_message_source_is_hazard_cascade() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
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

    let all = &app
        .world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0;
    assert_eq!(all.len(), 2);
    assert!(
        all.iter()
            .all(|m| m.source == Some(SourceId::hazard(HazardKind::Cascade).build()))
    );
    // Bind n so compiler doesn't warn on unused:
    let _ = n;
}

// Behavior 22: every emitted message has healer == None.
#[test]
fn every_message_healer_is_none() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let _n1 = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    let _n2 = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 5.0, 10.0);
    let _n3 = spawn_cell_at(&mut app, Vec2::new(0.0, 50.0), 5.0, 10.0);
    let _n4 = spawn_cell_at(&mut app, Vec2::new(0.0, -50.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let all = &app
        .world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0;
    assert_eq!(all.len(), 4);
    assert!(all.iter().all(|m| m.healer.is_none()));
}

// Behavior 23: every message's target is a living cell — never victim, never
// corpse.
#[test]
fn every_target_is_a_live_cell_never_victim_or_corpse() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        },
    );
    add_cascade_stacks(&mut app, 1);

    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    let corpse = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 0.0, 10.0);
    let v = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    send_cell_destroyed(&mut app, v, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 1);
    let msgs = heals_for_cell(&app, neighbour);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].target, neighbour);
    assert!(heals_for_cell(&app, v).is_empty());
    assert!(heals_for_cell(&app, corpse).is_empty());
}
