use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D, Velocity2D};

use super::super::*;
use crate::{
    bolt::{
        components::{Bolt, PiercingRemaining},
        messages::BoltImpactWall,
        resources::BoltConfig,
    },
    effect::effects::piercing::ActivePiercings,
    shared::{BOLT_LAYER, GameDrawLayer, WALL_LAYER},
    wall::components::Wall,
};

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(RantzPhysics2dPlugin)
        .add_message::<BoltImpactWall>()
        .insert_resource(WallHitMessages::default())
        .add_systems(
            FixedUpdate,
            bolt_wall_collision
                .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
        )
        .add_systems(FixedUpdate, collect_wall_hits.after(bolt_wall_collision));
    app
}

/// Accumulates one fixed timestep of overstep, then runs one update.
pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

/// Spawns a bolt at the given position with the given velocity.
pub(super) fn spawn_bolt(app: &mut App, x: f32, y: f32, vx: f32, vy: f32) -> Entity {
    Bolt::builder()
        .at_position(Vec2::new(x, y))
        .config(&BoltConfig::default())
        .with_velocity(Velocity2D(Vec2::new(vx, vy)))
        .primary()
        .spawn(app.world_mut())
}

/// Spawns a bolt with `ActivePiercings` and `PiercingRemaining` components.
pub(super) fn spawn_piercing_bolt(
    app: &mut App,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    active_piercings: Vec<u32>,
    piercing_remaining: u32,
) -> Entity {
    let entity = Bolt::builder()
        .at_position(Vec2::new(x, y))
        .config(&BoltConfig::default())
        .with_velocity(Velocity2D(Vec2::new(vx, vy)))
        .primary()
        .spawn(app.world_mut());
    app.world_mut().entity_mut(entity).insert((
        ActivePiercings(active_piercings),
        PiercingRemaining(piercing_remaining),
    ));
    entity
}

/// Spawns a wall entity at the given position with the given half-extents.
pub(super) fn spawn_wall(
    app: &mut App,
    x: f32,
    y: f32,
    half_width: f32,
    half_height: f32,
) -> Entity {
    let pos = Vec2::new(x, y);
    app.world_mut()
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

/// Collects `BoltImpactWall` messages into a resource for test assertions.
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
