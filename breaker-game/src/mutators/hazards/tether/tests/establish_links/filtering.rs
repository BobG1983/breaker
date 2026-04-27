//! Behaviors 32–35 + W7 — eligibility filters (zero pairs, empty grid,
//! Dead/Invulnerable handling, mutual exclusion through a center cell).

use bevy::prelude::*;

use super::{
    super::helpers::{
        add_tether_stacks, build_establish_tether_app, canonical_tether_config, enter_playing,
        install_tether_config, spawn_cell_at, spawn_cell_dead_at, spawn_cell_invulnerable_at,
    },
    link_helpers::{count_linked_pairs, links_map},
};

// ════════════════════════════════════════════════════════════════════════════
// Behavior 32 — Zero eligible pairs → zero links
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn zero_eligible_pairs_yields_zero_links() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);
    // All cells far apart — no adjacent pairs.
    spawn_cell_at(&mut app, Vec2::new(0.0, 0.0));
    spawn_cell_at(&mut app, Vec2::new(1000.0, 0.0));
    spawn_cell_at(&mut app, Vec2::new(2000.0, 0.0));
    enter_playing(&mut app);

    assert_eq!(count_linked_pairs(&mut app), 0);
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 33 — Empty cell grid does not panic
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn empty_cell_grid_does_not_panic() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);
    // No cells spawned.
    enter_playing(&mut app);

    assert_eq!(count_linked_pairs(&mut app), 0);
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 34 — Dead cells are filtered out of eligibility (invulnerable
// cells ARE eligible post-W7 — see the separate W7 tests below)
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn dead_cells_are_filtered_out_of_eligibility() {
    // Post-W7: `LiveCellPositions` retains `Without<Dead>` but drops
    // `Without<Invulnerable>`. This test keeps the 3-cell (0,50,100) layout
    // and uses the MIDDLE cell as DEAD (not invulnerable) to pin the
    // Dead-filter branch. The outer survivors are d² = 10000 > 4900 apart
    // → no eligible pair when the bridge is filtered out.
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 7); // 100% coverage to force selection attempt.
    let _a = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0));
    let _b = spawn_cell_dead_at(&mut app, Vec2::new(50.0, 0.0));
    let _c = spawn_cell_at(&mut app, Vec2::new(100.0, 0.0));
    enter_playing(&mut app);

    assert_eq!(
        count_linked_pairs(&mut app),
        0,
        "Dead cell invisible to Tether; outer survivors too far apart"
    );
}

#[test]
fn dead_marked_cells_are_filtered_out_of_eligibility() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 7);
    // (0,0), (50,0), (100,0); middle is Dead-marked. Remaining d² = 10000 > 4900.
    let _a = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0));
    let _b = spawn_cell_dead_at(&mut app, Vec2::new(50.0, 0.0));
    let _c = spawn_cell_at(&mut app, Vec2::new(100.0, 0.0));
    enter_playing(&mut app);

    assert_eq!(count_linked_pairs(&mut app), 0);
}

// ════════════════════════════════════════════════════════════════════════════
// W7 — Invulnerable cells ARE eligible tether partners post-filter-drop.
// Pipeline's `invulnerable_filter::<Cell>` neutralizes inert pairs downstream.
//
// Behavior 4 from `.claude/specs/w7-drop-invulnerable-filter-tests.md`.
// ════════════════════════════════════════════════════════════════════════════

// ── W7 Behavior 4 — one invulnerable + one vulnerable cell form an eligible pair.

#[test]
fn invulnerable_cell_can_be_selected_as_tether_partner() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 7); // 100% coverage.
    let a = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0));
    let b = spawn_cell_invulnerable_at(&mut app, Vec2::new(50.0, 0.0));
    enter_playing(&mut app);

    // distance² = 2500 ≤ 4900 → eligible pair; selector must pair them.
    assert_eq!(
        count_linked_pairs(&mut app),
        1,
        "invulnerable cell must be eligible as a tether partner under W7"
    );

    // Assert mutual linkage between `a` and `b`.
    let map = links_map(&mut app);
    assert_eq!(
        map.get(&a).copied(),
        Some(b),
        "vulnerable `a` must point at invulnerable `b`"
    );
    assert_eq!(
        map.get(&b).copied(),
        Some(a),
        "invulnerable `b` must point back at vulnerable `a`"
    );
}

