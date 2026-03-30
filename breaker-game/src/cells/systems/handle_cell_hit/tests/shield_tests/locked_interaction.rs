//! Behavior 7: `Locked` cell is checked before shield -- charges not consumed.

use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    cells::{components::CellHealth, messages::DamageCell},
    effect::effects::shield::ShieldActive,
};

// ── Behavior 7: Locked cell is checked before shield ──

#[test]
fn locked_cell_checked_before_shield_charges_not_consumed() {
    // Given: Cell with CellHealth::new(10.0), Locked component, AND ShieldActive { charges: 3 }.
    //        DamageCell { cell, damage: 10.0, source_chip: None }.
    // Then: Locked guard fires first. Health 10.0. Charges remain 3.
    let mut app = test_app();
    let cell = spawn_locked_cell(&mut app, 10.0);
    app.world_mut()
        .entity_mut(cell)
        .insert(ShieldActive { charges: 3 });

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
        "locked+shielded cell HP should remain 10.0, got {}",
        health.current
    );
    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(
        captured.0.is_empty(),
        "locked+shielded cell should not produce RequestCellDestroyed"
    );

    // Shield charges must remain unchanged (Locked guard fires first)
    let shield = app.world().get::<ShieldActive>(cell).unwrap();
    assert_eq!(
        shield.charges, 3,
        "locked guard should fire before shield — charges should remain 3, got {}",
        shield.charges
    );
}
