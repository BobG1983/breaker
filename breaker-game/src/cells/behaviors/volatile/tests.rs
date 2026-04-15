//! Groups D, E, F — integration tests for Volatile's end-to-end explosion at
//! death, chain reactions, and safety/idempotency claims.
//!
//! These tests wire the full effects + death pipeline via
//! `TestAppBuilder::with_effects_pipeline()` and drive damage into the pipeline
//! by duplicating the `PendingCellDamage` / `enqueue_cell_damage` helper pattern
//! from `breaker-game/src/shared/death_pipeline/systems/tests/helpers.rs`. The
//! duplication is intentional — `death_pipeline`'s helpers are `pub(super)` and
//! are not accessible from the cells domain. Per the Wave 1 test spec, we
//! duplicate rather than promote visibility.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    cells::behaviors::volatile::stamp::{STAMP_SOURCE, volatile_tree},
    effect_v3::EffectV3Systems,
    prelude::*,
    shared::death_pipeline::sets::DeathPipelineSystems,
};

// ── Local helpers (duplicated from death_pipeline::helpers::{PendingCellDamage,
// enqueue_cell_damage, ...}) ─────────────────────────────────────────────────

#[derive(Resource, Default)]
struct PendingCellDamage(Vec<DamageDealt<Cell>>);

/// Drains `PendingCellDamage` into the `DamageDealt<Cell>` message queue — one
/// shot per seeded damage, so subsequent ticks do not re-inject the same damage.
fn enqueue_cell_damage(
    mut pending: ResMut<PendingCellDamage>,
    mut writer: MessageWriter<DamageDealt<Cell>>,
) {
    for msg in pending.0.drain(..) {
        writer.write(msg);
    }
}

/// Injected `Destroyed<Cell>` messages (used by Group F behavior 28 to drive
/// the bridge directly without going through `handle_kill`).
#[derive(Resource, Default)]
struct TestCellDestroyedMessages(Vec<Destroyed<Cell>>);

fn inject_cell_destroyed(
    mut messages: ResMut<TestCellDestroyedMessages>,
    mut writer: MessageWriter<Destroyed<Cell>>,
) {
    for msg in messages.0.drain(..) {
        writer.write(msg);
    }
}

/// Builds a volatile cell entity directly (no builder sugar) at `pos` with
/// `hp` hit points and a pre-stamped `BoundEffects` entry that will detonate
/// on death with `(damage, radius)`.
fn spawn_volatile_cell(app: &mut App, pos: Vec2, damage: f32, radius: f32, hp: f32) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Hp::new(hp),
            KilledBy::default(),
            Position2D(pos),
            BoundEffects(vec![(
                STAMP_SOURCE.to_owned(),
                volatile_tree(damage, radius),
            )]),
        ))
        .id()
}

/// Spawns a plain (non-volatile) cell with `hp` hit points at `pos`.
fn spawn_plain_cell(app: &mut App, pos: Vec2, hp: f32) -> Entity {
    app.world_mut()
        .spawn((Cell, Hp::new(hp), KilledBy::default(), Position2D(pos)))
        .id()
}

fn damage_msg(target: Entity, amount: f32) -> DamageDealt<Cell> {
    DamageDealt {
        dealer: None,
        target,
        amount,
        source_chip: None,
        _marker: PhantomData,
    }
}

/// Builds a plugin-integration `App` with volatile wiring and the given
/// pending-damage seeds. Collectors for `Destroyed<Cell>` and `DamageDealt<Cell>`
/// are attached.
fn build_volatile_test_app() -> App {
    let mut app = TestAppBuilder::new().with_effects_pipeline().build();
    attach_message_capture::<Destroyed<Cell>>(&mut app);
    attach_message_capture::<DamageDealt<Cell>>(&mut app);
    app.init_resource::<PendingCellDamage>();
    app.add_systems(
        FixedUpdate,
        enqueue_cell_damage.before(DeathPipelineSystems::ApplyDamage),
    );
    app
}

/// Runs N ticks and returns the set of `Destroyed<Cell>.victim` values seen
/// across all ticks. This works around `MessageCollector`'s per-tick clear by
/// reading the collector between ticks.
fn run_ticks_and_collect_destroyed(app: &mut App, ticks: usize) -> Vec<Entity> {
    let mut out: Vec<Entity> = Vec::new();
    for _ in 0..ticks {
        tick(app);
        let destroyed = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
        for msg in &destroyed.0 {
            out.push(msg.victim);
        }
    }
    out
}

/// Runs N ticks and returns every `Destroyed<Cell>` message seen across all
/// ticks (full message — includes `victim`, `victim_pos`, etc). Works around
/// `MessageCollector`'s per-tick clear by cloning the collector between ticks.
fn run_ticks_capture_destroyed(app: &mut App, ticks: usize) -> Vec<Destroyed<Cell>> {
    let mut out: Vec<Destroyed<Cell>> = Vec::new();
    for _ in 0..ticks {
        tick(app);
        let destroyed = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
        out.extend(destroyed.0.iter().cloned());
    }
    out
}

