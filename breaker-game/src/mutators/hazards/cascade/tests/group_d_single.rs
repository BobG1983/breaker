use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::{
    mutators::hazards::definition::HazardKind,
    prelude::{HealCap, SourceId, SourceIdExt},
};

// ════════════════════════════════════════════════════════════════════════════
// Group D — Single-death / single-neighbour message shape
// ════════════════════════════════════════════════════════════════════════════

// Behavior 11: single death + single neighbour → one HealDealt<Cell> with
// correct fields.
#[test]
fn single_death_single_neighbour_emits_one_message_with_correct_fields() {
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
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 1);
    let msgs = heals_for_cell(&app, neighbour);
    assert_eq!(msgs.len(), 1);
    let msg = &msgs[0];
    assert_eq!(msg.target, neighbour);
    assert!(
        (msg.amount - 1.0).abs() < f32::EPSILON,
        "amount should equal heal_per_neighbour(1) = 1.0, got {}",
        msg.amount
    );
    assert!(matches!(msg.cap, HealCap::Starting));
    assert_eq!(msg.healer, None);
    assert_eq!(
        msg.source,
        Some(SourceId::hazard(HazardKind::Cascade).build())
    );
}

// Behavior 11 edge: neighbour at exactly the radius boundary (distance² == 4900)
// is included.
#[test]
fn neighbour_at_exact_radius_boundary_is_included() {
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
    // At distance exactly 70.0 — distance² == ADJACENCY_RADIUS_SQ (4900).
    let neighbour = spawn_cell_at(&mut app, Vec2::new(70.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 1);
    let msgs = heals_for_cell(&app, neighbour);
    assert_eq!(msgs.len(), 1);
    assert!((msgs[0].amount - 1.0).abs() < f32::EPSILON);
    assert!(matches!(msgs[0].cap, HealCap::Starting));
}

// Behavior 12: stack-3 → amount is base + 2 * per_level.
#[test]
fn stack_three_message_amount_is_base_plus_two_per_level() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, cascade_heal_on_death);
    install_cascade_config(
        &mut app,
        CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 5.0,
        },
    );
    add_cascade_stacks(&mut app, 3);

    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 100.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 1);
    let msgs = heals_for_cell(&app, neighbour);
    assert_eq!(msgs.len(), 1);
    assert!(
        (msgs[0].amount - 20.0).abs() < f32::EPSILON,
        "amount should be 10.0 + 5.0*2 = 20.0, got {}",
        msgs[0].amount
    );
    assert!(matches!(msgs[0].cap, HealCap::Starting));
    assert_eq!(
        msgs[0].source,
        Some(SourceId::hazard(HazardKind::Cascade).build())
    );
}

// Behavior 12 edge: stack 5 produces amount == 30.0 (design-doc pinned).
#[test]
fn stack_five_message_amount_is_thirty() {
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
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 100.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let msgs = heals_for_cell(&app, neighbour);
    assert_eq!(msgs.len(), 1);
    assert!(
        (msgs[0].amount - 30.0).abs() < f32::EPSILON,
        "amount should be 30.0 at stack 5, got {}",
        msgs[0].amount
    );
}

// Behavior 13: neighbour outside radius → zero messages.
#[test]
fn neighbour_outside_radius_is_not_emitted() {
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
    // distance² = 40000 > 4900
    let _far = spawn_cell_at(&mut app, Vec2::new(200.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 0);
}

// Behavior 13 edge: just-outside radius — (71.0, 0.0), distance² = 5041 > 4900.
#[test]
fn neighbour_just_outside_radius_is_not_emitted() {
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
    // distance² = 71*71 = 5041 > 4900
    let _just_out = spawn_cell_at(&mut app, Vec2::new(71.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 0);
}

// Behavior 14: the Destroyed<Cell>.victim entity is never a heal target.
// Corrected per minor-spec-fix 2: set victim's Hp.current = 5.0 (>0.0) so
// this test pins the `*victim != entity` guard, not the `hp.current <= 0.0`
// guard.
#[test]
fn destroyed_victim_entity_is_never_self_healed() {
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

    // Victim Hp.current = 5.0 (> 0.0) so the only guard that can exclude
    // this entity is the `*victim != entity` self-exclusion check.
    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(
        cascade_heal_collector_len(&app),
        0,
        "victim entity must never receive a self-heal message"
    );
    let self_msgs = heals_for_cell(&app, victim);
    assert!(self_msgs.is_empty());
}

// Behavior 14 edge: victim self-excluded, but adjacent neighbour still healed.
#[test]
fn self_exclusion_does_not_block_adjacent_neighbour_heal() {
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
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(cascade_heal_collector_len(&app), 1);
    let msgs = heals_for_cell(&app, neighbour);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].target, neighbour);
    // victim got none:
    assert!(heals_for_cell(&app, victim).is_empty());
}

// Behavior 15: a separately-dead cell (Hp.current <= 0.0) is not a heal
// target.
#[test]
fn separately_dead_cell_is_not_a_heal_target() {
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
    // Separate victim entity at origin (its position is what triggers
    // adjacency); its Hp doesn't matter because the victim is referenced by
    // entity id in the message.
    let victim = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    assert_eq!(
        cascade_heal_collector_len(&app),
        1,
        "exactly one message — for the living neighbour, none for the corpse"
    );
    let neighbour_msgs = heals_for_cell(&app, neighbour);
    assert_eq!(neighbour_msgs.len(), 1);
    assert_eq!(neighbour_msgs[0].target, neighbour);
    let corpse_msgs = heals_for_cell(&app, corpse);
    assert!(
        corpse_msgs.is_empty(),
        "corpse with Hp.current = 0.0 must not receive a message"
    );
}

// ── B36c: source matches builder-produced hazard:cascade ──

#[test]
fn cascade_heal_source_equals_builder() {
    use crate::{mutators::hazards::definition::HazardKind, prelude::SourceIdExt};
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
    let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
    send_cell_destroyed(&mut app, victim, Vec2::ZERO);

    run_fixed_update(&mut app);

    let msgs = heals_for_cell(&app, neighbour);
    assert!(!msgs.is_empty());
    let expected = SourceId::hazard(HazardKind::Cascade).build();
    for m in &msgs {
        assert_eq!(m.source.as_ref(), Some(&expected));
    }
}
