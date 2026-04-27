//! Behaviors 25–26 — adjacency-radius inclusivity / exclusivity.

use bevy::prelude::*;

use super::{
    super::helpers::{
        add_tether_stacks, build_establish_tether_app, canonical_tether_config, enter_playing,
        install_tether_config, spawn_cell_at,
    },
    link_helpers::count_linked_pairs,
};

// ════════════════════════════════════════════════════════════════════════════
// Behavior 25 — Diagonals (distance² > ADJACENCY_RADIUS_SQ) are excluded
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn diagonal_pairs_outside_radius_are_excluded() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);
    // distance² = 50² + 50² = 5000 > 4900.
    let _a = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0));
    let _b = spawn_cell_at(&mut app, Vec2::new(50.0, 50.0));
    enter_playing(&mut app);

    assert_eq!(
        count_linked_pairs(&mut app),
        0,
        "pair outside adjacency radius must not be selected"
    );
}

#[test]
fn inside_radius_but_just_inside_is_eligible() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 7); // 100% coverage to force selection.
    // distance² = 50² + 48² = 2500 + 2304 = 4804 ≤ 4900.
    let _a = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0));
    let _b = spawn_cell_at(&mut app, Vec2::new(50.0, 48.0));
    enter_playing(&mut app);

    assert_eq!(
        count_linked_pairs(&mut app),
        1,
        "pair just inside radius must be selected at 100% coverage"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 26 — Adjacency boundary: exactly at ADJACENCY_RADIUS_SQ is included
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn adjacency_boundary_inclusive_at_exact_radius() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 7);
    // Three cells in a row at (0,0), (70,0), (140,0) — consecutive d² = 4900.
    // Two eligible pairs (0↔1, 1↔2); cell 1 is in both → matching limit 1.
    let _a = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0));
    let _b = spawn_cell_at(&mut app, Vec2::new(70.0, 0.0));
    let _c = spawn_cell_at(&mut app, Vec2::new(140.0, 0.0));
    enter_playing(&mut app);

    assert_eq!(
        count_linked_pairs(&mut app),
        1,
        "at d² = 4900 exactly, both pairs are eligible but mutual exclusion \
         caps matching at 1"
    );
}

#[test]
fn adjacency_boundary_exclusive_above_radius() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 7);
    // (0,0), (71,0), (140,0): d²(0↔1) = 5041 (NOT eligible); d²(1↔2) = 69² = 4761 (eligible).
    let _a = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0));
    let _b = spawn_cell_at(&mut app, Vec2::new(71.0, 0.0));
    let _c = spawn_cell_at(&mut app, Vec2::new(140.0, 0.0));
    enter_playing(&mut app);

    // Only pair 1↔2 eligible → 1 pair selected.
    assert_eq!(count_linked_pairs(&mut app), 1);
}
