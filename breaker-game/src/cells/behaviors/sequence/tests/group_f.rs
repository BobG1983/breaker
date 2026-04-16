//! Group F — Sequence × Volatile emergent combo.
//!
//! Volatile chain damage on non-active sequence cells must be reverted, not
//! cascaded. This is the marquee design intent of the Sequence mechanic.

use bevy::prelude::*;

use super::helpers::*;
use crate::{cells::components::SequenceActive, prelude::*};

// ── Behavior 24 ────────────────────────────────────────────────────────────

#[test]
fn volatile_blast_on_non_active_sequence_cell_leaves_it_at_full_hp() {
    let mut app = build_sequence_test_app();

    let v0 = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    // e0 is OUTSIDE the radius (distance 50 > radius 40) — stays alive and
    // active so the group does not advance during this test.
    let e0 = spawn_sequence_cell(&mut app, Vec2::new(50.0, 0.0), 1, 0, 20.0);
    // e1 is INSIDE the radius (distance 20 < radius 40) — the non-active
    // sequence cell whose damage must be reverted.
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 1, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(v0, 100.0));
    let (destroyed, damage) = run_ticks_capture_destroyed_and_damage(&mut app, 3);

    assert!(
        destroyed.iter().any(|m| m.victim == v0),
        "v0 should be destroyed"
    );

    let e1_hp = app.world().get::<Hp>(e1).expect("e1 should still have Hp");
    assert!(
        (e1_hp.current - 20.0).abs() < f32::EPSILON,
        "non-active e1 should be reset to 20.0 after the blast, got {}",
        e1_hp.current
    );
    assert!(app.world().get::<Dead>(e1).is_none());
    assert!(!destroyed.iter().any(|m| m.victim == e1));

    // Proof that the blast did hit e1 (otherwise this test would false-pass).
    assert!(
        damage
            .iter()
            .any(|m| m.target == e1 && (m.amount - 25.0).abs() < f32::EPSILON),
        "the captured DamageDealt<Cell> stream should include a 25.0 hit on e1"
    );

    let e0_hp = app.world().get::<Hp>(e0).expect("e0 should still have Hp");
    assert!((e0_hp.current - 20.0).abs() < f32::EPSILON);
    assert!(app.world().get::<SequenceActive>(e0).is_some());
}

// ── Behavior 24 edge: two non-active sequence cells in the radius
#[test]
fn volatile_blast_with_two_non_active_sequence_cells_resets_both() {
    let mut app = build_sequence_test_app();

    let v0 = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let _e0 = spawn_sequence_cell(&mut app, Vec2::new(50.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 1, 20.0);
    let e1b = spawn_sequence_cell(&mut app, Vec2::new(30.0, 0.0), 1, 2, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(v0, 100.0));
    run_ticks_capture_destroyed_and_damage(&mut app, 3);

    let first_hp = app.world().get::<Hp>(e1).unwrap();
    let second_hp = app.world().get::<Hp>(e1b).unwrap();
    assert!(
        (first_hp.current - 20.0).abs() < f32::EPSILON,
        "e1 should be reset, got {}",
        first_hp.current
    );
    assert!(
        (second_hp.current - 20.0).abs() < f32::EPSILON,
        "e1b should be reset, got {}",
        second_hp.current
    );
}

// ── Behavior 25 ────────────────────────────────────────────────────────────

#[test]
fn volatile_blast_kills_only_the_active_sequence_cell_in_range() {
    let mut app = build_sequence_test_app();

    let v0 = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let e0 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 1, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(v0, 100.0));
    let (destroyed, _damage) = run_ticks_capture_destroyed_and_damage(&mut app, 3);

    assert!(destroyed.iter().any(|m| m.victim == v0));
    assert!(
        destroyed.iter().any(|m| m.victim == e0),
        "active e0 should die — damage is NOT reverted on active cells"
    );
    assert!(
        !destroyed.iter().any(|m| m.victim == e1),
        "e1 should not be destroyed — its blast damage is reverted"
    );

    let e1_hp = app.world().get::<Hp>(e1).expect("e1 should still have Hp");
    assert!(
        (e1_hp.current - 20.0).abs() < f32::EPSILON,
        "e1 should be at full HP, got {}",
        e1_hp.current
    );
    assert!(
        app.world().get::<SequenceActive>(e1).is_some(),
        "e1 should be promoted to active after e0 dies"
    );
}

