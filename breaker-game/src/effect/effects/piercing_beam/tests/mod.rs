use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D, Velocity2D};

use crate::{
    bolt::resources::DEFAULT_BOLT_BASE_DAMAGE,
    cells::{components::Cell, messages::DamageCell},
    effect::effects::piercing_beam::{
        PiercingBeamRequest, fire, process_piercing_beam, register, reverse,
    },
    shared::{BOLT_LAYER, CELL_LAYER, CleanupOnNodeExit, PlayfieldConfig, WALL_LAYER},
};

mod fire_tests;
mod process_tests;

// ── Test helpers ────────────────────────────────────────────────

fn piercing_beam_fire_world() -> World {
    let mut world = World::new();
    world.insert_resource(PlayfieldConfig::default());
    world
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
struct DamageCellCollector(Vec<DamageCell>);

fn collect_damage_cells(
    mut reader: MessageReader<DamageCell>,
    mut collector: ResMut<DamageCellCollector>,
) {
    for msg in reader.read() {
        collector.0.push(msg.clone());
    }
}

fn piercing_beam_damage_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzPhysics2dPlugin);
    app.add_message::<DamageCell>();
    app.insert_resource(DamageCellCollector::default());
    app.insert_resource(PlayfieldConfig::default());
    app.add_systems(Update, process_piercing_beam);
    app.add_systems(Update, collect_damage_cells.after(process_piercing_beam));
    app
}
