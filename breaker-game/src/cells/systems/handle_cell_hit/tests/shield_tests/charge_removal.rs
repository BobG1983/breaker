//! Behavior 3: Last charge consumed removes `ShieldActive` component.

use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    cells::{components::CellHealth, messages::DamageCell},
    effect::effects::shield::ShieldActive,
};

// ── Behavior 3: Last charge consumed removes ShieldActive component ──

#[test]
fn last_charge_consumed_removes_shield_active() {
    // Given: Cell with CellHealth::new(10.0) and ShieldActive { charges: 1 }.
    //        DamageCell { cell, damage: 10.0, source_chip: None }.
    // Then: Health remains 10.0. ShieldActive removed (charges reached 0).
    let mut app = test_app();
    let cell = spawn_shielded_cell_with_charges(&mut app, 10.0, 1);

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

    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON,
        "shielded cell HP should remain 10.0 (last charge absorbs), got {}",
        health.current
    );
    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(
        captured.0.is_empty(),
        "shielded cell should not produce RequestCellDestroyed"
    );

    // ShieldActive should be removed after charges reach 0
    assert!(
        app.world().get::<ShieldActive>(cell).is_none(),
        "ShieldActive should be removed when last charge is consumed"
    );
}

#[test]
fn last_charge_consumed_with_zero_damage() {
    // Edge case: ShieldActive { charges: 1 } with damage 0.0 — still consumes charge.
    let mut app = test_app();
    let cell = spawn_shielded_cell_with_charges(&mut app, 10.0, 1);

    app.init_resource::<CapturedDestroyed>();
    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 0.0,
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
        "zero-damage hit on shielded cell should keep HP at 10.0, got {}",
        health.current
    );
    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(
        captured.0.is_empty(),
        "zero-damage hit on shielded cell should not produce RequestCellDestroyed"
    );

    // Shield removed because the charge was consumed even with 0 damage
    assert!(
        app.world().get::<ShieldActive>(cell).is_none(),
        "ShieldActive should be removed when last charge consumed, even with zero damage"
    );
}
