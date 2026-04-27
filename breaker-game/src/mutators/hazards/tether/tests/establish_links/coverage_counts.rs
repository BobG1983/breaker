//! Behaviors 20–23 — pair-count expectations across hazard stacks.

use std::collections::HashMap;

use bevy::prelude::*;

use super::{
    super::helpers::{
        add_tether_stacks, build_establish_tether_app, canonical_tether_config, enter_playing,
        install_tether_config, spawn_cell_row,
    },
    link_helpers::{count_linked_pairs, links_map},
};

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
