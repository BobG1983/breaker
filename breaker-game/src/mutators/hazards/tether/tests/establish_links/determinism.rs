//! Behaviors 27–28 — determinism under shared `GameRng`.

use super::super::helpers::{
    add_tether_stacks, build_establish_tether_app, canonical_tether_config, enter_playing,
    install_tether_config, link_partners, spawn_cell_row,
};

// ════════════════════════════════════════════════════════════════════════════
// Behavior 27 — same seed → same pair selection
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
// Behavior 28 — different seed → different pair selection
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
