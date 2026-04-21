use std::{marker::PhantomData, time::Duration};

use bevy::{ecs::world::CommandQueue, prelude::*};

use super::super::system::*;
use crate::{
    cells::components::Cell,
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::ActiveHazards,
    },
    prelude::*,
    shared::death_pipeline::Destroyed,
};

pub(super) fn test_app_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message::<Destroyed<Cell>>()
        .build()
}

pub(super) fn test_app_not_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveHazards>()
        .with_message::<Destroyed<Cell>>()
        .build()
}

pub(super) const fn canonical_config() -> FractureConfig {
    FractureConfig {
        base_splits:      2,
        per_level_splits: 1,
    }
}

pub(super) fn install_fracture_config(app: &mut App, cfg: FractureConfig) {
    app.world_mut().insert_resource(cfg);
}

pub(super) fn add_fracture_stacks(app: &mut App, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(HazardKind::Fracture);
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

pub(super) fn write_cell_destroyed(app: &mut App, pos: Vec2) {
    app.world_mut()
        .resource_mut::<Messages<Destroyed<Cell>>>()
        .write(Destroyed::<Cell> {
            victim:     Entity::PLACEHOLDER,
            killer:     None,
            victim_pos: pos,
            killer_pos: None,
            _marker:    PhantomData,
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

pub(super) fn approx_eq_vec2(a: Vec2, b: Vec2, tol: f32) -> bool {
    (a - b).length() < tol
}
