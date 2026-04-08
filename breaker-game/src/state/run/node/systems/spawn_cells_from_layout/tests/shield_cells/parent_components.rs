//! Tests for `ShieldParent` marker, `Locked`, and `Locks` components
//! on shield cells.

use bevy::prelude::*;

use super::helpers::*;
use crate::cells::components::*;

// -- Behavior 1: Shield cell has ShieldParent + OrbitConfig + Locked + Locks --

#[test]
fn shield_cell_has_shield_parent_marker() {
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    let shield_count = app
        .world_mut()
        .query::<(&Cell, &ShieldParent)>()
        .iter(app.world())
        .count();
    assert_eq!(
        shield_count, 1,
        "shield cell ('H') should have ShieldParent marker"
    );
}

#[test]
fn shield_cell_has_locked_and_lock_adjacents() {
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    let shield_locked_count = app
        .world_mut()
        .query::<(&Cell, &ShieldParent, &Locked, &Locks)>()
        .iter(app.world())
        .count();
    assert_eq!(
        shield_locked_count, 1,
        "shield cell should have Locked + Locks"
    );
}

#[test]
fn shield_cell_lock_adjacents_contains_orbit_entity_ids() {
    // Given: shield with orbit_count=3
    // When: spawn runs
    // Then: Locks contains exactly 3 entity IDs (the orbit children)
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    let shield_adjacents: Vec<&Locks> = app
        .world_mut()
        .query_filtered::<&Locks, With<ShieldParent>>()
        .iter(app.world())
        .collect();
    assert_eq!(shield_adjacents.len(), 1);
    assert_eq!(
        shield_adjacents[0].0.len(),
        3,
        "shield Locks should contain 3 orbit entity IDs, got {}",
        shield_adjacents[0].0.len()
    );

    // Verify each entity in Locks is an actual OrbitCell
    for &orbit_entity in &shield_adjacents[0].0 {
        assert!(
            app.world().get::<OrbitCell>(orbit_entity).is_some(),
            "Locks entity {orbit_entity:?} should have OrbitCell component"
        );
    }
}
