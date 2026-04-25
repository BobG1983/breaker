//! Group D — `sympathy_heal_adjacent` positive message shape (Behaviors 34–43).
//!
//! Each damage event yields a ring-1 heal per adjacent cell, with geometric
//! attenuation through subsequent rings. Self-heal is excluded at all depths.
//! Every emitted `HealDealt<Cell>` carries `cap: HealCap::Starting`,
//! `source: Some("hazard:sympathy")`, `healer: None`, and the neighbour's
//! entity as `target`.
//!
//! Coordinate convention: `ADJACENCY_RADIUS_SQ = 4900.0` (70² world units).
//! Cells 50 units apart are adjacent (d² = 2500); 100 units apart are NOT
//! adjacent (d² = 10000).

use bevy::prelude::*;

use super::{
    super::system::sympathy_heal_adjacent,
    helpers::{
        add_sympathy_stacks, canonical_sympathy_config, heal_collector_len, heals_for_cell,
        install_sympathy_config, run_fixed_update, spawn_cell_at_default, test_app_playing,
        write_cell_damage,
    },
};
use crate::{hazard::definition::HazardKind, prelude::*};

// ── Behavior 34 — stack 1, 100 damage, two adjacent neighbours heal 25 each ─

#[test]
fn stack_one_damage_100_two_neighbours_each_heal_25() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 1);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    let c = spawn_cell_at_default(&mut app, Vec2::new(-50.0, 0.0));

    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    assert_eq!(
        heal_collector_len(&app),
        2,
        "expected 2 heals (one per adjacent neighbour); got {}",
        heal_collector_len(&app)
    );

    let heals_b = heals_for_cell(&app, b);
    assert_eq!(heals_b.len(), 1, "B must receive exactly 1 heal");
    assert!(
        (heals_b[0].amount - 25.0).abs() < 1e-4,
        "B heal must be 25.0, got {}",
        heals_b[0].amount
    );
    assert!(matches!(heals_b[0].cap, HealCap::Starting));
    assert_eq!(
        heals_b[0].source.as_ref(),
        Some(&SourceId::hazard(HazardKind::Sympathy).build())
    );
    assert_eq!(heals_b[0].healer, None);
    assert_eq!(heals_b[0].target, b);

    let heals_c = heals_for_cell(&app, c);
    assert_eq!(heals_c.len(), 1, "C must receive exactly 1 heal");
    assert!(
        (heals_c[0].amount - 25.0).abs() < 1e-4,
        "C heal must be 25.0, got {}",
        heals_c[0].amount
    );
    assert!(matches!(heals_c[0].cap, HealCap::Starting));
    assert_eq!(
        heals_c[0].source.as_ref(),
        Some(&SourceId::hazard(HazardKind::Sympathy).build())
    );

    let heals_a = heals_for_cell(&app, a);
    assert_eq!(heals_a.len(), 0, "A must never self-heal");
}

// ── Behavior 35 — stack 3, 80 damage, three neighbours heal 28 each ─────────

#[test]
fn stack_three_damage_80_three_neighbours_each_heal_28() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 3);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    let c = spawn_cell_at_default(&mut app, Vec2::new(-50.0, 0.0));
    let d = spawn_cell_at_default(&mut app, Vec2::new(0.0, 50.0));

    write_cell_damage(&mut app, a, 80.0);

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 3);
    // Stack 3 heal_percent = 25 + 5*2 = 35%. 80 × 0.35 = 28.
    for entity in [b, c, d] {
        let msgs = heals_for_cell(&app, entity);
        assert_eq!(
            msgs.len(),
            1,
            "each neighbour must receive exactly 1 heal; got {}",
            msgs.len()
        );
        assert!(
            (msgs[0].amount - 28.0).abs() < 1e-4,
            "heal amount must be 28.0, got {}",
            msgs[0].amount
        );
        assert!(matches!(msgs[0].cap, HealCap::Starting));
        assert_eq!(
            msgs[0].source.as_ref(),
            Some(&SourceId::hazard(HazardKind::Sympathy).build())
        );
        assert_eq!(msgs[0].healer, None);
    }
}

// ── Behavior 36 — stack 6: ring-1 via direct, ring-2 via intermediary ───────

#[test]
fn stack_six_ring_two_through_intermediary_heals_attenuate() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 6);

    // A-B adjacent (d²=2500), B-C adjacent (d²=2500), A-C NOT adjacent (d²=10000).
    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    let c = spawn_cell_at_default(&mut app, Vec2::new(100.0, 0.0));

    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 2);

    // Stack 6: heal_percent = 50.0, cascade_depth = 2.
    // Ring 1 heal (B) = 100 × 0.5 = 50.
    // Ring 2 heal (C) = 50 × 0.5 = 25.
    let heals_b = heals_for_cell(&app, b);
    assert_eq!(heals_b.len(), 1);
    assert!(
        (heals_b[0].amount - 50.0).abs() < 1e-4,
        "B (ring 1) heal must be 50.0, got {}",
        heals_b[0].amount
    );

    let heals_c = heals_for_cell(&app, c);
    assert_eq!(heals_c.len(), 1);
    assert!(
        (heals_c[0].amount - 25.0).abs() < 1e-4,
        "C (ring 2) heal must be 25.0 (50% of ring-1 heal), got {}",
        heals_c[0].amount
    );

    assert!(heals_for_cell(&app, a).is_empty(), "A must not self-heal");

    // All emitted messages carry the canonical cap/source/healer shape.
    let all = app
        .world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0
        .clone();
    assert!(all.iter().all(|m| matches!(m.cap, HealCap::Starting)));
    assert!(
        all.iter()
            .all(|m| m.source == Some(SourceId::hazard(HazardKind::Sympathy).build()))
    );
    assert!(all.iter().all(|m| m.healer.is_none()));
}

