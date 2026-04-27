//! Behaviors 29–31 — harness-safety when resources are absent and run-condition gating.

use super::{
    super::helpers::{
        add_hazard_stacks, add_tether_stacks, build_establish_tether_app,
        build_establish_tether_app_no_rng, canonical_tether_config, enter_playing,
        install_tether_config, spawn_cell_row,
    },
    link_helpers::count_linked_pairs,
};
use crate::mutators::hazards::definition::HazardKind;

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
