//! Tests for `salvo_cell_collision` — behaviors 24-28.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    cells::behaviors::survival::{components::SurvivalTurret, salvo::components::Salvo},
    prelude::*,
};

// ── Behavior 24: Salvo overlapping a cell writes DamageDealt<Cell> ──

#[test]
fn salvo_overlapping_cell_writes_damage() {
    let mut app = build_salvo_cell_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    let salvo = spawn_salvo(
        &mut app,
        Vec2::new(100.0, 150.0),
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );
    let cell = spawn_collision_cell(&mut app, Vec2::new(100.0, 150.0), Vec2::new(20.0, 10.0));

    advance_to_playing(&mut app);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    let msgs: Vec<_> = collector.0.iter().filter(|m| m.target == cell).collect();

    assert!(
        !msgs.is_empty(),
        "overlapping salvo should write DamageDealt<Cell>"
    );
    assert!(
        (msgs[0].amount - 5.0).abs() < f32::EPSILON,
        "damage amount should be 5.0, got {}",
        msgs[0].amount
    );
    assert_eq!(
        msgs[0].dealer,
        Some(salvo),
        "dealer should be the salvo entity"
    );
}

// Behavior 24 edge: SalvoDamage(0.0) still writes message
#[test]
fn salvo_zero_damage_still_writes_message() {
    let mut app = build_salvo_cell_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    let _salvo = spawn_salvo(
        &mut app,
        Vec2::new(100.0, 150.0),
        Vec2::new(0.0, -300.0),
        0.0, // zero damage
        turret,
    );
    let cell = spawn_collision_cell(&mut app, Vec2::new(100.0, 150.0), Vec2::new(20.0, 10.0));

    advance_to_playing(&mut app);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    let msgs: Vec<_> = collector.0.iter().filter(|m| m.target == cell).collect();

    assert!(
        !msgs.is_empty(),
        "zero-damage salvo should still write DamageDealt<Cell>"
    );
    assert!(
        msgs[0].amount.abs() < f32::EPSILON,
        "damage amount should be 0.0, got {}",
        msgs[0].amount
    );
}

// ── Behavior 25: Salvo is NOT consumed by cell collision (passes through) ──

#[test]
fn salvo_passes_through_cell_not_despawned() {
    let mut app = build_salvo_cell_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    let salvo = spawn_salvo(
        &mut app,
        Vec2::new(100.0, 150.0),
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );
    let _cell = spawn_collision_cell(&mut app, Vec2::new(100.0, 150.0), Vec2::new(20.0, 10.0));

    advance_to_playing(&mut app);
    tick(&mut app);

    assert!(
        app.world().get_entity(salvo).is_ok(),
        "salvo should still exist after cell collision (passes through)"
    );
    assert!(
        app.world().get::<Salvo>(salvo).is_some(),
        "salvo should still have Salvo marker"
    );
}

// Behavior 25 edge: salvo overlaps 3 cells, writes 3 messages, still alive
#[test]
fn salvo_overlapping_three_cells_writes_three_messages() {
    let mut app = build_salvo_cell_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    let salvo = spawn_salvo(
        &mut app,
        Vec2::new(100.0, 150.0),
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );
    let cell1 = spawn_collision_cell(&mut app, Vec2::new(100.0, 150.0), Vec2::new(20.0, 10.0));
    let cell2 = spawn_collision_cell(&mut app, Vec2::new(100.0, 148.0), Vec2::new(20.0, 10.0));
    let cell3 = spawn_collision_cell(&mut app, Vec2::new(100.0, 146.0), Vec2::new(20.0, 10.0));

    advance_to_playing(&mut app);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    let targets: Vec<Entity> = collector.0.iter().map(|m| m.target).collect();

    assert!(targets.contains(&cell1), "should write damage for cell1");
    assert!(targets.contains(&cell2), "should write damage for cell2");
    assert!(targets.contains(&cell3), "should write damage for cell3");

    assert!(
        app.world().get_entity(salvo).is_ok(),
        "salvo should still exist after hitting 3 cells"
    );
}

// ── Behavior 26: Salvo not overlapping any cell writes no damage ──

#[test]
fn salvo_not_overlapping_cell_writes_no_damage() {
    let mut app = build_salvo_cell_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    let _salvo = spawn_salvo(
        &mut app,
        Vec2::new(100.0, 300.0), // far from cell
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );
    let _cell = spawn_collision_cell(&mut app, Vec2::new(100.0, 150.0), Vec2::new(20.0, 10.0));

    advance_to_playing(&mut app);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert!(
        collector.0.is_empty(),
        "no DamageDealt<Cell> should be written when salvo and cell don't overlap"
    );
}

// ── Behavior 27: Salvo does not damage its own source turret ──

#[test]
fn salvo_does_not_damage_source_turret() {
    let mut app = build_salvo_cell_collision_app();

    // Spawn a turret cell
    let turret = spawn_collision_cell(&mut app, Vec2::new(50.0, 200.0), Vec2::new(20.0, 10.0));
    // Make it a survival turret
    app.world_mut().entity_mut(turret).insert(SurvivalTurret);

    // Spawn another cell that overlaps
    let other_cell = spawn_collision_cell(&mut app, Vec2::new(50.0, 195.0), Vec2::new(20.0, 10.0));

    // Spawn a salvo from the turret, overlapping both the turret and the other cell
    let _salvo = spawn_salvo(
        &mut app,
        Vec2::new(50.0, 197.0), // overlaps both cells
        Vec2::new(0.0, -300.0),
        5.0,
        turret, // source is the turret
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();

    // Should not have damage for the source turret
    assert!(
        collector.0.iter().all(|m| m.target != turret),
        "salvo should not damage its own source turret"
    );

    // Should have damage for the other cell
    assert!(
        collector.0.iter().any(|m| m.target == other_cell),
        "salvo should damage other cells it overlaps"
    );
}

// ── Behavior 28: Empty world: system is a no-op ──

#[test]
fn no_salvos_no_cells_no_crash() {
    let mut app = build_salvo_cell_collision_app();

    advance_to_playing(&mut app);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert!(collector.0.is_empty(), "no messages in empty world");
}

#[test]
fn salvos_exist_but_no_cells_no_crash() {
    let mut app = build_salvo_cell_collision_app();

    let turret = app.world_mut().spawn_empty().id();
    let _salvo = spawn_salvo(
        &mut app,
        Vec2::new(100.0, 150.0),
        Vec2::new(0.0, -300.0),
        5.0,
        turret,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert!(
        collector.0.is_empty(),
        "no damage when salvos exist but no cells"
    );
}
