//! Behaviors 27–28 — determinism under shared `HazardRng`.

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

// ── Group E B28: same HazardRng seed → identical link set (independent apps) ──

#[test]
fn establish_tether_links_is_deterministic_for_same_hazard_rng_seed() {
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
        "same HazardRng seed must produce identical link sets across independent apps"
    );
}

#[test]
fn establish_tether_links_seed_0_and_seed_42_produce_different_link_sets() {
    let mut app_0 = build_establish_tether_app(0);
    install_tether_config(&mut app_0, canonical_tether_config());
    add_tether_stacks(&mut app_0, 1);
    spawn_cell_row(&mut app_0, 10, 50.0);
    enter_playing(&mut app_0);

    let mut app_42 = build_establish_tether_app(42);
    install_tether_config(&mut app_42, canonical_tether_config());
    add_tether_stacks(&mut app_42, 1);
    spawn_cell_row(&mut app_42, 10, 50.0);
    enter_playing(&mut app_42);

    let set_0 = link_partners(&mut app_0);
    let set_42 = link_partners(&mut app_42);
    assert_ne!(
        set_0, set_42,
        "HazardRng seed 0 and seed 42 must produce different link sets (different stream → different shuffle)"
    );
}
