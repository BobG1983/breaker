//! Group E — `sympathy_heal_adjacent` edge cases, topology, accumulation
//! (Behaviors 44–52).
//!
//! Dead / Invulnerable cells must be excluded from the adjacency snapshot so
//! BFS can neither heal them nor propagate through them. Multiple independent
//! damage events each yield their own heal set. A cell adjacent to two damaged
//! primaries in one tick receives two heal messages. BFS visited-set semantics
//! put a cell in the earliest ring that reaches it.

use bevy::prelude::*;

use super::{
    super::system::sympathy_heal_adjacent,
    helpers::{
        add_sympathy_stacks, canonical_sympathy_config, heal_collector_len, heals_for_cell,
        install_sympathy_config, run_fixed_update, spawn_cell_at_default, spawn_cell_dead_at,
        spawn_cell_invulnerable_at, test_app_playing, write_cell_damage,
    },
};

// ── Behavior 44 — Dead neighbour is excluded from BFS adjacency set ─────────

#[test]
fn dead_neighbour_is_excluded_from_bfs_adjacency() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 1);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let b = spawn_cell_dead_at(&mut app, Vec2::new(50.0, 0.0));
    let c = spawn_cell_at_default(&mut app, Vec2::new(-50.0, 0.0));

    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 1);
    assert_eq!(
        heals_for_cell(&app, b).len(),
        0,
        "Dead cell B must be excluded from the adjacency set"
    );
    let heals_c = heals_for_cell(&app, c);
    assert_eq!(heals_c.len(), 1);
    assert!((heals_c[0].amount - 25.0).abs() < 1e-4);
}

// ── Behavior 45 — Invulnerable neighbour is excluded from BFS adjacency ─────

#[test]
fn invulnerable_neighbour_is_excluded_from_bfs_adjacency() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 1);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let b = spawn_cell_invulnerable_at(&mut app, Vec2::new(50.0, 0.0));
    let c = spawn_cell_at_default(&mut app, Vec2::new(-50.0, 0.0));

    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 1);
    assert_eq!(
        heals_for_cell(&app, b).len(),
        0,
        "Invulnerable cell B must be excluded from the adjacency set"
    );
    let heals_c = heals_for_cell(&app, c);
    assert_eq!(heals_c.len(), 1);
    assert!((heals_c[0].amount - 25.0).abs() < 1e-4);
}

// ── Behavior 46 — Dead ring-1 intermediary breaks the BFS chain ─────────────

#[test]
fn dead_ring_one_intermediary_breaks_chain() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 6);

    // A=(0,0), B=(50,0) Dead, C=(100,0). A-B adjacent, B-C adjacent, A-C NOT adjacent.
    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let _b_dead = spawn_cell_dead_at(&mut app, Vec2::new(50.0, 0.0));
    let _c = spawn_cell_at_default(&mut app, Vec2::new(100.0, 0.0));

    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    assert_eq!(
        heal_collector_len(&app),
        0,
        "Dead intermediary must break the BFS chain; C is not reachable"
    );
}

// ── Behavior 47 — multiple independent DamageDealt messages → independent sets

#[test]
fn multiple_independent_damage_messages_each_emit_own_heal_set() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 1);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    let x = spawn_cell_at_default(&mut app, Vec2::new(1000.0, 0.0));
    let y = spawn_cell_at_default(&mut app, Vec2::new(1050.0, 0.0));

    write_cell_damage(&mut app, a, 100.0);
    write_cell_damage(&mut app, x, 100.0);

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 2);

    let heals_b = heals_for_cell(&app, b);
    assert_eq!(heals_b.len(), 1);
    assert!((heals_b[0].amount - 25.0).abs() < 1e-4);

    let heals_y = heals_for_cell(&app, y);
    assert_eq!(heals_y.len(), 1);
    assert!((heals_y[0].amount - 25.0).abs() < 1e-4);
}

// ── Behavior 48 — cell adjacent to two damaged primaries gets two heals ─────

#[test]
fn cell_adjacent_to_two_primaries_gets_two_heal_messages() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 1);

    // A at (0,0), C at (50,0) — BOTH damaged; B at (25,0) adjacent to both.
    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let c = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    let b = spawn_cell_at_default(&mut app, Vec2::new(25.0, 0.0));

    write_cell_damage(&mut app, a, 100.0);
    write_cell_damage(&mut app, c, 100.0);

    run_fixed_update(&mut app);

    let heals_b = heals_for_cell(&app, b);
    assert_eq!(
        heals_b.len(),
        2,
        "B must receive TWO heal messages (one per damaged primary); got {}",
        heals_b.len()
    );
    for msg in &heals_b {
        assert!(
            (msg.amount - 25.0).abs() < 1e-4,
            "each heal amount must be 25.0, got {}",
            msg.amount
        );
        assert!(matches!(
            msg.cap,
            crate::shared::death_pipeline::HealCap::Starting
        ));
        assert_eq!(msg.source.as_deref(), Some("hazard:sympathy"));
    }
}

