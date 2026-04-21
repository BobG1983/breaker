//! Multiple-incoming-message behaviour: independent clusters, accumulation on
//! the same primary, and killing-blow dealer attribution.

use bevy::prelude::*;

use super::helpers::*;
use crate::prelude::*;

// ════════════════════════════════════════════════════════════════════════
// Behavior 39 — Multiple incoming messages each redistribute independently
// ════════════════════════════════════════════════════════════════════════

#[test]
fn two_non_overlapping_clusters_each_redistribute_independently() {
    let mut app = build_apply_damage_to_cells_app();
    let p1 = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    let n1 = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    let p2 = spawn_cell_at(&mut app, Vec2::new(500.0, 0.0), 100.0);
    let n2 = spawn_cell_at(&mut app, Vec2::new(550.0, 0.0), 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(p1, 50.0, None));
    push_damage(&mut app, damage_msg(p2, 50.0, None));
    tick(&mut app);

    assert!((hp_of(&app, p1) - 60.0).abs() < f32::EPSILON);
    assert!((hp_of(&app, n1) - 90.0).abs() < f32::EPSILON);
    assert!((hp_of(&app, p2) - 60.0).abs() < f32::EPSILON);
    assert!((hp_of(&app, n2) - 90.0).abs() < f32::EPSILON);
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 40 — Same-primary multi-message accumulates damage correctly
// ════════════════════════════════════════════════════════════════════════

#[test]
fn two_messages_to_same_primary_each_redistribute_and_accumulate() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 100.0);
    let neighbor = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(primary, 50.0, None));
    push_damage(&mut app, damage_msg(primary, 50.0, None));
    tick(&mut app);

    // Each message: primary takes 40.0, neighbor takes 10.0.
    // Totals: primary = 100 - 80 = 20; neighbor = 100 - 20 = 80.
    assert!(
        (hp_of(&app, primary) - 20.0).abs() < f32::EPSILON,
        "primary HP expected 20.0 (took 40 + 40), got {}",
        hp_of(&app, primary)
    );
    assert!(
        (hp_of(&app, neighbor) - 80.0).abs() < f32::EPSILON,
        "neighbor HP expected 80.0 (took 10 + 10), got {}",
        hp_of(&app, neighbor)
    );

    // KilledBy.dealer stays None because neither message carries a dealer.
    let killed_by = app.world().get::<KilledBy>(primary).unwrap();
    assert_eq!(killed_by.dealer, None);
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 40b — Killing-blow attribution follows message order, not arrival
// ════════════════════════════════════════════════════════════════════════

#[test]
fn multi_dealer_killing_blow_attributes_to_order_resolved_dealer() {
    // Two messages hit the same isolated primary in one tick. The first
    // message carries no dealer; the second carries a dealer and pushes
    // running HP past zero. Attribution must follow the order the primary's
    // running HP crosses zero — i.e., the second message's dealer — rather
    // than first-write-wins semantics.
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 15.0);
    // No neighbors → isolated-primary branch → each message delivers full
    // `amount` to primary (pass-through regardless of diffusion).
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    let dealer = app.world_mut().spawn_empty().id();

    // Msg 1: no dealer, drops HP 15 → 5 (still positive).
    push_damage(&mut app, damage_msg(primary, 10.0, None));
    // Msg 2: carries dealer, pushes HP 5 → -15 (killing blow).
    push_damage(&mut app, damage_msg(primary, 20.0, Some(dealer)));
    tick(&mut app);

    assert!(hp_of(&app, primary) <= 0.0, "primary should be killed");
    let killed_by = app.world().get::<KilledBy>(primary).unwrap();
    assert_eq!(
        killed_by.dealer,
        Some(dealer),
        "killing-blow dealer must be the second message's dealer"
    );
}

#[test]
fn multi_dealer_first_message_kills_attributes_to_first() {
    // First message alone kills the primary. Subsequent messages should not
    // overwrite attribution.
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 15.0);
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    let dealer_a = app.world_mut().spawn_empty().id();
    let dealer_b = app.world_mut().spawn_empty().id();

    // Msg 1: dealer_a, 20 dmg → kills (15 → -5).
    push_damage(&mut app, damage_msg(primary, 20.0, Some(dealer_a)));
    // Msg 2: dealer_b, 5 dmg → piles on after kill.
    push_damage(&mut app, damage_msg(primary, 5.0, Some(dealer_b)));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(primary).unwrap();
    assert_eq!(
        killed_by.dealer,
        Some(dealer_a),
        "killing-blow dealer must be the first message's dealer"
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 44 — KilledBy.dealer is set on reduced-damage killing blow
// ════════════════════════════════════════════════════════════════════════

#[test]
fn killed_by_dealer_set_on_reduced_damage_killing_blow() {
    let mut app = build_apply_damage_to_cells_app();
    let primary = spawn_cell_at(&mut app, Vec2::ZERO, 40.0);
    let neighbor = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 100.0);
    let dealer = app.world_mut().spawn_empty().id();
    install_diffusion_config(&mut app, canonical_diffusion_config());
    add_diffusion_stacks(&mut app, 1);

    push_damage(&mut app, damage_msg(primary, 50.0, Some(dealer)));
    tick(&mut app);

    // Primary took 40.0 reduced damage (50 * 0.80), HP 40 - 40 = 0.0 — killed.
    assert!(
        (hp_of(&app, primary) - 0.0).abs() < f32::EPSILON,
        "primary HP expected 0.0 (killing blow at reduced damage), got {}",
        hp_of(&app, primary)
    );
    let killed_by = app.world().get::<KilledBy>(primary).unwrap();
    assert_eq!(
        killed_by.dealer,
        Some(dealer),
        "KilledBy.dealer must be the original dealer, got {:?}",
        killed_by.dealer
    );
    // Neighbor still receives share: 50 * 0.20 / 1 = 10 → HP 90.
    assert!(
        (hp_of(&app, neighbor) - 90.0).abs() < f32::EPSILON,
        "neighbor HP expected 90.0, got {}",
        hp_of(&app, neighbor)
    );
}
