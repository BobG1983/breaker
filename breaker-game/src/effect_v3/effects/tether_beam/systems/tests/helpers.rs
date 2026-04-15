use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::super::system::*;
use crate::{
    cells::components::Cell,
    shared::{
        death_pipeline::{DamageDealt, Dead},
        test_utils::TestAppBuilder,
    },
};

pub(super) fn tether_test_app() -> App {
    TestAppBuilder::new()
        .with_message_capture::<DamageDealt<Cell>>()
        .with_system(FixedUpdate, tick_tether_beam)
        .build()
}

pub(super) fn cleanup_test_app() -> App {
    TestAppBuilder::new()
        .with_system(FixedUpdate, cleanup_tether_beams)
        .build()
}

/// Spawns a placeholder "bolt endpoint" — the tick system only reads
/// `Position2D` from the entities stored in `TetherBeamSource`, so no
/// `Bolt` marker is required.
pub(super) fn spawn_endpoint(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut().spawn(Position2D(pos)).id()
}

pub(super) fn spawn_alive_cell(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut().spawn((Cell, Position2D(pos))).id()
}

pub(super) fn spawn_dead_cell(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut().spawn((Cell, Position2D(pos), Dead)).id()
}

pub(super) fn damage_msgs(app: &App) -> Vec<DamageDealt<Cell>> {
    app.world()
        .resource::<crate::shared::test_utils::MessageCollector<DamageDealt<Cell>>>()
        .0
        .clone()
}
