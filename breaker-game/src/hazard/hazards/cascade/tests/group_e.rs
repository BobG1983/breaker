use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::{
    hazard::definition::HazardKind,
    prelude::{HealCap, SourceId, SourceIdExt},
};

// ════════════════════════════════════════════════════════════════════════════
// Group E — Multiple deaths / multiple neighbours
// ════════════════════════════════════════════════════════════════════════════

// Behavior 16: multiple neighbours within radius → one message per neighbour.
#[test]
fn multiple_neighbours_each_receive_one_message() {
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
    let n1 = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    let n2 = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 5.0, 10.0);
    let n3 = spawn_cell_at(&mut app, Vec2::new(0.0, 50.0), 5.0, 10.0);
    let n4 = spawn_cell_at(&mut app, Vec2::new(0.0, -50.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 4);
    for n in [n1, n2, n3, n4] {
        let msgs = heals_for_cell(&app, n);
        assert_eq!(msgs.len(), 1, "each neighbour gets exactly one message");
        assert!((msgs[0].amount - 1.0).abs() < f32::EPSILON);
        assert!(matches!(msgs[0].cap, HealCap::Starting));
        assert_eq!(
            msgs[0].source,
            Some(SourceId::hazard(HazardKind::Cascade).build())
        );
        assert_eq!(msgs[0].healer, None);
    }
}

// Behavior 16 edge: adding a fifth outside-radius cell — still exactly 4 messages.
#[test]
fn multiple_neighbours_with_one_outside_radius_still_four_messages() {
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
    let far = spawn_cell_at(&mut app, Vec2::new(200.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 4);
    assert!(heals_for_cell(&app, far).is_empty());
}

// Behavior 17: multiple deaths, shared neighbour → N separate messages.
#[test]
fn multiple_deaths_shared_neighbour_emit_two_messages() {
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
    let victim_a = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 0.0, 10.0);
    let victim_b = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 0.0, 10.0);
    send_cell_destroyed(&mut app, victim_a, Vec2::new(50.0, 0.0));
    send_cell_destroyed(&mut app, victim_b, Vec2::new(-50.0, 0.0));

    run_fixed_update(&mut app);

    let msgs = heals_for_cell(&app, n);
    assert_eq!(
        msgs.len(),
        2,
        "two deaths adjacent to one neighbour → two separate HealDealt<Cell> messages"
    );
    for msg in &msgs {
        assert!((msg.amount - 1.0).abs() < f32::EPSILON);
        assert!(matches!(msg.cap, HealCap::Starting));
        assert_eq!(
            msg.source,
            Some(SourceId::hazard(HazardKind::Cascade).build())
        );
    }
}

// Behavior 17 edge: three shared-neighbour deaths → three messages.
#[test]
fn three_deaths_shared_neighbour_emit_three_messages() {
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
    let vc = spawn_cell_at(&mut app, Vec2::new(0.0, 50.0), 0.0, 10.0);
    send_cell_destroyed(&mut app, va, Vec2::new(50.0, 0.0));
    send_cell_destroyed(&mut app, vb, Vec2::new(-50.0, 0.0));
    send_cell_destroyed(&mut app, vc, Vec2::new(0.0, 50.0));

    run_fixed_update(&mut app);

    let msgs = heals_for_cell(&app, n);
    assert_eq!(msgs.len(), 3);
}

// Behavior 18: multiple deaths with disjoint neighbours → union of per-death
// messages.
#[test]
fn disjoint_deaths_produce_one_message_each() {
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

    let a = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    let b = spawn_cell_at(&mut app, Vec2::new(-200.0, 0.0), 5.0, 10.0);
    let va = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0); // adjacent to a
    let vb = spawn_cell_at(&mut app, Vec2::new(-250.0, 0.0), 0.0, 10.0); // adjacent to b
    send_cell_destroyed(&mut app, va, Vec2::new(0.0, 0.0));
    send_cell_destroyed(&mut app, vb, Vec2::new(-250.0, 0.0));

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 2);
    assert_eq!(heals_for_cell(&app, a).len(), 1);
    assert_eq!(heals_for_cell(&app, b).len(), 1);
}

// Behavior 19: mixed case — one death, one neighbour in + one out.
#[test]
fn single_death_mixed_in_out_radius_emits_one_message() {
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
    let in_r = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    let out_r = spawn_cell_at(&mut app, Vec2::new(200.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 1);
    assert_eq!(heals_for_cell(&app, in_r).len(), 1);
    assert!(heals_for_cell(&app, out_r).is_empty());
}
