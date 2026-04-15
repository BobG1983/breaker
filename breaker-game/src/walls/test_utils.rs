//! Shared test helpers for the walls domain.
//!
//! Provides wall spawning utilities for tests across domains:
//! - `spawn_wall` — raw-component wall at arbitrary position and half-extents
//! - `spawn_left_wall` / `spawn_right_wall` / `spawn_ceiling_wall` — builder-based
//!   walls at default `PlayfieldConfig` positions with `GlobalPosition2D` sync

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

use crate::{prelude::*, shared::GameDrawLayer, walls::components::Wall};

/// Spawns a wall entity at `(x, y)` with the given half-extents.
///
/// Includes all standard wall components: `Wall`, `Spatial2D`, `Position2D`,
/// `GlobalPosition2D`, `Aabb2D`, `CollisionLayers`, and `GameDrawLayer::Wall`.
pub(crate) fn spawn_wall(
    app: &mut App,
    x: f32,
    y: f32,
    half_width: f32,
    half_height: f32,
) -> Entity {
    let pos = Vec2::new(x, y);
    let world = app.world_mut();
    world
        .spawn((
            Wall,
            Aabb2D::new(Vec2::ZERO, Vec2::new(half_width, half_height)),
            CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            GameDrawLayer::Wall,
        ))
        .id()
}

/// Spawns a left wall using `Wall::builder().left()` with default playfield config.
///
/// Manually syncs `GlobalPosition2D` from the builder-set `Position2D` so
/// collision queries work without running the spatial sync system.
pub(crate) fn spawn_left_wall(app: &mut App) -> Entity {
    let pf = PlayfieldConfig::default();
    let entity = {
        let world = app.world_mut();
        let entity = Wall::builder().left(&pf).spawn(&mut world.commands());
        world.flush();
        entity
    };
    let pos = app.world().get::<Position2D>(entity).unwrap().0;
    app.world_mut()
        .entity_mut(entity)
        .insert(GlobalPosition2D(pos));
    entity
}

/// Spawns a right wall using `Wall::builder().right()` with default playfield config.
///
/// Manually syncs `GlobalPosition2D` from the builder-set `Position2D` so
/// collision queries work without running the spatial sync system.
pub(crate) fn spawn_right_wall(app: &mut App) -> Entity {
    let pf = PlayfieldConfig::default();
    let entity = {
        let world = app.world_mut();
        let entity = Wall::builder().right(&pf).spawn(&mut world.commands());
        world.flush();
        entity
    };
    let pos = app.world().get::<Position2D>(entity).unwrap().0;
    app.world_mut()
        .entity_mut(entity)
        .insert(GlobalPosition2D(pos));
    entity
}

/// Spawns a ceiling wall using `Wall::builder().ceiling()` with default playfield config.
///
/// Manually syncs `GlobalPosition2D` from the builder-set `Position2D` so
/// collision queries work without running the spatial sync system.
pub(crate) fn spawn_ceiling_wall(app: &mut App) -> Entity {
    let pf = PlayfieldConfig::default();
    let entity = {
        let world = app.world_mut();
        let entity = Wall::builder().ceiling(&pf).spawn(&mut world.commands());
        world.flush();
        entity
    };
    let pos = app.world().get::<Position2D>(entity).unwrap().0;
    app.world_mut()
        .entity_mut(entity)
        .insert(GlobalPosition2D(pos));
    entity
}
