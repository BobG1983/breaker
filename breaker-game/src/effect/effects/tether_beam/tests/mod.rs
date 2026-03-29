use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
};
use rantzsoft_spatial2d::components::{
    GlobalPosition2D, Position2D, Scale2D, Spatial2D, Velocity2D,
};

use super::*;
use crate::{
    bolt::{
        BASE_BOLT_DAMAGE,
        components::{Bolt, BoltBaseSpeed, BoltMaxSpeed, BoltMinSpeed, BoltRadius, ExtraBolt},
        resources::BoltConfig,
    },
    cells::{components::Cell, messages::DamageCell},
    shared::{
        BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, CleanupOnNodeExit, CleanupOnRunEnd, GameDrawLayer,
        WALL_LAYER, rng::GameRng,
    },
};

mod fire_tests;
mod tick_damage_tests;
mod tick_lifetime_tests;

fn world_with_bolt_config() -> World {
    let mut world = World::new();
    world.insert_resource(BoltConfig::default());
    world.insert_resource(GameRng::default());
    world
}

/// Collects [`DamageCell`] messages into a resource for test assertions.
#[derive(Resource, Default)]
struct DamageCellCollector(Vec<DamageCell>);

fn collect_damage_cells(
    mut reader: MessageReader<DamageCell>,
    mut collector: ResMut<DamageCellCollector>,
) {
    for msg in reader.read() {
        collector.0.push(msg.clone());
    }
}

fn damage_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzPhysics2dPlugin);
    app.add_message::<DamageCell>();
    app.insert_resource(DamageCellCollector::default());
    app.add_systems(Update, tick_tether_beam);
    app.add_systems(Update, collect_damage_cells.after(tick_tether_beam));
    app
}

/// Accumulates one fixed timestep then runs one update (ensures quadtree maintenance runs).
fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

fn spawn_test_cell(app: &mut App, x: f32, y: f32) -> Entity {
    spawn_test_cell_with_extents(app, x, y, Vec2::new(10.0, 10.0))
}

fn spawn_test_cell_with_extents(app: &mut App, x: f32, y: f32, half_extents: Vec2) -> Entity {
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Cell,
            Aabb2D::new(Vec2::ZERO, half_extents),
            CollisionLayers::new(CELL_LAYER, 0),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
        ))
        .id()
}

/// Spawn a tether beam with two bolt entities at given positions.
fn spawn_tether_beam(
    app: &mut App,
    pos_a: Vec2,
    pos_b: Vec2,
    damage_mult: f32,
) -> (Entity, Entity, Entity) {
    spawn_tether_beam_with_edm(app, pos_a, pos_b, damage_mult, 1.0)
}

/// Spawn a tether beam with a specific `effective_damage_multiplier` value.
fn spawn_tether_beam_with_edm(
    app: &mut App,
    pos_a: Vec2,
    pos_b: Vec2,
    damage_mult: f32,
    effective_damage_multiplier: f32,
) -> (Entity, Entity, Entity) {
    let bolt_a = app
        .world_mut()
        .spawn((Bolt, Position2D(pos_a), GlobalPosition2D(pos_a), Spatial2D))
        .id();
    let bolt_b = app
        .world_mut()
        .spawn((Bolt, Position2D(pos_b), GlobalPosition2D(pos_b), Spatial2D))
        .id();
    let beam = app
        .world_mut()
        .spawn((
            TetherBeamComponent {
                bolt_a,
                bolt_b,
                damage_mult,
                effective_damage_multiplier,
            },
            CleanupOnNodeExit,
        ))
        .id();
    // Add TetherBoltMarker to each bolt
    app.world_mut()
        .entity_mut(bolt_a)
        .insert(TetherBoltMarker(beam));
    app.world_mut()
        .entity_mut(bolt_b)
        .insert(TetherBoltMarker(beam));
    (bolt_a, bolt_b, beam)
}
