//! Behavior 1 & 4: locked cell immunity and unlocked cell damage.

use bevy::prelude::*;

use super::helpers::*;
use crate::cells::{components::*, messages::DamageCell};

// ---------------------------------------------------------------
// Behavior 1: Locked cell immune to bolt damage
// ---------------------------------------------------------------

#[test]
fn locked_cell_hp_unchanged_after_damage_cell() {
    let mut app = hit_app();
    let cell = spawn_locked_cell(&mut app, 10.0);

    app.insert_resource(TestDamageCellMessage(Some(DamageCell {
        cell,
        damage: 10.0,
        source_chip: None,
    })));
    tick(&mut app);

    // Locked cell should still exist (not destroyed)
    assert!(
        app.world().get_entity(cell).is_ok(),
        "locked cell should not be despawned by DamageCell"
    );
    // HP should be untouched
    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON,
        "locked cell HP should remain 10.0, got {}",
        health.current
    );
}

// ---------------------------------------------------------------
// Behavior 4: Unlocked cell takes normal damage
// ---------------------------------------------------------------

#[derive(Resource, Default)]
struct CapturedRequestCellDestroyed(Vec<crate::cells::messages::RequestCellDestroyed>);

fn capture_request_cell_destroyed(
    mut reader: MessageReader<crate::cells::messages::RequestCellDestroyed>,
    mut captured: ResMut<CapturedRequestCellDestroyed>,
) {
    for msg in reader.read() {
        captured.0.push(msg.clone());
    }
}

#[test]
fn unlocked_cell_takes_damage_and_sends_request_destroyed() {
    use crate::cells::systems::handle_cell_hit;

    let mut app = hit_app();
    let cell = spawn_unlocked_cell(&mut app, 10.0);

    app.init_resource::<CapturedRequestCellDestroyed>();
    app.insert_resource(TestDamageCellMessage(Some(DamageCell {
        cell,
        damage: 10.0,
        source_chip: None,
    })));
    app.add_systems(
        FixedUpdate,
        capture_request_cell_destroyed.after(handle_cell_hit),
    );
    tick(&mut app);

    // Two-phase destruction: entity stays alive, RequestCellDestroyed sent
    let captured = app.world().resource::<CapturedRequestCellDestroyed>();
    assert_eq!(
        captured.0.len(),
        1,
        "unlocked 10-HP cell should produce RequestCellDestroyed from 10.0 damage"
    );
    assert_eq!(
        captured.0[0].cell, cell,
        "RequestCellDestroyed should carry the destroyed cell entity"
    );
}
