//! Shared test helpers for the cells domain.
//!
//! Provides common cell spawning, dimension, and definition factories used
//! across multiple cells test suites. Suite-specific helpers remain local.

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D};

use crate::{
    cells::{
        components::{Cell, CellDamageVisuals, CellHeight, CellWidth},
        definition::{CellTypeDefinition, Toughness},
        resources::CellConfig,
    },
    shared::{BOLT_LAYER, CELL_LAYER, GameDrawLayer},
};

/// Returns default `(CellWidth, CellHeight)` from `CellConfig::default()`.
pub(crate) fn default_cell_dims() -> (CellWidth, CellHeight) {
    let cc = CellConfig::default();
    (CellWidth::new(cc.width), CellHeight::new(cc.height))
}

/// Returns a standard `CellDamageVisuals` for tests that need visual components.
pub(crate) fn default_damage_visuals() -> CellDamageVisuals {
    CellDamageVisuals {
        hdr_base: 4.0,
        green_min: 0.2,
        blue_range: 0.4,
        blue_base: 0.2,
    }
}

/// Creates a test `CellTypeDefinition` with known values.
///
/// This is the canonical test definition used across builder tests and
/// registry seeding. Uses `Toughness::Standard`, alias `"T"`, and
/// standard damage visual parameters.
pub(crate) fn test_cell_definition() -> CellTypeDefinition {
    CellTypeDefinition {
        id: "test".to_owned(),
        alias: "T".to_owned(),
        toughness: Toughness::default(),
        color_rgb: [1.0, 0.5, 0.2],
        required_to_clear: true,
        damage_hdr_base: 4.0,
        damage_green_min: 0.2,
        damage_blue_range: 0.4,
        damage_blue_base: 0.2,
        behaviors: None,
        effects: None,
    }
}

/// Spawns a cell at the given position with standard physics components.
///
/// Includes `Cell`, `CellWidth`, `CellHeight`, `Aabb2D`, `CollisionLayers`,
/// `Position2D`, `GlobalPosition2D`, `Spatial2D`, and `GameDrawLayer::Cell`.
/// Used by collision-oriented test suites (bolt-cell, cell-wall).
pub(crate) fn spawn_cell(app: &mut App, x: f32, y: f32) -> Entity {
    let (cw, ch) = default_cell_dims();
    let half_extents = Vec2::new(cw.half_width(), ch.half_height());
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Cell,
            cw,
            ch,
            Aabb2D::new(Vec2::ZERO, half_extents),
            CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            GameDrawLayer::Cell,
        ))
        .id()
}

/// Spawns a cell via `Commands` backed by a `CommandQueue`, then applies the queue.
///
/// Used by builder tests that need to exercise the `Cell::builder()` API
/// through `Commands`.
pub(crate) fn spawn_cell_in_world(
    world: &mut World,
    build_fn: impl FnOnce(&mut Commands) -> Entity,
) -> Entity {
    let mut queue = bevy::ecs::world::CommandQueue::default();
    let entity = {
        let mut commands = Commands::new(&mut queue, world);
        build_fn(&mut commands)
    };
    queue.apply(world);
    entity
}