// ── Behavior 37 — isolated damaged cell produces no heals ───────────────────

#[test]
fn isolated_damaged_cell_produces_no_heals() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 1);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    // D is at (500, 500), distance² = 500000 > 4900.
    let _d = spawn_cell_at_default(&mut app, Vec2::new(500.0, 500.0));

    write_cell_damage(&mut app, a, 50.0);

    run_fixed_update(&mut app);

    assert_eq!(heal_collector_len(&app), 0);
}

// ── Behavior 38 — damaged cell is never in its own heal set (ring 1) ────────

#[test]
fn damaged_cell_is_never_in_own_heal_set_ring_one() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 1);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));

    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    assert!(heals_for_cell(&app, a).is_empty(), "A must not self-heal");
    let heals_b = heals_for_cell(&app, b);
    assert_eq!(heals_b.len(), 1);
    assert!((heals_b[0].amount - 25.0).abs() < 1e-4);
}

// ── Behavior 39 — self-heal exclusion at depth >= 2 (A never re-enters BFS) ─

#[test]
fn self_heal_exclusion_holds_at_depth_two() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 6);

    // A=(0,0), B=(50,0) [ring 1 of A], C=(100,0) [ring 2 via B], D=(50,50) [ring 1 of B, ring 2 overall].
    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    let c = spawn_cell_at_default(&mut app, Vec2::new(100.0, 0.0));
    let d = spawn_cell_at_default(&mut app, Vec2::new(50.0, 50.0));

    write_cell_damage(&mut app, a, 100.0);

    run_fixed_update(&mut app);

    assert!(
        heals_for_cell(&app, a).is_empty(),
        "A must not appear in any ring"
    );
    assert_eq!(heals_for_cell(&app, b).len(), 1, "B must receive ring-1");
    assert_eq!(heals_for_cell(&app, c).len(), 1, "C must receive ring-2");
    assert_eq!(heals_for_cell(&app, d).len(), 1, "D must receive ring-2");
}

// ── Behavior 40 — every emitted message carries HealCap::Starting ───────────

#[test]
fn every_emitted_message_has_cap_starting() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 3);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let _b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    let _c = spawn_cell_at_default(&mut app, Vec2::new(-50.0, 0.0));
    let _d = spawn_cell_at_default(&mut app, Vec2::new(0.0, 50.0));
    write_cell_damage(&mut app, a, 80.0);

    run_fixed_update(&mut app);

    let all = app
        .world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0
        .clone();
    assert_eq!(all.len(), 3);
    assert!(
        all.iter().all(|m| matches!(m.cap, HealCap::Starting)),
        "every message must carry HealCap::Starting"
    );
}

// ── Behavior 41 — every emitted message carries source=Some("hazard:sympathy")

#[test]
fn every_emitted_message_has_sympathy_sentinel_source() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 3);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let _b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    let _c = spawn_cell_at_default(&mut app, Vec2::new(-50.0, 0.0));
    let _d = spawn_cell_at_default(&mut app, Vec2::new(0.0, 50.0));
    write_cell_damage(&mut app, a, 80.0);

    run_fixed_update(&mut app);

    let all = app
        .world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0
        .clone();
    assert_eq!(all.len(), 3);
    assert!(
        all.iter()
            .all(|m| m.source == Some(SourceId::hazard(HazardKind::Sympathy).build())),
        "every message's source must be Some(\"hazard:sympathy\")"
    );
}

// ── Behavior 42 — every emitted message carries healer == None ──────────────

#[test]
fn every_emitted_message_has_healer_none() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 3);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let _b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    let _c = spawn_cell_at_default(&mut app, Vec2::new(-50.0, 0.0));
    let _d = spawn_cell_at_default(&mut app, Vec2::new(0.0, 50.0));
    write_cell_damage(&mut app, a, 80.0);

    run_fixed_update(&mut app);

    let all = app
        .world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0
        .clone();
    assert_eq!(all.len(), 3);
    assert!(all.iter().all(|m| m.healer.is_none()));
}

// ── Behavior 43 — every emitted message's target matches the neighbour ──────

#[test]
fn every_message_target_matches_neighbour_entity() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 3);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));
    let c = spawn_cell_at_default(&mut app, Vec2::new(-50.0, 0.0));
    let d = spawn_cell_at_default(&mut app, Vec2::new(0.0, 50.0));
    write_cell_damage(&mut app, a, 80.0);

    run_fixed_update(&mut app);

    for entity in [b, c, d] {
        let matches_count = app
            .world()
            .resource::<MessageCollector<HealDealt<Cell>>>()
            .0
            .iter()
            .filter(|m| m.target == entity)
            .count();
        assert_eq!(
            matches_count, 1,
            "each of {{B, C, D}} must have exactly 1 message targeting it"
        );
    }
}

// ── B33: source matches builder-produced hazard:sympathy ──

#[test]
fn sympathy_heal_source_equals_builder() {
    use crate::{hazard::definition::HazardKind, prelude::SourceIdExt};
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, sympathy_heal_adjacent);
    install_sympathy_config(&mut app, canonical_sympathy_config());
    add_sympathy_stacks(&mut app, 1);

    let a = spawn_cell_at_default(&mut app, Vec2::ZERO);
    let b = spawn_cell_at_default(&mut app, Vec2::new(50.0, 0.0));

    write_cell_damage(&mut app, a, 100.0);
    run_fixed_update(&mut app);

    let heals = heals_for_cell(&app, b);
    assert!(!heals.is_empty(), "expected at least 1 sympathy heal");
    let expected = SourceId::hazard(HazardKind::Sympathy).build();
    for heal in &heals {
        assert_eq!(heal.source.as_ref(), Some(&expected));
    }
}