/// Runs N ticks and returns every `Destroyed<Cell>` and `DamageDealt<Cell>`
/// message seen across all ticks. Works around `MessageCollector`'s per-tick
/// clear by cloning both collectors between ticks.
fn run_ticks_capture_destroyed_and_damage(
    app: &mut App,
    ticks: usize,
) -> (Vec<Destroyed<Cell>>, Vec<DamageDealt<Cell>>) {
    let mut destroyed_out: Vec<Destroyed<Cell>> = Vec::new();
    let mut damage_out: Vec<DamageDealt<Cell>> = Vec::new();
    for _ in 0..ticks {
        tick(app);
        let destroyed = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
        destroyed_out.extend(destroyed.0.iter().cloned());
        let damage = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        damage_out.extend(damage.0.iter().cloned());
    }
    (destroyed_out, damage_out)
}

// ── Group D: End-to-end explosion at death ──────────────────────────────────

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
    let t2 = app
        .world_mut()
        .spawn((
            Cell,
            Hp::new(25.0),
            KilledBy::default(),
            Position2D(Vec2::new(20.0, 0.0)),
            Invulnerable,
        ))
        .id();
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

// ── Group E: Chain reactions ────────────────────────────────────────────────

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

// ── Group F: Safety and idempotency ─────────────────────────────────────────

// Behavior 28
#[test]
fn volatile_cell_already_dead_still_fires_explosion_exactly_once() {
    let mut app = TestAppBuilder::new().with_effects_pipeline().build();
    attach_message_capture::<Destroyed<Cell>>(&mut app);
    attach_message_capture::<DamageDealt<Cell>>(&mut app);
    app.init_resource::<TestCellDestroyedMessages>();
    app.add_systems(
        FixedUpdate,
        inject_cell_destroyed.before(EffectV3Systems::Bridge),
    );

    let source = app
        .world_mut()
        .spawn((
            Cell,
            Dead,
            Position2D(Vec2::new(0.0, 0.0)),
            Hp::new(0.0),
            KilledBy::default(),
            BoundEffects(vec![(STAMP_SOURCE.to_owned(), volatile_tree(25.0, 40.0))]),
        ))
        .id();
    let target = app
        .world_mut()
        .spawn((
            Cell,
            Hp::new(10.0),
            KilledBy::default(),
            Position2D(Vec2::new(20.0, 0.0)),
        ))
        .id();

    app.insert_resource(TestCellDestroyedMessages(vec![Destroyed::<Cell> {
        victim:     source,
        killer:     None,
        victim_pos: Vec2::new(0.0, 0.0),
        killer_pos: None,
        _marker:    PhantomData,
    }]));

    tick(&mut app);

    let damage = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    let target_hits = damage
        .0
        .iter()
        .filter(|m| m.target == target && (m.amount - 25.0).abs() < f32::EPSILON)
        .count();
    assert_eq!(
        target_hits, 1,
        "the explosion must fire exactly once and deliver 25.0 to target"
    );

    assert!(
        app.world().get_entity(source).is_ok(),
        "source was never passed through handle_kill — it should still be present"
    );
}

// Behavior 28 edge: injecting the same Destroyed<Cell> twice => explosion fires twice
#[test]
fn injecting_destroyed_twice_fires_explosion_twice() {
    let mut app = TestAppBuilder::new().with_effects_pipeline().build();
    attach_message_capture::<DamageDealt<Cell>>(&mut app);
    app.init_resource::<TestCellDestroyedMessages>();
    app.add_systems(
        FixedUpdate,
        inject_cell_destroyed.before(EffectV3Systems::Bridge),
    );

    let source = app
        .world_mut()
        .spawn((
            Cell,
            Dead,
            Position2D(Vec2::new(0.0, 0.0)),
            Hp::new(0.0),
            KilledBy::default(),
            BoundEffects(vec![(STAMP_SOURCE.to_owned(), volatile_tree(25.0, 40.0))]),
        ))
        .id();
    let target = app
        .world_mut()
        .spawn((
            Cell,
            Hp::new(10.0),
            KilledBy::default(),
            Position2D(Vec2::new(20.0, 0.0)),
        ))
        .id();

    let msg = Destroyed::<Cell> {
        victim:     source,
        killer:     None,
        victim_pos: Vec2::new(0.0, 0.0),
        killer_pos: None,
        _marker:    PhantomData,
    };
    app.insert_resource(TestCellDestroyedMessages(vec![msg.clone(), msg]));

    tick(&mut app);

    let damage = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    let target_hits = damage.0.iter().filter(|m| m.target == target).count();
    assert_eq!(
        target_hits, 2,
        "two injected Destroyed<Cell> messages should drive the bridge twice"
    );
}

