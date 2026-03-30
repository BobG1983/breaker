//! Wall entity spawning — creates invisible boundary entities for CCD collision.

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{Position2D, Scale2D};

use crate::{
    shared::{BOLT_LAYER, GameDrawLayer, PlayfieldConfig, WALL_LAYER},
    wall::{
        components::{Wall, WallSize},
        messages::WallsSpawned,
    },
};

/// Spawns left, right, and ceiling wall entities.
///
/// No floor wall — bolt-lost handles that case separately.
/// Wall thickness is loaded from [`PlayfieldConfig::wall_half_thickness`].
pub(crate) fn spawn_walls(
    mut commands: Commands,
    playfield: Res<PlayfieldConfig>,
    mut walls_spawned: MessageWriter<WallsSpawned>,
) {
    let half_width = playfield.width / 2.0;
    let half_height = playfield.height / 2.0;
    let wall_ht = playfield.wall_half_thickness();

    // Left wall
    commands.spawn((
        Wall,
        WallSize {},
        Position2D(Vec2::new(playfield.left() - wall_ht, 0.0)),
        Scale2D {
            x: wall_ht,
            y: half_height,
        },
        Aabb2D::new(Vec2::ZERO, Vec2::new(wall_ht, half_height)),
        CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
        GameDrawLayer::Wall,
    ));

    // Right wall
    commands.spawn((
        Wall,
        WallSize {},
        Position2D(Vec2::new(playfield.right() + wall_ht, 0.0)),
        Scale2D {
            x: wall_ht,
            y: half_height,
        },
        Aabb2D::new(Vec2::ZERO, Vec2::new(wall_ht, half_height)),
        CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
        GameDrawLayer::Wall,
    ));

    // Ceiling
    commands.spawn((
        Wall,
        WallSize {},
        Position2D(Vec2::new(0.0, playfield.top() + wall_ht)),
        Scale2D {
            x: half_width,
            y: wall_ht,
        },
        Aabb2D::new(Vec2::ZERO, Vec2::new(half_width, wall_ht)),
        CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
        GameDrawLayer::Wall,
    ));

    walls_spawned.write(WallsSpawned);
}
