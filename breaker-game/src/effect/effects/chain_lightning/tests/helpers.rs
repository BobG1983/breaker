use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D};

pub(super) use super::super::effect::*;
use crate::{
    cells::{components::Cell, messages::DamageCell},
    shared::{CELL_LAYER, GameRng},
};

pub(super) fn chain_lightning_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzPhysics2dPlugin);
    app.insert_resource(GameRng::from_seed(42));
    app
}

/// Accumulates one fixed timestep then runs one update (ensures quadtree maintenance runs).
pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

pub(super) fn spawn_test_cell(app: &mut App, x: f32, y: f32) -> Entity {
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Cell,
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
            CollisionLayers::new(CELL_LAYER, 0),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
        ))
        .id()
}

/// Collects [`DamageCell`] messages into a resource for test assertions.
#[derive(Resource, Default)]
pub(super) struct DamageCellCollector(pub Vec<DamageCell>);

pub(super) fn collect_damage_cells(
    mut reader: MessageReader<DamageCell>,
    mut collector: ResMut<DamageCellCollector>,
) {
    for msg in reader.read() {
        collector.0.push(msg.clone());
    }
}

pub(super) fn chain_lightning_damage_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzPhysics2dPlugin);
    app.add_message::<DamageCell>();
    app.insert_resource(DamageCellCollector::default());
    app.add_systems(Update, process_chain_lightning);
    app.add_systems(Update, collect_damage_cells.after(process_chain_lightning));
    app
}
