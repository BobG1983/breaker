//! Group E — Chain reactions.

use bevy::prelude::*;

use super::helpers::*;
use crate::prelude::*;

// Behavior 25
#[test]
fn three_volatile_cells_chain_reaction_kills_all_through_chained_detonations() {
    let mut app = build_volatile_test_app();

    let a = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 35.0, 10.0);
    let b = spawn_volatile_cell(&mut app, Vec2::new(30.0, 0.0), 25.0, 35.0, 10.0);
    let c = spawn_volatile_cell(&mut app, Vec2::new(60.0, 0.0), 25.0, 35.0, 10.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(a, 100.0)]));

    // Reliability pin: after exactly 3 ticks, all three must be dead. This is
    // the concrete reliability claim from Behavior 25 in the spec.
    let victims = run_ticks_and_collect_destroyed(&mut app, 3);

    assert!(
        app.world().get_entity(a).is_err(),
        "A should be despawned after 3 ticks"
    );
    assert!(
        app.world().get_entity(b).is_err(),
        "B should be despawned after 3 ticks (within A's radius 35)"
    );
    assert!(
        app.world().get_entity(c).is_err(),
        "C should be despawned after 3 ticks (within B's radius 35)"
    );

    let set: std::collections::HashSet<Entity> = victims.into_iter().collect();
    assert_eq!(
        set,
        std::collections::HashSet::from([a, b, c]),
        "exactly A, B, C should appear across all Destroyed<Cell> messages within 3 ticks"
    );
}

// Behavior 26
#[test]
fn three_volatile_cells_chain_stops_when_gap_exceeds_radius() {
    let mut app = build_volatile_test_app();

    let a = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 35.0, 10.0);
    let b = spawn_volatile_cell(&mut app, Vec2::new(30.0, 0.0), 25.0, 35.0, 10.0);
    let c = spawn_volatile_cell(&mut app, Vec2::new(100.0, 0.0), 25.0, 35.0, 10.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(a, 100.0)]));

    let victims = run_ticks_and_collect_destroyed(&mut app, 5);

    assert!(app.world().get_entity(a).is_err());
    assert!(app.world().get_entity(b).is_err());
    assert!(
        app.world().get_entity(c).is_ok(),
        "C should still be present — B→C distance 70 > radius 35"
    );
    let c_hp = app.world().get::<Hp>(c).unwrap();
    assert!((c_hp.current - 10.0).abs() < f32::EPSILON);
    assert!(app.world().get::<Dead>(c).is_none());

    let set: std::collections::HashSet<Entity> = victims.into_iter().collect();
    assert_eq!(set, std::collections::HashSet::from([a, b]));
}

// Behavior 26 edge: radius 29 — A alone
#[test]
fn volatile_chain_stops_when_radius_below_pair_distance() {
    let mut app = build_volatile_test_app();

    let a = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 29.0, 10.0);
    let b = spawn_volatile_cell(&mut app, Vec2::new(30.0, 0.0), 25.0, 29.0, 10.0);
    let c = spawn_volatile_cell(&mut app, Vec2::new(100.0, 0.0), 25.0, 29.0, 10.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(a, 100.0)]));

    let victims = run_ticks_and_collect_destroyed(&mut app, 5);

    assert!(app.world().get_entity(a).is_err());
    assert!(app.world().get_entity(b).is_ok());
    assert!(app.world().get_entity(c).is_ok());
    let b_hp = app.world().get::<Hp>(b).unwrap();
    assert!((b_hp.current - 10.0).abs() < f32::EPSILON);
    let c_hp = app.world().get::<Hp>(c).unwrap();
    assert!((c_hp.current - 10.0).abs() < f32::EPSILON);
    assert!(app.world().get::<Dead>(b).is_none());
    assert!(app.world().get::<Dead>(c).is_none());

    assert_eq!(victims, vec![a]);
}

// Behavior 27
#[test]
fn non_volatile_middle_cell_breaks_chain() {
    let mut app = build_volatile_test_app();

    let a = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 35.0, 10.0);
    let b = spawn_plain_cell(&mut app, Vec2::new(30.0, 0.0), 10.0);
    let c = spawn_volatile_cell(&mut app, Vec2::new(60.0, 0.0), 25.0, 35.0, 10.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(a, 100.0)]));

    let victims = run_ticks_and_collect_destroyed(&mut app, 5);

    assert!(app.world().get_entity(a).is_err());
    assert!(app.world().get_entity(b).is_err());
    assert!(
        app.world().get_entity(c).is_ok(),
        "C should survive — A→C distance 60 > radius 35, and B is non-volatile"
    );
    let c_hp = app.world().get::<Hp>(c).unwrap();
    assert!((c_hp.current - 10.0).abs() < f32::EPSILON);
    assert!(app.world().get::<Dead>(c).is_none());

    let set: std::collections::HashSet<Entity> = victims.into_iter().collect();
    assert_eq!(set, std::collections::HashSet::from([a, b]));
}

// Behavior 27 edge: replace B with a volatile whose radius just barely reaches C
#[test]
fn volatile_middle_cell_chains_to_c_via_exact_boundary() {
    let mut app = build_volatile_test_app();

    let a = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 35.0, 10.0);
    // B at (25, 0): A→B distance 25 ✓ (<35), B→C distance 35 exactly (inclusive).
    let b = spawn_volatile_cell(&mut app, Vec2::new(25.0, 0.0), 25.0, 35.0, 10.0);
    let c = spawn_volatile_cell(&mut app, Vec2::new(60.0, 0.0), 25.0, 35.0, 10.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(a, 100.0)]));

    let victims = run_ticks_and_collect_destroyed(&mut app, 5);

    assert!(app.world().get_entity(a).is_err());
    assert!(app.world().get_entity(b).is_err());
    assert!(app.world().get_entity(c).is_err());

    let set: std::collections::HashSet<Entity> = victims.into_iter().collect();
    assert_eq!(set, std::collections::HashSet::from([a, b, c]));
}
