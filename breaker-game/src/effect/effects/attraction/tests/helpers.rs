use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::{
    BaseSpeed, MaxSpeed, MinAngleHorizontal, MinAngleVertical, MinSpeed, Position2D,
};

use crate::bolt::messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall};

// Shared test app builders and helpers used by sub-modules.

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<crate::shared::game_state::GameState>();
    app.add_sub_state::<crate::shared::PlayingState>();
    app.insert_resource(CollisionQuadtree::default());
    app.add_systems(Update, super::super::effect::apply_attraction);
    app
}

pub(super) fn test_app_with_manage() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<BoltImpactCell>();
    app.add_message::<BoltImpactWall>();
    app.add_message::<BoltImpactBreaker>();
    app.add_systems(
        FixedUpdate,
        (
            enqueue_messages.before(super::super::effect::manage_attraction_types),
            super::super::effect::manage_attraction_types,
        ),
    );
    app
}

pub(super) fn enter_playing(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<crate::shared::game_state::GameState>>()
        .set(crate::shared::game_state::GameState::Playing);
    app.update();
}

/// Populate the quadtree with entities that have positions and collision layers.
pub(super) fn populate_quadtree(app: &mut App, entries: &[(Entity, Vec2, CollisionLayers)]) {
    let mut quadtree = app.world_mut().resource_mut::<CollisionQuadtree>();
    for &(entity, pos, layers) in entries {
        quadtree
            .quadtree
            .insert(entity, Aabb2D::new(pos, Vec2::new(8.0, 8.0)), layers);
    }
}

/// Resource holding test impact messages to enqueue before the system runs.
#[derive(Resource, Default)]
pub(super) struct TestImpactMessages {
    pub(super) cell: Vec<BoltImpactCell>,
    pub(super) wall: Vec<BoltImpactWall>,
    pub(super) breaker: Vec<BoltImpactBreaker>,
}

fn enqueue_messages(
    msgs: Res<TestImpactMessages>,
    mut cell_writer: MessageWriter<BoltImpactCell>,
    mut wall_writer: MessageWriter<BoltImpactWall>,
    mut breaker_writer: MessageWriter<BoltImpactBreaker>,
) {
    for msg in &msgs.cell {
        cell_writer.write(msg.clone());
    }
    for msg in &msgs.wall {
        wall_writer.write(msg.clone());
    }
    for msg in &msgs.breaker {
        breaker_writer.write(msg.clone());
    }
}

pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

/// Spatial params required by `SpatialData`. Permissive values so the
/// velocity formula preserves direction unchanged — `base_speed` 10000.0
/// with no angle clamping and no min/max speed clamping.
pub(super) fn spatial_params() -> (
    BaseSpeed,
    MinSpeed,
    MaxSpeed,
    MinAngleHorizontal,
    MinAngleVertical,
    Position2D,
) {
    (
        BaseSpeed(10000.0),
        MinSpeed(0.0),
        MaxSpeed(f32::MAX),
        MinAngleHorizontal(0.0),
        MinAngleVertical(0.0),
        Position2D(Vec2::ZERO),
    )
}
