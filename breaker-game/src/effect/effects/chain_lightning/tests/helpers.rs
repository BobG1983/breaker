use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D};

pub(super) use super::super::effect::*;
use crate::{
    cells::{components::Cell, messages::DamageCell},
    effect::core::EffectSourceChip,
    shared::{CELL_LAYER, CleanupOnNodeExit, GameRng},
};

/// App with `MinimalPlugins` + physics + `GameRng` seeded at 42.
/// Does NOT register `DamageCell` message (use `chain_lightning_damage_test_app` for that).
pub(super) fn chain_lightning_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzPhysics2dPlugin);
    app.insert_resource(GameRng::from_seed(42));
    app.add_message::<DamageCell>();
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

/// App with `MinimalPlugins` + physics + `DamageCell` message + `tick_chain_lightning` system
/// + `DamageCellCollector` for verifying `DamageCell` messages after ticking.
pub(super) fn chain_lightning_damage_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzPhysics2dPlugin);
    app.insert_resource(GameRng::from_seed(42));
    app.add_message::<DamageCell>();
    app.insert_resource(DamageCellCollector::default());
    app.add_systems(Update, tick_chain_lightning);
    app.add_systems(Update, collect_damage_cells.after(tick_chain_lightning));
    app
}

/// Configuration for spawning a `ChainLightningChain` in tests.
pub(super) struct SpawnChainConfig {
    pub source: Vec2,
    pub remaining_jumps: u32,
    pub damage: f32,
    pub range: f32,
    pub arc_speed: f32,
    pub hit_set: HashSet<Entity>,
    pub state: ChainState,
    pub source_chip: Option<String>,
}

/// Spawn a `ChainLightningChain` entity with specified parameters for unit testing
/// `tick_chain_lightning` independently of `fire()`.
pub(super) fn spawn_chain(app: &mut App, config: SpawnChainConfig) -> Entity {
    app.world_mut()
        .spawn((
            ChainLightningChain {
                source: config.source,
                remaining_jumps: config.remaining_jumps,
                damage: config.damage,
                hit_set: config.hit_set,
                state: config.state,
                range: config.range,
                arc_speed: config.arc_speed,
            },
            EffectSourceChip(config.source_chip),
            CleanupOnNodeExit,
        ))
        .id()
}

/// Spawn a bare `ChainLightningArc` marker entity with `Transform` and `CleanupOnNodeExit`.
/// The arc's logical state (target, position) lives in `ChainState::ArcTraveling` on the chain entity.
pub(super) fn spawn_arc(app: &mut App, position: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            ChainLightningArc,
            Transform::from_xyz(position.x, position.y, 0.0),
            CleanupOnNodeExit,
        ))
        .id()
}
