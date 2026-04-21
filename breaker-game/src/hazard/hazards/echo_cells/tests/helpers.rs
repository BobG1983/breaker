use std::{marker::PhantomData, time::Duration};

use bevy::{ecs::world::CommandQueue, prelude::*};

use super::super::system::*;
use crate::{
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::ActiveHazards,
    },
    prelude::*,
};

pub(super) fn test_app_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message::<Destroyed<Cell>>()
        .build()
}

pub(super) fn write_destroyed(app: &mut App, victim: Entity, pos: Vec2) {
    app.world_mut()
        .resource_mut::<Messages<Destroyed<Cell>>>()
        .write(Destroyed::<Cell> {
            victim,
            killer: None,
            victim_pos: pos,
            killer_pos: None,
            _marker: PhantomData,
        });
}

pub(super) fn tick_with_dt(app: &mut App, dt: Duration) {
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .set_timestep(dt);
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(dt);
    app.update();
}

// ── New shared helpers (Fracture precedent) ───────────────────────────

pub(super) fn test_app_not_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveHazards>()
        .with_message::<Destroyed<Cell>>()
        .build()
}

pub(super) const fn canonical_config() -> EchoCellsConfig {
    EchoCellsConfig {
        delay_secs:           1.5,
        base_hp:              1.0,
        per_level_multiplier: 2.0,
    }
}

pub(super) fn install_echo_cells_config(app: &mut App, cfg: EchoCellsConfig) {
    app.world_mut().insert_resource(cfg);
}

pub(super) fn add_echo_cells_stacks(app: &mut App, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(HazardKind::EchoCells);
    }
}

pub(super) fn activate_now(app: &mut App, tuning: &HazardTuning) {
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate(tuning, &mut commands);
    }
    queue.apply(app.world_mut());
}
