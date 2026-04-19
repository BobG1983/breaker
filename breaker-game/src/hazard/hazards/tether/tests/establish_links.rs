//! Section C — `establish_tether_links` system (Behaviours 20–35).
//!
//! Fires `OnEnter(NodeState::Playing)` behind `hazard_active(Tether)`.
//! Pins mutual-exclusion matching, adjacency-radius inclusivity, determinism
//! under shared `GameRng`, harness-safety when resources are absent, and
//! run-condition / filter behavior.

use std::collections::HashMap;

use bevy::prelude::*;

use super::{
    super::system::TetherLink,
    helpers::{
        add_hazard_stacks, add_tether_stacks, build_establish_tether_app,
        build_establish_tether_app_no_rng, canonical_tether_config, enter_playing,
        install_tether_config, link_partners, spawn_cell_at, spawn_cell_dead_at,
        spawn_cell_invulnerable_at, spawn_cell_row,
    },
};
use crate::hazard::definition::HazardKind;

// ── Link-count helpers (local to establish tests) ────────────────────────────

/// Counts cells carrying `TetherLink`, divided by 2.
fn count_linked_pairs(app: &mut App) -> usize {
    let mut q = app.world_mut().query_filtered::<Entity, With<TetherLink>>();
    let total = q.iter(app.world()).count();
    total / 2
}

