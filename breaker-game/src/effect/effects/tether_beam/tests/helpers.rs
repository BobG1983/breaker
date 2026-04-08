pub(super) use std::collections::HashSet;

pub(super) use bevy::prelude::*;
pub(super) use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
};
pub(super) use rantzsoft_spatial2d::components::{
    BaseSpeed, GlobalPosition2D, MaxSpeed, MinSpeed, Position2D, Scale2D, Spatial2D, Velocity2D,
};
pub(super) use rantzsoft_stateflow::CleanupOnExit;

pub(super) use crate::{
    bolt::{
        DEFAULT_BOLT_BASE_DAMAGE,
        components::{Bolt, BoltRadius, ExtraBolt},
        definition::BoltDefinition,
        registry::BoltRegistry,
    },
    cells::{components::Cell, messages::DamageCell},
    effect::effects::tether_beam::*,
    shared::{
        BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, GameDrawLayer, WALL_LAYER, birthing::Birthing,
        rng::GameRng,
    },
    state::types::{NodeState, RunState},
};

pub(super) fn world_with_bolt_registry() -> World {
    let mut world = World::new();
    let mut registry = BoltRegistry::default();
    registry.insert(
        "Bolt".to_string(),
        BoltDefinition {
            name: "Bolt".to_owned(),
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
        },
    );
    world.insert_resource(registry);
    world.insert_resource(GameRng::default());
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<ColorMaterial>>();
    world
}

/// Collects [`DamageCell`] messages into a resource for test assertions.
#[derive(Resource, Default)]
pub(super) struct DamageCellCollector(pub(super) Vec<DamageCell>);

pub(super) fn collect_damage_cells(
    mut reader: MessageReader<DamageCell>,
    mut collector: ResMut<DamageCellCollector>,
) {
    for msg in reader.read() {
        collector.0.push(msg.clone());
    }
}

pub(super) fn damage_test_app() -> App {
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
pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

pub(super) fn spawn_test_cell(app: &mut App, x: f32, y: f32) -> Entity {
    spawn_test_cell_with_extents(app, x, y, Vec2::new(10.0, 10.0))
}

pub(super) fn spawn_test_cell_with_extents(
    app: &mut App,
    x: f32,
    y: f32,
    half_extents: Vec2,
) -> Entity {
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
pub(super) fn spawn_tether_beam(
    app: &mut App,
    pos_a: Vec2,
    pos_b: Vec2,
    damage_mult: f32,
) -> (Entity, Entity, Entity) {
    spawn_tether_beam_with_edm(app, pos_a, pos_b, damage_mult, 1.0)
}

/// Spawn a tether beam with a specific `effective_damage_multiplier` value.
pub(super) fn spawn_tether_beam_with_edm(
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
                base_damage: DEFAULT_BOLT_BASE_DAMAGE,
            },
            CleanupOnExit::<NodeState>::default(),
        ))
        .id();
    // Add TetherBoltMarker to each bolt
    app.world_mut().entity_mut(bolt_a).insert(TetherBoltMarker);
    app.world_mut().entity_mut(bolt_b).insert(TetherBoltMarker);
    (bolt_a, bolt_b, beam)
}
