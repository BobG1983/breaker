use bevy::{ecs::world::CommandQueue, prelude::*};

use crate::cells::{builder::core::types::GuardianSpawnConfig, definition::CellTypeDefinition};

/// Creates a test `CellTypeDefinition` with known values.
pub(super) fn test_cell_definition() -> CellTypeDefinition {
    CellTypeDefinition {
        id: "test".to_owned(),
        alias: "T".to_owned(),
        hp: 20.0,
        color_rgb: [1.0, 0.5, 0.2],
        required_to_clear: true,
        damage_hdr_base: 4.0,
        damage_green_min: 0.2,
        damage_blue_range: 0.4,
        damage_blue_base: 0.2,
        behaviors: None,

        effects: None,
    }
}

/// Spawns a cell via Commands backed by a `CommandQueue`, then applies the queue.
pub(super) fn spawn_cell_in_world(
    world: &mut World,
    build_fn: impl FnOnce(&mut Commands) -> Entity,
) -> Entity {
    let mut queue = CommandQueue::default();
    let entity = {
        let mut commands = Commands::new(&mut queue, world);
        build_fn(&mut commands)
    };
    queue.apply(world);
    entity
}

pub(super) fn test_guardian_config() -> GuardianSpawnConfig {
    GuardianSpawnConfig {
        hp: 10.0,
        color_rgb: [0.5, 0.8, 1.0],
        slide_speed: 30.0,
        cell_height: 24.0,
        step_x: 72.0,
        step_y: 26.0,
    }
}
