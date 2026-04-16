//! Part G — `kill_bump_vulnerable_cells` system tests (behaviors 47-54).

use bevy::prelude::*;

use super::helpers::*;
use crate::prelude::*;

// ── Behavior 47: BumpVulnerable cell: breaker contact writes lethal DamageDealt ──

#[test]
fn bump_vulnerable_cell_breaker_contact_kills() {
    let mut app = build_bump_vulnerable_test_app();

    let cell = spawn_bolt_immune_cell(&mut app, 20.0); // has BumpVulnerable
    let breaker = spawn_test_breaker(&mut app);

    advance_to_playing(&mut app);

    push_breaker_impact(&mut app, breaker_impact(breaker, cell));

    // Multiple ticks for full death pipeline
    tick(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell);
    let is_dead = hp.is_none_or(|h| h.current <= 0.0)
        || app.world().get::<Dead>(cell).is_some()
        || app.world().get_entity(cell).is_err();
    assert!(
        is_dead,
        "BumpVulnerable cell should be killed by breaker contact"
    );
}

// Behavior 47 edge: cell with very low HP
#[test]
fn bump_vulnerable_cell_with_tiny_hp_killed() {
    let mut app = build_bump_vulnerable_test_app();

    let cell = {
        let world = app.world_mut();
        crate::cells::test_utils::spawn_cell_in_world(world, |commands| {
            Cell::builder()
                .survival(crate::cells::definition::AttackPattern::StraightDown, 10.0)
                .position(Vec2::ZERO)
                .dimensions(TEST_CELL_DIM, TEST_CELL_DIM)
                .hp(0.001)
                .headless()
                .spawn(commands)
        })
    };
    let breaker = spawn_test_breaker(&mut app);

    advance_to_playing(&mut app);

    push_breaker_impact(&mut app, breaker_impact(breaker, cell));

    tick(&mut app);
    tick(&mut app);

    let is_dead = app.world().get_entity(cell).is_err()
        || app.world().get::<Dead>(cell).is_some()
        || app.world().get::<Hp>(cell).is_none_or(|h| h.current <= 0.0);
    assert!(
        is_dead,
        "BumpVulnerable cell with 0.001 HP should be killed"
    );
}

// ── Behavior 48: Non-BumpVulnerable cell: breaker contact writes no damage ──

#[test]
fn non_bump_vulnerable_cell_breaker_contact_no_damage() {
    let mut app = build_bump_vulnerable_test_app();

    let cell = spawn_plain_cell(&mut app, 20.0); // no BumpVulnerable
    let breaker = spawn_test_breaker(&mut app);

    advance_to_playing(&mut app);

    push_breaker_impact(&mut app, breaker_impact(breaker, cell));

    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "non-BumpVulnerable cell should take no damage, got hp.current == {}",
        hp.current
    );
}

// ── Behavior 49: BumpVulnerable cell already Dead: no duplicate damage ──

#[test]
fn bump_vulnerable_dead_cell_no_duplicate_damage() {
    let mut app = build_bump_vulnerable_test_app();

    let cell = spawn_bolt_immune_cell(&mut app, 20.0);
    // Pre-insert Dead marker
    app.world_mut().entity_mut(cell).insert(Dead);
    let breaker = spawn_test_breaker(&mut app);

    advance_to_playing(&mut app);

    push_breaker_impact(&mut app, breaker_impact(breaker, cell));

    tick(&mut app);

    // Cell should still have the same HP (Dead filter prevents the system
    // from writing damage, but even if it did, apply_damage skips Dead).
    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "Dead BumpVulnerable cell should not receive duplicate damage, got hp.current == {}",
        hp.current
    );
}

// ── Behavior 50: BumpVulnerable + Invulnerable cell: damage written but absorbed ──

#[test]
fn bump_vulnerable_invulnerable_cell_damage_absorbed() {
    let mut app = build_bump_vulnerable_test_app();

    let cell = spawn_bolt_immune_cell(&mut app, 20.0);
    // Add Invulnerable marker (e.g., Locked cell)
    app.world_mut().entity_mut(cell).insert(Invulnerable);
    let breaker = spawn_test_breaker(&mut app);

    advance_to_playing(&mut app);

    push_breaker_impact(&mut app, breaker_impact(breaker, cell));

    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "Invulnerable cell should absorb damage, got hp.current == {}",
        hp.current
    );
}

// ── Behavior 51: Multiple breaker impacts on same BumpVulnerable cell in one tick ──

