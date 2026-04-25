//! Group D — End-to-end explosion at death.

use bevy::prelude::*;

use super::helpers::*;
use crate::prelude::*;

// Behavior 20
#[test]
fn volatile_detonation_damages_target_inside_radius_for_exact_damage() {
    let mut app = build_volatile_test_app();

    let source = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let target = spawn_plain_cell(&mut app, Vec2::new(30.0, 0.0), 25.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(source, 100.0)]));

    // 2 ticks: tick 1 kills source and fires ExplodeConfig → DamageDealt{target};
    // tick 2 applies that damage and kills target.
    let destroyed_msgs = run_ticks_capture_destroyed(&mut app, 2);

    assert!(
        app.world().get_entity(source).is_err(),
        "source cell should be despawned within 2 ticks"
    );
    assert!(
        app.world().get_entity(target).is_err(),
        "target cell (hp 25 == damage 25) should be killed by the detonation within 2 ticks"
    );

    let victims: Vec<Entity> = destroyed_msgs.iter().map(|m| m.victim).collect();
    assert!(
        victims.contains(&source),
        "Destroyed<Cell> should include source"
    );
    assert!(
        victims.contains(&target),
        "Destroyed<Cell> should include target"
    );

    let source_msg = destroyed_msgs
        .iter()
        .find(|m| m.victim == source)
        .expect("source destroyed");
    assert_eq!(source_msg.victim_pos, Vec2::new(0.0, 0.0));
    let target_msg = destroyed_msgs
        .iter()
        .find(|m| m.victim == target)
        .expect("target destroyed");
    assert_eq!(target_msg.victim_pos, Vec2::new(30.0, 0.0));
}

// Behavior 20 edge: partial damage — target survives
#[test]
fn volatile_detonation_applies_partial_damage_without_over_kill() {
    let mut app = build_volatile_test_app();

    let source = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let target = spawn_plain_cell(&mut app, Vec2::new(30.0, 0.0), 30.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(source, 100.0)]));

    // 2 ticks: tick 1 kills source and fires DamageDealt{target, 25}; tick 2
    // applies that damage (target hp 30 → 5, target survives).
    let destroyed_msgs = run_ticks_capture_destroyed(&mut app, 2);

    assert!(app.world().get_entity(source).is_err());
    assert!(
        app.world().get_entity(target).is_ok(),
        "target with hp 30 > damage 25 should still be present"
    );
    let hp = app
        .world()
        .get::<Hp>(target)
        .expect("target should still have Hp");
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "target hp should be 5.0 (30 - 25), got {}",
        hp.current
    );
    assert!(app.world().get::<Dead>(target).is_none());

    assert_eq!(destroyed_msgs.len(), 1);
    assert_eq!(destroyed_msgs[0].victim, source);
}

// Behavior 21
#[test]
fn volatile_detonation_does_not_damage_target_outside_radius() {
    let mut app = build_volatile_test_app();

    let source = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let target = spawn_plain_cell(&mut app, Vec2::new(50.0, 0.0), 25.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(source, 100.0)]));

    tick(&mut app);

    assert!(
        app.world().get_entity(target).is_ok(),
        "target at distance 50 > radius 40 should be unaffected"
    );
    let hp = app.world().get::<Hp>(target).unwrap();
    assert!((hp.current - 25.0).abs() < f32::EPSILON);
    assert!(app.world().get::<Dead>(target).is_none());

    let destroyed = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
    assert_eq!(destroyed.0.len(), 1);
    assert_eq!(destroyed.0[0].victim, source);
}

// Behavior 21 edge: boundary target at exactly radius distance is damaged
#[test]
fn volatile_detonation_damages_target_at_exact_radius_boundary() {
    let mut app = build_volatile_test_app();

    let source = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let target = spawn_plain_cell(&mut app, Vec2::new(40.0, 0.0), 25.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(source, 100.0)]));

    // 2 ticks: tick 1 kills source + fires explosion; tick 2 applies damage to target.
    let destroyed_msgs = run_ticks_capture_destroyed(&mut app, 2);

    assert!(
        app.world().get_entity(target).is_err(),
        "target at distance == radius should be damaged (inclusive) and killed within 2 ticks"
    );

    let victims: Vec<Entity> = destroyed_msgs.iter().map(|m| m.victim).collect();
    assert!(victims.contains(&source));
    assert!(victims.contains(&target));

    let target_msg = destroyed_msgs
        .iter()
        .find(|m| m.victim == target)
        .expect("target destroyed");
    assert_eq!(target_msg.victim_pos, Vec2::new(40.0, 0.0));
}

// Behavior 22
#[test]
fn volatile_detonation_spares_target_just_outside_radius() {
    let mut app = build_volatile_test_app();

    let source = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let target = spawn_plain_cell(&mut app, Vec2::new(40.001, 0.0), 25.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(source, 100.0)]));

    tick(&mut app);

    assert!(app.world().get_entity(target).is_ok());
    let hp = app.world().get::<Hp>(target).unwrap();
    assert!((hp.current - 25.0).abs() < f32::EPSILON);
    assert!(app.world().get::<Dead>(target).is_none());

    let destroyed = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
    assert_eq!(destroyed.0.len(), 1);
    assert_eq!(destroyed.0[0].victim, source);
}

