//! Behaviors 1-2: Shielded cell absorbs single hit, overkill, damage amount irrelevance.

use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    cells::{components::CellHealth, messages::DamageCell},
    effect::effects::shield::ShieldActive,
};

// ── Behavior 1: Shielded cell absorbs damage and decrements charges by 1 ──

#[test]
fn shielded_cell_absorbs_damage_and_decrements_one_charge() {
    // Given: Cell with CellHealth::new(10.0) and ShieldActive { charges: 3 }.
    //        DamageCell { cell, damage: 10.0, source_chip: None }.
    // When: handle_cell_hit runs
    // Then: Cell health remains 10.0. No RequestCellDestroyed. ShieldActive.charges is 2.
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

    // Health should remain unchanged (damage absorbed)
    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON,
        "shielded cell HP should remain 10.0, got {}",
        health.current
    );

    // No destruction
    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(
        captured.0.is_empty(),
        "shielded cell should not produce RequestCellDestroyed"
    );

    // Charges decremented from 3 to 2
    let shield = app.world().get::<ShieldActive>(cell).unwrap();
    assert_eq!(
        shield.charges, 2,
        "shield charges should decrement from 3 to 2 after absorbing one hit, got {}",
        shield.charges
    );
}

#[test]
fn shielded_cell_absorbs_massive_overkill_and_decrements_one_charge() {
    // Behavior 1 edge case: DamageCell with damage 999.0 (massive overkill).
    // Shield absorbs the hit regardless of damage amount.
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
        "shielded cell should absorb 999.0 overkill damage, HP should be 10.0, got {}",
        health.current
    );
    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(
        captured.0.is_empty(),
        "shielded cell should not produce RequestCellDestroyed even with overkill damage"
    );

    // Charges decremented from 3 to 2
    let shield = app.world().get::<ShieldActive>(cell).unwrap();
    assert_eq!(
        shield.charges, 2,
        "shield charges should decrement from 3 to 2 after absorbing overkill hit, got {}",
        shield.charges
    );
}

// ── Behavior 2: Shield absorbs damage regardless of damage amount ──

#[test]
fn shield_absorbs_damage_regardless_of_amount() {
    // Given: Cell with CellHealth::new(5.0) and ShieldActive { charges: 2 }.
    //        DamageCell { cell, damage: 50.0, source_chip: None }.
    // Then: Health remains 5.0. charges 2 to 1.
    let mut app = test_app();
    let cell = spawn_shielded_cell_with_charges(&mut app, 5.0, 2);

    app.init_resource::<CapturedDestroyed>();
    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 50.0,
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
        (health.current - 5.0).abs() < f32::EPSILON,
        "shielded cell HP should remain 5.0, got {}",
        health.current
    );
    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(
        captured.0.is_empty(),
        "shielded cell should not produce RequestCellDestroyed"
    );
    let shield = app.world().get::<ShieldActive>(cell).unwrap();
    assert_eq!(
        shield.charges, 1,
        "shield charges should decrement from 2 to 1, got {}",
        shield.charges
    );
}