#[test]
fn bump_vulnerable_cell_multiple_breaker_impacts_same_tick() {
    let mut app = build_bump_vulnerable_test_app();

    let cell = spawn_bolt_immune_cell(&mut app, 20.0);
    let breaker1 = spawn_test_breaker(&mut app);
    let breaker2 = spawn_test_breaker(&mut app);

    advance_to_playing(&mut app);

    push_breaker_impact(&mut app, breaker_impact(breaker1, cell));
    push_breaker_impact(&mut app, breaker_impact(breaker2, cell));

    tick(&mut app);
    tick(&mut app);

    let is_dead = app.world().get_entity(cell).is_err()
        || app.world().get::<Dead>(cell).is_some()
        || app.world().get::<Hp>(cell).is_none_or(|h| h.current <= 0.0);
    assert!(
        is_dead,
        "BumpVulnerable cell with multiple impacts should be killed"
    );
}

// ── Behavior 52: Mixed: BumpVulnerable cell + non-vulnerable cell both contacted ──

#[test]
fn mixed_bump_vulnerable_and_non_vulnerable_cells() {
    let mut app = build_bump_vulnerable_test_app();

    let cell_a = spawn_bolt_immune_cell(&mut app, 20.0); // BumpVulnerable
    let cell_b = spawn_plain_cell(&mut app, 20.0); // no BumpVulnerable
    let breaker = spawn_test_breaker(&mut app);

    advance_to_playing(&mut app);

    push_breaker_impact(&mut app, breaker_impact(breaker, cell_a));
    push_breaker_impact(&mut app, breaker_impact(breaker, cell_b));

    tick(&mut app);
    tick(&mut app);

    let a_dead = app.world().get_entity(cell_a).is_err()
        || app.world().get::<Dead>(cell_a).is_some()
        || app
            .world()
            .get::<Hp>(cell_a)
            .is_none_or(|h| h.current <= 0.0);
    assert!(a_dead, "BumpVulnerable cell A should be killed");

    let hp_b = app
        .world()
        .get::<Hp>(cell_b)
        .expect("cell B should have Hp");
    assert!(
        (hp_b.current - 20.0).abs() < f32::EPSILON,
        "non-BumpVulnerable cell B should be unharmed, got hp.current == {}",
        hp_b.current
    );
}

// ── Behavior 53: No BreakerImpactCell messages: system is a no-op ──

#[test]
fn kill_bump_vulnerable_cells_no_impacts_no_op() {
    let mut app = build_bump_vulnerable_test_app();

    let cell = spawn_bolt_immune_cell(&mut app, 20.0);

    advance_to_playing(&mut app);
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "no impacts means no damage, got hp.current == {}",
        hp.current
    );
}

// ── Behavior 54: kill_bump_vulnerable_cells damage flows through full death pipeline ──

#[test]
fn bump_vulnerable_cell_death_flows_through_pipeline() {
    let mut app = build_bump_vulnerable_test_app();

    let cell = {
        let world = app.world_mut();
        crate::cells::test_utils::spawn_cell_in_world(world, |commands| {
            Cell::builder()
                .survival(crate::cells::definition::AttackPattern::StraightDown, 10.0)
                .required_to_clear(true)
                .position(Vec2::ZERO)
                .dimensions(TEST_CELL_DIM, TEST_CELL_DIM)
                .hp(20.0)
                .headless()
                .spawn(commands)
        })
    };
    let breaker = spawn_test_breaker(&mut app);

    advance_to_playing(&mut app);

    push_breaker_impact(&mut app, breaker_impact(breaker, cell));

    // Multiple ticks for full pipeline: write damage -> apply -> detect -> handle -> despawn
    // Accumulate Destroyed<Cell> per tick (collector clears at First each tick).
    let mut destroyed_msgs: Vec<Destroyed<Cell>> = Vec::new();
    for _ in 0..5 {
        tick(&mut app);
        let collected = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
        destroyed_msgs.extend(collected.0.iter().cloned());
    }

    let is_dead_or_gone =
        app.world().get_entity(cell).is_err() || app.world().get::<Dead>(cell).is_some();
    assert!(
        is_dead_or_gone,
        "BumpVulnerable cell should be dead after full death pipeline"
    );

    assert!(
        !destroyed_msgs.is_empty(),
        "Destroyed<Cell> message should be emitted for killed BumpVulnerable cell"
    );
}
