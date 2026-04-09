use bevy::{ecs::world::CommandQueue, prelude::*};
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D, Velocity2D};

use crate::{
    bolt::{
        components::{Bolt, PiercingRemaining},
        definition::BoltDefinition,
        messages::BoltImpactWall,
        systems::bolt_wall_collision::*,
    },
    effect::effects::piercing::ActivePiercings,
    shared::{BOLT_LAYER, GameDrawLayer, WALL_LAYER},
    walls::components::Wall,
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

pub(super) use crate::shared::test_utils::tick;

/// Creates a `BoltDefinition` matching the values previously provided by
/// `BoltConfig::default()`, so existing position calculations remain valid.
fn test_bolt_definition() -> BoltDefinition {
    BoltDefinition {
        name: "Bolt".to_string(),
        base_speed: 400.0,
        min_speed: 200.0,
        max_speed: 800.0,
        radius: 8.0,
        base_damage: 10.0,
        effects: vec![],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    }
}

/// Spawns a bolt at the given position with the given velocity.
pub(super) fn spawn_bolt(app: &mut App, x: f32, y: f32, vx: f32, vy: f32) -> Entity {
    let def = test_bolt_definition();
    let world = app.world_mut();
    let mut queue = CommandQueue::default();
    let entity = {
        let mut commands = Commands::new(&mut queue, world);
        Bolt::builder()
            .at_position(Vec2::new(x, y))
            .definition(&def)
            .with_velocity(Velocity2D(Vec2::new(vx, vy)))
            .primary()
            .headless()
            .spawn(&mut commands)
    };
    queue.apply(world);
    entity
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
    let def = test_bolt_definition();
    let world = app.world_mut();
    let mut queue = CommandQueue::default();
    let entity = {
        let mut commands = Commands::new(&mut queue, world);
        Bolt::builder()
            .at_position(Vec2::new(x, y))
            .definition(&def)
            .with_velocity(Velocity2D(Vec2::new(vx, vy)))
            .primary()
            .headless()
            .spawn(&mut commands)
    };
    queue.apply(world);
    world.entity_mut(entity).insert((
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