// Behavior 22 edge: target just inside radius is killed
#[test]
fn volatile_detonation_kills_target_just_inside_radius() {
    let mut app = build_volatile_test_app();

    let source = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let target = spawn_plain_cell(&mut app, Vec2::new(39.999, 0.0), 25.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(source, 100.0)]));

    // 2 ticks: tick 1 kills source + fires explosion; tick 2 applies damage to target.
    let victims_vec = run_ticks_and_collect_destroyed(&mut app, 2);

    assert!(app.world().get_entity(target).is_err());

    let victims: std::collections::HashSet<Entity> = victims_vec.into_iter().collect();
    assert_eq!(victims, std::collections::HashSet::from([source, target]));
}

// Behavior 23
#[test]
fn volatile_detonation_damages_all_three_non_volatile_targets_within_radius() {
    let mut app = build_volatile_test_app();

    let source = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let t1 = spawn_plain_cell(&mut app, Vec2::new(10.0, 0.0), 25.0);
    let t2 = spawn_plain_cell(&mut app, Vec2::new(20.0, 0.0), 25.0);
    let t3 = spawn_plain_cell(&mut app, Vec2::new(-30.0, 0.0), 25.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(source, 100.0)]));

    // 2 ticks: tick 1 kills source + fires explosion; tick 2 applies damage to
    // all three targets and kills them.
    let destroyed_msgs = run_ticks_capture_destroyed(&mut app, 2);

    assert!(app.world().get_entity(t1).is_err());
    assert!(app.world().get_entity(t2).is_err());
    assert!(app.world().get_entity(t3).is_err());

    let victims: std::collections::HashSet<Entity> =
        destroyed_msgs.iter().map(|m| m.victim).collect();
    assert_eq!(
        victims,
        std::collections::HashSet::from([source, t1, t2, t3])
    );

    let source_msg = destroyed_msgs
        .iter()
        .find(|m| m.victim == source)
        .expect("source destroyed");
    assert_eq!(source_msg.victim_pos, Vec2::new(0.0, 0.0));
    let t1_msg = destroyed_msgs
        .iter()
        .find(|m| m.victim == t1)
        .expect("t1 destroyed");
    assert_eq!(t1_msg.victim_pos, Vec2::new(10.0, 0.0));
    let t2_msg = destroyed_msgs
        .iter()
        .find(|m| m.victim == t2)
        .expect("t2 destroyed");
    assert_eq!(t2_msg.victim_pos, Vec2::new(20.0, 0.0));
    let t3_msg = destroyed_msgs
        .iter()
        .find(|m| m.victim == t3)
        .expect("t3 destroyed");
    assert_eq!(t3_msg.victim_pos, Vec2::new(-30.0, 0.0));
}

// Behavior 23 edge: one target is Invulnerable
#[test]
fn volatile_detonation_respects_invulnerable_filter_on_targets() {
    let mut app = build_volatile_test_app();

    let source = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let t1 = spawn_plain_cell(&mut app, Vec2::new(10.0, 0.0), 25.0);
    let t2 = spawn_invulnerable_cell(&mut app, Vec2::new(20.0, 0.0), 25.0);
    let t3 = spawn_plain_cell(&mut app, Vec2::new(-30.0, 0.0), 25.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(source, 100.0)]));

    // 2 ticks: tick 1 kills source + fires explosion; tick 2 applies damage —
    // t1 and t3 die, t2 is filtered out by Invulnerable.
    let victims_vec = run_ticks_and_collect_destroyed(&mut app, 2);

    assert!(app.world().get_entity(t1).is_err());
    assert!(
        app.world().get_entity(t2).is_ok(),
        "invulnerable target should be spared"
    );
    let t2_hp = app.world().get::<Hp>(t2).unwrap();
    assert!((t2_hp.current - 25.0).abs() < f32::EPSILON);
    assert!(app.world().get::<Dead>(t2).is_none());
    assert!(app.world().get_entity(t3).is_err());

    let victims: std::collections::HashSet<Entity> = victims_vec.into_iter().collect();
    assert_eq!(victims, std::collections::HashSet::from([source, t1, t3]));
}

// Behavior 24
#[test]
fn volatile_detonation_with_no_other_cells_in_range_emits_no_extra_damage() {
    let mut app = build_volatile_test_app();

    let source = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(source, 100.0)]));

    tick(&mut app);

    let destroyed = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
    assert_eq!(destroyed.0.len(), 1, "only source should be destroyed");
    assert_eq!(destroyed.0[0].victim, source);

    let damage = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        damage.0.len(),
        1,
        "only the initial pending damage should have been delivered"
    );
}

// Behavior 24 edge: non-`Cell` entity inside the radius is ignored
#[test]
fn volatile_detonation_does_not_damage_non_cell_entities_in_range() {
    let mut app = build_volatile_test_app();

    let source = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let _bare = app.world_mut().spawn(Position2D(Vec2::new(10.0, 0.0))).id();

    app.insert_resource(PendingCellDamage(vec![damage_msg(source, 100.0)]));

    tick(&mut app);

    let damage = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        damage.0.len(),
        1,
        "non-cell entity inside radius must not receive DamageDealt<Cell>"
    );
}
