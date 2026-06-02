//! Behaviors 29–31 — harness-safety when resources are absent and run-condition gating.
//! Also Group E B26 (`HazardRng` migration) and B27 (no-rng early-return).

use super::{
    super::helpers::{
        add_hazard_stacks, add_tether_stacks, build_establish_tether_app,
        build_establish_tether_app_no_rng, canonical_tether_config, enter_playing,
        install_tether_config, spawn_cell_row,
    },
    link_helpers::count_linked_pairs,
};
use crate::{mutators::hazards::definition::HazardKind, shared::rng::HazardRng};

// ════════════════════════════════════════════════════════════════════════════
// Behavior 29 / B27 — Harness-safe without HazardRng
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn harness_safe_without_hazard_rng() {
    let mut app = build_establish_tether_app_no_rng();
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);
    spawn_cell_row(&mut app, 10, 50.0);
    enter_playing(&mut app);

    // No panic is the primary check. No links inserted either.
    assert_eq!(
        count_linked_pairs(&mut app),
        0,
        "no HazardRng → system early-returns → zero links"
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

// ── B27 edge case: GameRng inserted but NOT HazardRng — still no links ────

#[test]
fn game_rng_without_hazard_rng_still_produces_zero_links() {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use crate::prelude::GameRng;

    let mut app = build_establish_tether_app_no_rng();
    // Insert GameRng but NOT HazardRng.
    app.world_mut()
        .insert_resource(GameRng(ChaCha8Rng::seed_from_u64(42)));
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);
    spawn_cell_row(&mut app, 10, 50.0);
    enter_playing(&mut app);

    assert_eq!(
        count_linked_pairs(&mut app),
        0,
        "GameRng present but HazardRng absent → early-return guard fires → zero links"
    );
}

// ── Group E B26: establish_tether_links reads HazardRng (NOT GameRng) ─────

#[test]
fn establish_tether_links_reads_hazard_rng_not_game_rng() {
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;

    use crate::prelude::GameRng;

    const SENTINEL: u64 = 0xDEAD_BEEF_CAFE_1234;

    // build_establish_tether_app(42) inserts HazardRng::seed_from_u64(42).
    let mut app = build_establish_tether_app(42);
    // Also insert GameRng at SENTINEL — must remain untouched.
    app.world_mut()
        .insert_resource(GameRng(ChaCha8Rng::seed_from_u64(SENTINEL)));
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);
    spawn_cell_row(&mut app, 10, 50.0);
    enter_playing(&mut app);

    // System ran without panic; at least one link inserted (proves the system
    // reached the insertion path — i.e. HazardRng resolved successfully).
    assert!(
        count_linked_pairs(&mut app) >= 1,
        "establish_tether_links must resolve HazardRng and insert at least one TetherLink"
    );

    // GameRng stream must be unchanged.
    let world_draw: u64 = app.world_mut().resource_mut::<GameRng>().0.random();
    let sentinel_draw: u64 = ChaCha8Rng::seed_from_u64(SENTINEL).random();
    assert_eq!(
        world_draw, sentinel_draw,
        "establish_tether_links must NOT advance GameRng (stream was touched)"
    );
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