// Behavior 29
#[test]
fn volatile_detonation_skips_dead_targets_inside_radius() {
    let mut app = build_volatile_test_app();

    let source = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let target_live = spawn_plain_cell(&mut app, Vec2::new(20.0, 0.0), 25.0);
    let target_dead = app
        .world_mut()
        .spawn((
            Cell,
            Hp::new(25.0),
            KilledBy::default(),
            Position2D(Vec2::new(-20.0, 0.0)),
            Dead,
        ))
        .id();

    app.insert_resource(PendingCellDamage(vec![damage_msg(source, 100.0)]));

    // 2 ticks: tick 1 kills source + fires explosion (target_dead is filtered);
    // tick 2 applies damage to target_live and kills it.
    let (destroyed_msgs, damage_msgs) = run_ticks_capture_destroyed_and_damage(&mut app, 2);

    assert!(app.world().get_entity(target_live).is_err());
    assert!(
        app.world().get_entity(target_dead).is_ok(),
        "dead target spawned with Dead should still be present"
    );
    let dead_hp = app.world().get::<Hp>(target_dead).unwrap();
    assert!((dead_hp.current - 25.0).abs() < f32::EPSILON);

    assert_eq!(
        damage_msgs.len(),
        2,
        "expected pending(100) + explosion_target_live(25) — dead target is filtered"
    );

    let victims: std::collections::HashSet<Entity> =
        destroyed_msgs.iter().map(|m| m.victim).collect();
    assert_eq!(
        victims,
        std::collections::HashSet::from([source, target_live])
    );
}

// Behavior 29 edge: second dead target at different position — also filtered
#[test]
fn volatile_detonation_skips_multiple_dead_targets_inside_radius() {
    let mut app = build_volatile_test_app();

    let source = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let target_live = spawn_plain_cell(&mut app, Vec2::new(20.0, 0.0), 25.0);
    let target_dead = app
        .world_mut()
        .spawn((
            Cell,
            Hp::new(25.0),
            KilledBy::default(),
            Position2D(Vec2::new(-20.0, 0.0)),
            Dead,
        ))
        .id();
    let target_dead2 = app
        .world_mut()
        .spawn((
            Cell,
            Hp::new(25.0),
            KilledBy::default(),
            Position2D(Vec2::new(10.0, 10.0)),
            Dead,
        ))
        .id();

    app.insert_resource(PendingCellDamage(vec![damage_msg(source, 100.0)]));

    // 2 ticks: tick 1 kills source + fires explosion (both dead targets are
    // filtered); tick 2 applies damage to target_live and kills it.
    let (destroyed_msgs, damage_msgs) = run_ticks_capture_destroyed_and_damage(&mut app, 2);

    assert!(app.world().get_entity(target_live).is_err());
    assert!(app.world().get_entity(target_dead).is_ok());
    assert!(app.world().get_entity(target_dead2).is_ok());
    assert!((app.world().get::<Hp>(target_dead).unwrap().current - 25.0).abs() < f32::EPSILON);
    assert!((app.world().get::<Hp>(target_dead2).unwrap().current - 25.0).abs() < f32::EPSILON);

    assert_eq!(
        damage_msgs.len(),
        2,
        "still exactly pending(100) + explosion(25) — both dead targets filtered"
    );

    let victims: std::collections::HashSet<Entity> =
        destroyed_msgs.iter().map(|m| m.victim).collect();
    assert_eq!(
        victims,
        std::collections::HashSet::from([source, target_live])
    );
}

// Behavior 30
#[test]
fn volatile_small_radius_does_not_damage_target_outside() {
    let mut app = build_volatile_test_app();

    let source = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 0.001, 0.001, 10.0);
    let target = spawn_plain_cell(&mut app, Vec2::new(0.5, 0.0), 25.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(source, 100.0)]));

    tick(&mut app);

    assert!(app.world().get_entity(target).is_ok());
    let hp = app.world().get::<Hp>(target).unwrap();
    assert!((hp.current - 25.0).abs() < f32::EPSILON);

    let destroyed = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
    assert_eq!(destroyed.0.len(), 1);
    assert_eq!(destroyed.0[0].victim, source);
}

// Behavior 30 edge: target just inside the 0.001 radius takes 0.001 damage
#[test]
fn volatile_small_radius_damages_target_just_inside() {
    let mut app = build_volatile_test_app();

    let source = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 0.001, 0.001, 10.0);
    let target = spawn_plain_cell(&mut app, Vec2::new(0.0005, 0.0), 25.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(source, 100.0)]));

    // 2 ticks: tick 1 kills source + fires tiny explosion; tick 2 applies
    // 0.001 damage to target (target survives).
    tick(&mut app);
    tick(&mut app);

    assert!(
        app.world().get_entity(target).is_ok(),
        "target takes 0.001 damage but should not die (hp 25 > damage 0.001)"
    );
    let hp = app.world().get::<Hp>(target).unwrap();
    assert!(
        (hp.current - 24.999).abs() < 25.0f32.mul_add(f32::EPSILON, 1e-5),
        "target hp should be 24.999 within tolerance, got {}",
        hp.current
    );
}
