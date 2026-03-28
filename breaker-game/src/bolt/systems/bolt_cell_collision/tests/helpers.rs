use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D};

// Re-export constants used by test modules
pub(super) use super::super::system::MAX_BOUNCES;
use super::super::system::bolt_cell_collision;
use crate::{
    bolt::{
        components::{BoltBaseSpeed, BoltRadius},
        messages::{BoltImpactCell, BoltImpactWall},
        resources::BoltConfig,
    },
    cells::{
        components::{Cell, CellHealth, CellHeight, CellWidth},
        messages::DamageCell,
        resources::CellConfig,
    },
    shared::{BOLT_LAYER, CELL_LAYER, GameDrawLayer, WALL_LAYER},
    wall::components::{Wall, WallSize},
};

/// Real grid vertical spacing: `cell_height` (24) + padding (4) = 28
pub(super) const GRID_STEP_Y: f32 = 28.0;
/// Real grid horizontal spacing: `cell_width` (70) + padding (4) = 74
pub(super) const GRID_STEP_X: f32 = 74.0;

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(RantzPhysics2dPlugin)
        .add_message::<BoltImpactCell>()
        .add_message::<DamageCell>()
        .add_message::<BoltImpactWall>()
        .add_systems(
            FixedUpdate,
            bolt_cell_collision
                .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
        );
    app
}

pub(super) fn bolt_param_bundle() -> (BoltBaseSpeed, BoltRadius) {
    let bc = BoltConfig::default();
    (BoltBaseSpeed(bc.base_speed), BoltRadius(bc.radius))
}

pub(super) fn default_cell_dims() -> (CellWidth, CellHeight) {
    let cc = CellConfig::default();
    (CellWidth::new(cc.width), CellHeight::new(cc.height))
}

/// Accumulates one fixed timestep of overstep, then runs one update.
pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

/// Cell entities use `Position2D` as canonical position.
pub(super) fn spawn_cell(app: &mut App, x: f32, y: f32) -> Entity {
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

/// Spawns a cell with explicit [`CellHealth`] for piercing lookahead tests.
pub(super) fn spawn_cell_with_health(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
    let (cw, ch) = default_cell_dims();
    let half_extents = Vec2::new(cw.half_width(), ch.half_height());
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Cell,
            cw,
            ch,
            CellHealth::new(hp),
            Aabb2D::new(Vec2::ZERO, half_extents),
            CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            GameDrawLayer::Cell,
        ))
        .id()
}

pub(super) fn spawn_wall(app: &mut App, x: f32, y: f32, half_width: f32, half_height: f32) {
    let pos = Vec2::new(x, y);
    app.world_mut().spawn((
        Wall,
        WallSize {},
        Aabb2D::new(Vec2::ZERO, Vec2::new(half_width, half_height)),
        CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
        Position2D(pos),
        GlobalPosition2D(pos),
        Spatial2D,
        GameDrawLayer::Wall,
    ));
}

/// Spawns a cell with explicit `Aabb2D` `half_extents` that differ from the
/// legacy `CellWidth`/`CellHeight` dimensions. Used to test which source
/// the collision system reads for Minkowski expansion.
pub(super) fn spawn_cell_with_custom_aabb(
    app: &mut App,
    x: f32,
    y: f32,
    aabb_half_extents: Vec2,
) -> Entity {
    let (cw, ch) = default_cell_dims();
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Cell,
            cw,
            ch,
            Aabb2D::new(Vec2::ZERO, aabb_half_extents),
            CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            GameDrawLayer::Cell,
        ))
        .id()
}

/// Collects `BoltImpactCell` messages into a resource for test assertions.
#[derive(Resource, Default)]
pub(super) struct HitCells(pub(super) Vec<Entity>);

pub(super) fn collect_cell_hits(
    mut reader: MessageReader<BoltImpactCell>,
    mut hits: ResMut<HitCells>,
) {
    for msg in reader.read() {
        hits.0.push(msg.cell);
    }
}

/// Collects full `BoltImpactCell` messages (including the bolt field) for assertion.
#[derive(Resource, Default)]
pub(super) struct FullHitMessages(pub(super) Vec<BoltImpactCell>);

pub(super) fn collect_full_hits(
    mut reader: MessageReader<BoltImpactCell>,
    mut hits: ResMut<FullHitMessages>,
) {
    for msg in reader.read() {
        hits.0.push(msg.clone());
    }
}

/// Collects [`DamageCell`] messages into a resource for test assertions.
#[derive(Resource, Default)]
pub(super) struct DamageCellMessages(pub(super) Vec<DamageCell>);

pub(super) fn collect_damage_cells(
    mut reader: MessageReader<DamageCell>,
    mut msgs: ResMut<DamageCellMessages>,
) {
    for msg in reader.read() {
        msgs.0.push(msg.clone());
    }
}

/// Collects [`BoltImpactWall`] messages into a resource for test assertions.
#[derive(Resource, Default)]
pub(super) struct WallHitMessages(pub(super) Vec<BoltImpactWall>);

pub(super) fn collect_wall_hits(
    mut reader: MessageReader<BoltImpactWall>,
    mut msgs: ResMut<WallHitMessages>,
) {
    for msg in reader.read() {
        msgs.0.push(msg.clone());
    }
}

/// Creates a test app with `DamageCell` and `BoltImpactWall` message capture
/// in addition to the standard `BoltImpactCell`.
pub(super) fn test_app_with_damage_and_wall_messages() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(RantzPhysics2dPlugin)
        .add_message::<BoltImpactCell>()
        .add_message::<DamageCell>()
        .add_message::<BoltImpactWall>()
        .insert_resource(DamageCellMessages::default())
        .insert_resource(WallHitMessages::default())
        .insert_resource(FullHitMessages::default())
        .add_systems(
            FixedUpdate,
            bolt_cell_collision
                .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
        )
        .add_systems(
            FixedUpdate,
            (collect_damage_cells, collect_wall_hits, collect_full_hits).after(bolt_cell_collision),
        );
    app
}
