//! Group C — Collision Layers at Spawn
//!
//! Verifies that phantom cells spawn with the correct `CollisionLayers`
//! based on their starting phase.

use bevy::prelude::*;

use super::helpers::*;
use crate::{cells::behaviors::phantom::components::PhantomPhase, prelude::*};

// Behavior 19: Solid starting phase has standard CollisionLayers
#[test]
fn solid_starting_phase_has_standard_collision_layers() {
    let mut app = build_phantom_test_app();
    let entity = spawn_phantom_cell(&mut app, Vec2::ZERO, PhantomPhase::Solid, 20.0);

    let layers = app
        .world()
        .get::<CollisionLayers>(entity)
        .expect("phantom cell should have CollisionLayers");
    assert_eq!(
        layers.membership, CELL_LAYER,
        "Solid phantom cell membership should be CELL_LAYER, got 0x{:02X}",
        layers.membership
    );
    assert_eq!(
        layers.mask, BOLT_LAYER,
        "Solid phantom cell mask should be BOLT_LAYER, got 0x{:02X}",
        layers.mask
    );
}

// Behavior 20: Telegraph starting phase has standard CollisionLayers
#[test]
fn telegraph_starting_phase_has_standard_collision_layers() {
    let mut app = build_phantom_test_app();
    let entity = spawn_phantom_cell(&mut app, Vec2::ZERO, PhantomPhase::Telegraph, 20.0);

    let layers = app
        .world()
        .get::<CollisionLayers>(entity)
        .expect("phantom cell should have CollisionLayers");
    assert_eq!(
        layers.membership, CELL_LAYER,
        "Telegraph phantom cell membership should be CELL_LAYER, got 0x{:02X}",
        layers.membership
    );
    assert_eq!(
        layers.mask, BOLT_LAYER,
        "Telegraph phantom cell mask should be BOLT_LAYER, got 0x{:02X}",
        layers.mask
    );
}

// Behavior 21: Ghost starting phase has zeroed CollisionLayers
#[test]
fn ghost_starting_phase_has_zeroed_collision_layers() {
    let mut app = build_phantom_test_app();
    let entity = spawn_phantom_cell(&mut app, Vec2::ZERO, PhantomPhase::Ghost, 20.0);

    let layers = app
        .world()
        .get::<CollisionLayers>(entity)
        .expect("phantom cell should have CollisionLayers");
    assert_eq!(
        layers.membership, 0,
        "Ghost phantom cell membership should be 0, got 0x{:02X}",
        layers.membership
    );
    assert_eq!(
        layers.mask, 0,
        "Ghost phantom cell mask should be 0, got 0x{:02X}",
        layers.mask
    );
}

// Behavior 21 edge: non-phantom cell at same position still has standard layers
#[test]
fn non_phantom_cell_at_same_position_has_standard_layers() {
    let mut app = build_phantom_test_app();

    // Spawn a plain (non-phantom) cell for comparison
    let plain_entity = crate::cells::test_utils::spawn_cell_in_world(app.world_mut(), |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(TEST_CELL_DIM, TEST_CELL_DIM)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let layers = app
        .world()
        .get::<CollisionLayers>(plain_entity)
        .expect("plain cell should have CollisionLayers");
    assert_eq!(
        layers.membership, CELL_LAYER,
        "non-phantom cell membership should be CELL_LAYER"
    );
    assert_eq!(
        layers.mask, BOLT_LAYER,
        "non-phantom cell mask should be BOLT_LAYER"
    );
}
