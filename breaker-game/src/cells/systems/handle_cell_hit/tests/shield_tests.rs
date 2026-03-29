use bevy::prelude::*;

use super::helpers::*;
use crate::{
    cells::{components::CellHealth, messages::DamageCell},
    effect::effects::shield::ShieldActive,
};

// =========================================================================
// Wave 4B: Shield Protection — handle_cell_hit shield immunity
// =========================================================================

#[test]
fn shielded_cell_ignores_damage_cell() {
    // Behavior 8: Cell with ShieldActive ignores DamageCell message.
    // Given: Cell with CellHealth::new(10.0) and ShieldActive { charges: 3 }.
    //        DamageCell { cell, damage: 10.0, source_chip: None }.
    // When: handle_cell_hit runs
    // Then: Cell health remains 10.0. No RequestCellDestroyed.
    let mut app = test_app();
    let cell = spawn_shielded_cell(&mut app, 10.0);

    app.init_resource::<CapturedDestroyed>();
    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 10.0,
        source_chip: None,
    })));
    app.add_systems(
        FixedUpdate,
        (
            enqueue_from_resource.before(handle_cell_hit),
            capture_destroyed.after(handle_cell_hit),
        ),
    );
    tick(&mut app);

    assert!(
        app.world().get_entity(cell).is_ok(),
        "shielded cell should not be despawned"
    );
    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON,
        "shielded cell HP should remain 10.0, got {}",
        health.current
    );
    let captured = app.world().resource::<CapturedDestroyed>();
    assert_eq!(
        captured.0.len(),
        0,
        "shielded cell should not produce RequestCellDestroyed"
    );
}

#[test]
fn shielded_cell_ignores_massive_overkill_damage() {
    // Behavior 8 edge case: DamageCell with damage 999.0 (massive overkill).
    // Still ignored because ShieldActive is present.
    let mut app = test_app();
    let cell = spawn_shielded_cell(&mut app, 10.0);

    app.init_resource::<CapturedDestroyed>();
    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 999.0,
        source_chip: None,
    })));
    app.add_systems(
        FixedUpdate,
        (
            enqueue_from_resource.before(handle_cell_hit),
            capture_destroyed.after(handle_cell_hit),
        ),
    );
    tick(&mut app);

    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON,
        "shielded cell should ignore 999.0 overkill damage, HP should be 10.0, got {}",
        health.current
    );
    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(
        captured.0.is_empty(),
        "shielded cell should not produce RequestCellDestroyed even with overkill damage"
    );
}

#[test]
fn shielded_cell_skipped_but_unshielded_cell_takes_damage() {
    // Behavior 10: ShieldActive cell skipped but other cells still take damage.
    // Given: Cell A with ShieldActive. Cell B without ShieldActive.
    //        DamageCell for each (damage 10.0).
    // When: handle_cell_hit runs
    // Then: Cell A health 10.0 (shielded). Cell B health 20.0 (30 - 10 damage).
    let mut app = test_app();
    let cell_a = spawn_shielded_cell(&mut app, 10.0);
    let cell_b = spawn_cell(&mut app, 30.0);

    app.init_resource::<CapturedDestroyed>();
    app.init_resource::<TestMessages>();
    app.world_mut().resource_mut::<TestMessages>().0 = vec![
        DamageCell {
            cell: cell_a,
            damage: 10.0,
            source_chip: None,
        },
        DamageCell {
            cell: cell_b,
            damage: 10.0,
            source_chip: None,
        },
    ];
    app.add_systems(
        FixedUpdate,
        (
            enqueue_all.before(handle_cell_hit),
            capture_destroyed.after(handle_cell_hit),
        ),
    );
    tick(&mut app);

    let health_a = app.world().get::<CellHealth>(cell_a).unwrap();
    assert!(
        (health_a.current - 10.0).abs() < f32::EPSILON,
        "shielded cell A HP should remain 10.0, got {}",
        health_a.current
    );

    let health_b = app.world().get::<CellHealth>(cell_b).unwrap();
    assert!(
        (health_b.current - 20.0).abs() < f32::EPSILON,
        "unshielded cell B HP should be 30.0 - 10.0 = 20.0, got {}",
        health_b.current
    );

    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(
        captured.0.is_empty(),
        "neither cell should produce RequestCellDestroyed"
    );
}