// ── W7 Behavior 4 edge case — swap: `a` invulnerable, `b` vulnerable. Selector
//    is symmetric under invulnerability.

#[test]
fn invulnerable_cell_can_be_selected_as_tether_partner_symmetric() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 7);
    let a = spawn_cell_invulnerable_at(&mut app, Vec2::new(0.0, 0.0));
    let b = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0));
    enter_playing(&mut app);

    assert_eq!(
        count_linked_pairs(&mut app),
        1,
        "selector must be symmetric — invulnerable cell on EITHER endpoint is fine"
    );
    let map = links_map(&mut app);
    assert_eq!(map.get(&a).copied(), Some(b));
    assert_eq!(map.get(&b).copied(), Some(a));
}

// ── W7 Behavior 4 edge case — BOTH cells invulnerable. Inert pair acceptable
//    per user approval; ripple is zeroed by the pipeline.

#[test]
fn both_invulnerable_cells_may_form_an_inert_tether_pair() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 7);
    let a = spawn_cell_invulnerable_at(&mut app, Vec2::ZERO);
    let b = spawn_cell_invulnerable_at(&mut app, Vec2::new(50.0, 0.0));
    enter_playing(&mut app);

    assert_eq!(
        count_linked_pairs(&mut app),
        1,
        "invulnerable↔invulnerable inert pair is acceptable under W7"
    );
    let map = links_map(&mut app);
    assert_eq!(map.get(&a).copied(), Some(b));
    assert_eq!(map.get(&b).copied(), Some(a));
}

// ── W7 Behavior 4 edge case — one Invulnerable, one Dead. Dead filter still
//    applies; no eligible pair → zero links.

#[test]
fn invulnerable_plus_dead_yields_no_pair() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 7);
    let _a = spawn_cell_invulnerable_at(&mut app, Vec2::ZERO);
    let _b = spawn_cell_dead_at(&mut app, Vec2::new(50.0, 0.0));
    enter_playing(&mut app);

    assert_eq!(
        count_linked_pairs(&mut app),
        0,
        "Dead filter still applies — no eligible pair when the only partner is Dead"
    );
}

// ── W7 — 3-cell mutual-exclusion-through-invulnerable-bridge layout (keeps the
//    prior 3-cell geometry of `dead_and_invulnerable_cells_are_filtered_out`,
//    but with the middle cell INVULNERABLE). Under W7 the middle cell is
//    eligible, giving two bridge pairs (0↔50) and (50↔100) through the same
//    middle cell; mutual-exclusion caps the match at 1.

#[test]
fn invulnerable_middle_cell_serves_as_eligible_bridge_partner() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 7);
    // cells at (0,0), (50,0), (100,0); middle is Invulnerable (not Dead).
    // pairs: (0↔50) d²=2500 ✓, (50↔100) d²=2500 ✓, (0↔100) d²=10000 ✗.
    // Two eligible pairs through the middle; greedy mutual-exclusion picks ONE.
    let _a = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0));
    let _b = spawn_cell_invulnerable_at(&mut app, Vec2::new(50.0, 0.0));
    let _c = spawn_cell_at(&mut app, Vec2::new(100.0, 0.0));
    enter_playing(&mut app);

    assert_eq!(
        count_linked_pairs(&mut app),
        1,
        "invulnerable middle cell is an eligible bridge; mutual exclusion caps at 1"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 35 — Mutual exclusion holds when a central cell is in many pairs
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn mutual_exclusion_caps_center_cell_participation_at_one_pair() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 7); // 100% coverage.
    // Cross shape: (0,0), (50,0), (-50,0), (0,50), (0,-50) — 4 pairs through center.
    let _center = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0));
    let _right = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0));
    let _left = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0));
    let _up = spawn_cell_at(&mut app, Vec2::new(0.0, 50.0));
    let _down = spawn_cell_at(&mut app, Vec2::new(0.0, -50.0));
    enter_playing(&mut app);

    assert_eq!(
        count_linked_pairs(&mut app),
        1,
        "center cell can participate in at most one pair; other outer cells \
         are outside adjacency radius from each other (d² ≥ 5000)"
    );
}