// ── Behavior 49 — cascade depth 3 (stack 11) with geometric attenuation ─────

#[test]
fn cascade_depth_three_three_rings_with_geometric_attenuation() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 11);

    // Chain A=(0,0) → B=(50,0) → C=(100,0) → D=(150,0); only neighbour segs adjacent.
    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    let c = spawn_cell_at_default(&mut app, Vec2::new(100.0, 0.0));
    let d = spawn_cell_at_default(&mut app, Vec2::new(150.0, 0.0));

    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 3);

    // Stack 11: heal_percent == 75.0 (factor 0.75), cascade_depth == 3.
    let heals_b = heals_for_cell(&app, b);
    assert_eq!(heals_b.len(), 1);
    assert!(
        (heals_b[0].amount - 75.0).abs() < 1e-3,
        "B (ring 1) heal must be 75.0, got {}",
        heals_b[0].amount
    );

    let heals_c = heals_for_cell(&app, c);
    assert_eq!(heals_c.len(), 1);
    assert!(
        (heals_c[0].amount - 56.25).abs() < 1e-3,
        "C (ring 2) heal must be 56.25, got {}",
        heals_c[0].amount
    );

    let heals_d = heals_for_cell(&app, d);
    assert_eq!(heals_d.len(), 1);
    assert!(
        (heals_d[0].amount - 42.1875).abs() < 1e-3,
        "D (ring 3) heal must be 42.1875, got {}",
        heals_d[0].amount
    );

    assert!(heals_for_cell(&app, a).is_empty(), "A must not self-heal");
}

// ── Behavior 50 — cascade terminates when ring N+1 has no unvisited neighbour

#[test]
fn cascade_terminates_when_ring_has_no_unvisited_neighbour() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 6);

    // Only A-B; no ring-2 neighbour. depth=2 but ring 2 is empty → terminates.
    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));

    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 1);
    let heals_b = heals_for_cell(&app, b);
    assert_eq!(heals_b.len(), 1);
    assert!(
        (heals_b[0].amount - 50.0).abs() < 1e-4,
        "B (ring 1) must be 50.0, got {}",
        heals_b[0].amount
    );
}

// ── Behavior 51 — visited set excludes cells already placed in a prior ring ─

#[test]
fn visited_set_excludes_cells_already_in_earlier_ring() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 6);

    // A=(0,0), B=(50,0), C=(100,0). B' at (25,25): adjacent to A AND B.
    // B' MUST land in ring 1 (reached via A directly), not ring 2 (via B).
    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    let c = spawn_cell_at_default(&mut app, Vec2::new(100.0, 0.0));
    let b_prime = spawn_cell_at_default(&mut app, Vec2::new(25.0, 25.0));

    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 3);

    // B' must appear in ring 1 (amount 50.0), exactly once.
    let heals_b_prime = heals_for_cell(&app, b_prime);
    assert_eq!(heals_b_prime.len(), 1);
    assert!(
        (heals_b_prime[0].amount - 50.0).abs() < 1e-4,
        "B' must be ring 1 (amount 50.0), got {}",
        heals_b_prime[0].amount
    );

    // B is ring 1 (amount 50.0), C is ring 2 (amount 25.0).
    let heals_b = heals_for_cell(&app, b);
    assert_eq!(heals_b.len(), 1);
    assert!(
        (heals_b[0].amount - 50.0).abs() < 1e-4,
        "B ring-1 heal, got {}",
        heals_b[0].amount
    );
    let heals_c = heals_for_cell(&app, c);
    assert_eq!(heals_c.len(), 1);
    assert!(
        (heals_c[0].amount - 25.0).abs() < 1e-4,
        "C ring-2 heal, got {}",
        heals_c[0].amount
    );
}

// ── Behavior 52 — large stack (20) does not panic and produces heal 120 ─────

#[test]
fn large_stack_twenty_does_not_panic_and_produces_overheal() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 20);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));

    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    let heals_b = heals_for_cell(&app, b);
    assert_eq!(heals_b.len(), 1);
    // heal_percent(20) == 120.0 → heal amount 100.0 × 1.2 = 120.0.
    assert!(
        (heals_b[0].amount - 120.0).abs() < 1e-4,
        "stack-20 overheal must be 120.0, got {}",
        heals_b[0].amount
    );
}