/// Builds a map from entity → `TetherLink.partner`. Used to check mutual
/// bidirectional link integrity.
fn links_map(app: &mut App) -> HashMap<Entity, Entity> {
    let mut q = app.world_mut().query::<(Entity, &TetherLink)>();
    q.iter(app.world())
        .map(|(e, link)| (e, link.partner))
        .collect()
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 20 — Stack 1: 40% of 9 eligible pairs → 4 pairs linked
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn stack_one_links_four_of_nine_eligible_pairs() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);
    let _cells = spawn_cell_row(&mut app, 10, 50.0);
    enter_playing(&mut app);

    // round(9 * 0.40) = round(3.6) = 4, bounded by matching limit (5).
    assert_eq!(
        count_linked_pairs(&mut app),
        4,
        "stack 1 on 10-in-a-row should link exactly 4 pairs (round(3.6))"
    );

    // Every TetherLink must be mutual: A.partner = B and B.partner = A.
    let map = links_map(&mut app);
    for (e, partner) in &map {
        let back = map.get(partner).copied();
        assert_eq!(
            back,
            Some(*e),
            "TetherLink must be mutual — entity {e:?} points at {partner:?} but reverse is {back:?}"
        );
    }

    // Mutual exclusion: no cell appears in more than one pair.
    let mut appearances: HashMap<Entity, usize> = HashMap::new();
    for (e, partner) in &map {
        *appearances.entry(*e).or_insert(0) += 1;
        *appearances.entry(*partner).or_insert(0) += 1;
    }
    for (e, n) in &appearances {
        assert!(
            *n == 2,
            "cell {e:?} appears {n} times across links — mutual exclusion broken (expected 2: once as self, once as partner)"
        );
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 21 — Stack 3: 60% of 9 → 5 pairs requested, matching limit 5 → 5
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn stack_three_links_four_or_five_pairs() {
    // target_count = round(9 * 0.60) = 5, but greedy mutual-exclusion walk on
    // a 9-edge path graph yields a *maximal* (not maximum) matching — actual
    // count is 4 or 5 depending on shuffle order.
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 3);
    let _cells = spawn_cell_row(&mut app, 10, 50.0);
    enter_playing(&mut app);

    let count = count_linked_pairs(&mut app);
    assert!(
        (4..=5).contains(&count),
        "expected 4..=5 pairs (greedy matching on path graph), got {count}"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 22 — Stack 5: 80% of 9 → 7 requested, matching limit 5 → 5
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn stack_five_hits_matching_ceiling() {
    // target_count = round(9 * 0.80) = 7, but max matching on a 9-edge path
    // graph is 5; greedy walk yields 4 or 5 depending on shuffle.
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 5);
    let _cells = spawn_cell_row(&mut app, 10, 50.0);
    enter_playing(&mut app);

    let count = count_linked_pairs(&mut app);
    assert!(
        (4..=5).contains(&count),
        "target_count (7) exceeds matching limit; walk yields 4..=5, got {count}"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 23 — Stack 7: 100% coverage, matching limit 5 → 5
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn stack_seven_at_cap_hits_matching_ceiling() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 7);
    let _cells = spawn_cell_row(&mut app, 10, 50.0);
    enter_playing(&mut app);

    // coverage_percent(7) == 100.0 inclusive-boundary → target_count == 9,
    // but greedy mutual-exclusion walk on a 9-edge path graph caps at a
    // maximal matching of 4-5 pairs (not a maximum 5-pair matching).
    let count = count_linked_pairs(&mut app);
    assert!(
        (4..=5).contains(&count),
        "walk must stop at 4..=5 pairs on 9-edge path graph, got {count}"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 24 — Links are bidirectional — both ends hold `TetherLink`
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn links_are_bidirectional() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);
    let _cells = spawn_cell_row(&mut app, 10, 50.0);
    enter_playing(&mut app);

    let map = links_map(&mut app);
    assert!(!map.is_empty(), "expected at least one TetherLink");
    for (a, b) in &map {
        let back = map.get(b).copied();
        assert_eq!(
            back,
            Some(*a),
            "TetherLink on {a:?} points to {b:?} but reverse points to {back:?}"
        );
    }
}

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

// ════════════════════════════════════════════════════════════════════════════
// Behavior 27 — Determinism: same seed → same pair selection
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn same_seed_produces_same_pair_selection() {
    let mut app_a = build_establish_tether_app(42);
    install_tether_config(&mut app_a, canonical_tether_config());
    add_tether_stacks(&mut app_a, 1);
    spawn_cell_row(&mut app_a, 10, 50.0);
    enter_playing(&mut app_a);

    let mut app_b = build_establish_tether_app(42);
    install_tether_config(&mut app_b, canonical_tether_config());
    add_tether_stacks(&mut app_b, 1);
    spawn_cell_row(&mut app_b, 10, 50.0);
    enter_playing(&mut app_b);

    let a_set = link_partners(&mut app_a);
    let b_set = link_partners(&mut app_b);
    assert_eq!(
        a_set, b_set,
        "same seed must produce same link set (a={a_set:?}, b={b_set:?})"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 28 — Determinism: different seed → different pair selection
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn different_seed_produces_different_pair_selection() {
    let mut app_a = build_establish_tether_app(42);
    install_tether_config(&mut app_a, canonical_tether_config());
    add_tether_stacks(&mut app_a, 1);
    spawn_cell_row(&mut app_a, 10, 50.0);
    enter_playing(&mut app_a);

    let mut app_b = build_establish_tether_app(1337);
    install_tether_config(&mut app_b, canonical_tether_config());
    add_tether_stacks(&mut app_b, 1);
    spawn_cell_row(&mut app_b, 10, 50.0);
    enter_playing(&mut app_b);

    let a_set = link_partners(&mut app_a);
    let b_set = link_partners(&mut app_b);
    assert_ne!(
        a_set, b_set,
        "different seeds must produce different link sets (a={a_set:?}, b={b_set:?})"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 29 — Harness-safe without GameRng
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn establish_is_safe_without_game_rng() {
    let mut app = build_establish_tether_app_no_rng();
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);
    spawn_cell_row(&mut app, 10, 50.0);
    enter_playing(&mut app);

    // No panic is the primary check. No links inserted either.
    assert_eq!(
        count_linked_pairs(&mut app),
        0,
        "no GameRng → system early-returns → zero links"
    );
}

#[test]
fn establish_is_safe_without_rng_and_without_config() {
    let mut app = build_establish_tether_app_no_rng();
    add_tether_stacks(&mut app, 1);
    spawn_cell_row(&mut app, 10, 50.0);
    enter_playing(&mut app);

    assert_eq!(count_linked_pairs(&mut app), 0);
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 30 — System does NOT run when Tether is inactive
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn system_does_not_run_when_tether_inactive() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    // Add a non-Tether hazard stack instead.
    add_hazard_stacks(&mut app, HazardKind::Volatility, 1);
    spawn_cell_row(&mut app, 10, 50.0);
    enter_playing(&mut app);

    assert_eq!(
        count_linked_pairs(&mut app),
        0,
        "Tether inactive → run-condition gate off → zero links"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// Behavior 31 — System does not panic when TetherConfig absent
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn system_does_not_panic_without_tether_config() {
    let mut app = build_establish_tether_app(42);
    // Deliberately omit install_tether_config.
    add_tether_stacks(&mut app, 1);
    spawn_cell_row(&mut app, 10, 50.0);
    enter_playing(&mut app);

    assert_eq!(
        count_linked_pairs(&mut app),
        0,
        "no TetherConfig → Option<Res<_>> early-return → zero links"
    );
}

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
// Behavior 34 — Dead/Invulnerable cells are filtered out of eligibility
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn dead_and_invulnerable_cells_are_filtered_out() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 7); // 100% coverage to force selection attempt.
    // cells at (0,0), (50,0), (100,0); middle one is Invulnerable.
    // Remaining survivors (0) & (100) have d² = 10000 > 4900 → no eligible pair.
    let _a = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0));
    let _b = spawn_cell_invulnerable_at(&mut app, Vec2::new(50.0, 0.0));
    let _c = spawn_cell_at(&mut app, Vec2::new(100.0, 0.0));
    enter_playing(&mut app);

    assert_eq!(
        count_linked_pairs(&mut app),
        0,
        "filtered cell invisible to Tether; outer survivors too far apart"
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