// ── Behavior 25 edge: e1 OUTSIDE radius still gets promoted without damage
#[test]
fn volatile_blast_promotes_out_of_range_next_member_without_damage() {
    let mut app = build_sequence_test_app();

    let v0 = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let e0 = spawn_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 0, 20.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(50.0, 0.0), 1, 1, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(v0, 100.0));
    run_ticks_capture_destroyed_and_damage(&mut app, 3);

    assert!(
        app.world().get_entity(e0).is_err() || app.world().get::<Dead>(e0).is_some(),
        "e0 should be dead"
    );
    let e1_hp = app.world().get::<Hp>(e1).unwrap();
    assert!((e1_hp.current - 20.0).abs() < f32::EPSILON);
    assert!(
        app.world().get::<SequenceActive>(e1).is_some(),
        "e1 should be active after e0 dies"
    );
}

// ── Behavior 26 ────────────────────────────────────────────────────────────

#[test]
fn chain_reaction_between_two_volatiles_resets_non_active_sequence_twice() {
    let mut app = build_sequence_test_app();

    let v0 = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let v1 = spawn_volatile_cell(&mut app, Vec2::new(30.0, 0.0), 25.0, 40.0, 10.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(15.0, 0.0), 1, 1, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(v0, 100.0));
    // 4 is a safe over-provision per the spec's tick-count safety net.
    let (destroyed, damage) = run_ticks_capture_destroyed_and_damage(&mut app, 4);

    // Both volatiles should be destroyed.
    let destroyed_set: std::collections::HashSet<Entity> =
        destroyed.iter().map(|m| m.victim).collect();
    assert!(
        destroyed_set.contains(&v0),
        "v0 should be in the Destroyed<Cell> set"
    );
    assert!(
        destroyed_set.contains(&v1),
        "v1 should be in the Destroyed<Cell> set"
    );
    assert!(
        !destroyed_set.contains(&e1),
        "e1 should not be in the Destroyed<Cell> set"
    );

    let e1_hp = app.world().get::<Hp>(e1).expect("e1 should still have Hp");
    assert!(
        (e1_hp.current - 20.0).abs() < f32::EPSILON,
        "e1 should still be at full HP, got {}",
        e1_hp.current
    );
    assert!(app.world().get::<Dead>(e1).is_none());

    // Proof that e1 was hit by both blasts.
    let e1_hits = damage.iter().filter(|m| m.target == e1).count();
    assert!(
        e1_hits >= 2,
        "e1 should have been hit at least twice across the chain, got {e1_hits} hits",
    );

    let killed_by = app.world().get::<KilledBy>(e1).unwrap();
    assert!(
        killed_by.dealer.is_none(),
        "the reset should have cleared KilledBy.dealer, got {:?}",
        killed_by.dealer
    );
}

// ── Behavior 26 edge: e1 at position 0 takes the blast while active and dies
#[test]
fn volatile_chain_kills_active_position_zero_sequence_cell() {
    let mut app = build_sequence_test_app();

    let v0 = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let v1 = spawn_volatile_cell(&mut app, Vec2::new(30.0, 0.0), 25.0, 40.0, 10.0);
    let e1 = spawn_sequence_cell(&mut app, Vec2::new(15.0, 0.0), 1, 0, 20.0);
    advance_to_playing(&mut app);

    push_damage(&mut app, damage_msg(v0, 100.0));
    let (destroyed, _damage) = run_ticks_capture_destroyed_and_damage(&mut app, 4);

    let destroyed_set: std::collections::HashSet<Entity> =
        destroyed.iter().map(|m| m.victim).collect();
    // v0 and v1 dead.
    assert!(destroyed_set.contains(&v0));
    assert!(destroyed_set.contains(&v1));
    // e1 is active (position 0) and in range — it should die.
    assert!(
        destroyed_set.contains(&e1) || app.world().get::<Dead>(e1).is_some(),
        "active position-0 cell in range should die from the blast"
    );
}