#[test]
fn both_shielded_cells_immune() {
    // Behavior 10 edge case: Cell B has ShieldActive too — both cells are immune.
    let mut app = test_app();
    let cell_a = spawn_shielded_cell(&mut app, 10.0);
    let cell_b = spawn_shielded_cell(&mut app, 30.0);

    app.init_resource::<CapturedDestroyed>();
    app.init_resource::<TestMessages>();
    app.world_mut().resource_mut::<TestMessages>().0 = vec![
        DamageCell {
            cell: cell_a,
            damage: 10.0,
            source_chip: None,
        },
        DamageCell {
            cell: cell_b,
            damage: 10.0,
            source_chip: None,
        },
    ];
    app.add_systems(
        FixedUpdate,
        (
            enqueue_all.before(handle_cell_hit),
            capture_destroyed.after(handle_cell_hit),
        ),
    );
    tick(&mut app);

    let health_a = app.world().get::<CellHealth>(cell_a).unwrap();
    assert!(
        (health_a.current - 10.0).abs() < f32::EPSILON,
        "both-shielded cell A HP should remain 10.0, got {}",
        health_a.current
    );
    let health_b = app.world().get::<CellHealth>(cell_b).unwrap();
    assert!(
        (health_b.current - 30.0).abs() < f32::EPSILON,
        "both-shielded cell B HP should remain 30.0, got {}",
        health_b.current
    );
}

#[test]
fn locked_and_shielded_cell_both_immune() {
    // Behavior 11: Cell with BOTH Locked AND ShieldActive is immune to damage.
    // Given: Cell with CellHealth::new(10.0), Locked component, AND
    //        ShieldActive { charges: 1 }.
    //        DamageCell { cell, damage: 10.0, source_chip: None }.
    // When: handle_cell_hit runs
    // Then: Cell health remains 10.0. No RequestCellDestroyed.
    // Note: This test will PASS at RED because the existing Locked guard catches first.
    //       It serves as a regression guard ensuring shield immunity is independent of Locked.
    let mut app = test_app();
    let cell = spawn_locked_cell(&mut app, 10.0);
    // Add ShieldActive on top of Locked
    app.world_mut()
        .entity_mut(cell)
        .insert(ShieldActive { charges: 1 });

    app.init_resource::<CapturedDestroyed>();
    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 10.0,
        source_chip: None,
    })));
    app.add_systems(
        FixedUpdate,
        (
            enqueue_from_resource.before(handle_cell_hit),
            capture_destroyed.after(handle_cell_hit),
        ),
    );
    tick(&mut app);

    assert!(
        app.world().get_entity(cell).is_ok(),
        "locked+shielded cell should not be despawned"
    );
    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON,
        "locked+shielded cell HP should remain 10.0, got {}",
        health.current
    );
    let captured = app.world().resource::<CapturedDestroyed>();
    assert_eq!(
        captured.0.len(),
        0,
        "locked+shielded cell should not produce RequestCellDestroyed"
    );
}

#[test]
fn shielded_cell_dedup_both_messages_ignored() {
    // Behavior 12: ShieldActive priority relative to dedup tracking.
    // Given: Cell with ShieldActive. Two DamageCell messages for same cell.
    // When: handle_cell_hit runs
    // Then: Cell health 10.0. No RequestCellDestroyed.
    let mut app = test_app();
    let cell = spawn_shielded_cell(&mut app, 10.0);

    app.init_resource::<CapturedDestroyed>();
    app.init_resource::<TestMessages>();
    app.world_mut().resource_mut::<TestMessages>().0 = vec![
        DamageCell {
            cell,
            damage: 10.0,
            source_chip: None,
        },
        DamageCell {
            cell,
            damage: 10.0,
            source_chip: None,
        },
    ];
    app.add_systems(
        FixedUpdate,
        (
            enqueue_all.before(handle_cell_hit),
            capture_destroyed.after(handle_cell_hit),
        ),
    );
    tick(&mut app);

    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON,
        "shielded cell with two DamageCell should still have 10.0 HP, got {}",
        health.current
    );
    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(
        captured.0.is_empty(),
        "shielded cell should not produce any RequestCellDestroyed messages"
    );
}
