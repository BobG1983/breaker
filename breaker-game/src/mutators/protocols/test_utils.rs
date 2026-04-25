//! Protocol domain test utilities.
//!
//! Consolidates shared test helpers used across 2+ protocol scheduling test
//! suites. Suite-specific helpers remain in their local `tests/helpers.rs`.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

use crate::{cells::resources::CellConfig, prelude::*, shared::GameDrawLayer};

/// Spawns a fully-shaped, collidable [`Cell`] at `(x, y)` with `hp`. Uses
/// [`CellConfig::default()`] for width/height/half-extents and tags the
/// entity with the canonical bolt-vs-cell physics layers and a draw layer.
///
/// Used by impact-driven protocol scheduling tests (`burnout`,
/// `debt_collector`, `reckless_dash`) where the bolt collides with the cell
/// and the protocol's `BoltImpactCell` listener emits a same-tick
/// amplified `DamageDealt<Cell>`. Iron Curtain and Echo Strike have their
/// own cell-spawn variants because their tests don't need a collision shape.
pub(crate) fn spawn_cell_with_hp(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
    let cc = CellConfig::default();
    let half_extents = Vec2::new(cc.width / 2.0, cc.height / 2.0);
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Cell,
            crate::cells::components::CellWidth::new(cc.width),
            crate::cells::components::CellHeight::new(cc.height),
            Hp::new(hp),
            KilledBy { killer: None },
            Aabb2D::new(Vec2::ZERO, half_extents),
            CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            GameDrawLayer::Cell,
        ))
        .id()
}
