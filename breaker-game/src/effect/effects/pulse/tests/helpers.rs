pub(super) use std::collections::HashSet;

pub(super) use bevy::prelude::*;
pub(super) use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
};
pub(super) use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D};

pub(super) use crate::{
    bolt::resources::DEFAULT_BOLT_BASE_DAMAGE,
    cells::{components::Cell, messages::DamageCell},
    effect::effects::pulse::*,
    shared::{BOLT_LAYER, CELL_LAYER, WALL_LAYER},
};

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<crate::shared::game_state::GameState>();
    app.add_sub_state::<crate::shared::playing_state::PlayingState>();
    app.add_systems(Update, tick_pulse_emitter);
    app.add_systems(Update, tick_pulse_ring);
    app.add_systems(Update, despawn_finished_pulse_ring);
    app
}

pub(super) fn enter_playing(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<crate::shared::game_state::GameState>>()
        .set(crate::shared::game_state::GameState::Playing);
    app.update();
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
    app.add_systems(Update, apply_pulse_damage);
    app.add_systems(Update, collect_damage_cells.after(apply_pulse_damage));
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
