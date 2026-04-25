use std::marker::PhantomData;

use bevy::{ecs::message::Messages, prelude::*};
use rantzsoft_dmg::{RantzDmgAppExt, RantzDmgPlugin};
use rantzsoft_spatial2d::components::Position2D;

use super::super::system::*;
use crate::{
    cells::components::Cell,
    mutators::hazards::{definition::HazardKind, resources::ActiveHazards},
    prelude::*,
};

/// Default builder: state hierarchy at `NodeState::Playing`, `ActiveHazards`,
/// `Destroyed<Cell>` registered, `HealDealt<Cell>` captured.
pub(super) fn test_app_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message::<Destroyed<Cell>>()
        .with_message_capture::<HealDealt<Cell>>()
        .build()
}

pub(super) fn spawn_cell_at(app: &mut App, pos: Vec2, current: f32, starting: f32) -> Entity {
    spawn_cell_at_with_max(app, pos, current, starting, None)
}

pub(super) fn spawn_cell_at_with_max(
    app: &mut App,
    pos: Vec2,
    current: f32,
    starting: f32,
    max: Option<f32>,
) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            Hp {
                current,
                starting,
                max,
            },
        ))
        .id()
}

pub(super) fn send_cell_destroyed(app: &mut App, victim: Entity, victim_pos: Vec2) {
    app.world_mut()
        .resource_mut::<Messages<Destroyed<Cell>>>()
        .write(Destroyed::<Cell> {
            victim,
            killer: None,
            victim_pos,
            killer_pos: None,
            _marker: PhantomData,
        });
}

pub(super) fn run_fixed_update(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

pub(super) fn heals_for_cell(app: &App, target: Entity) -> Vec<HealDealt<Cell>> {
    app.world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0
        .iter()
        .filter(|m| m.target == target)
        .cloned()
        .collect()
}

pub(super) fn cascade_heal_collector_len(app: &App) -> usize {
    app.world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0
        .len()
}

pub(super) fn add_cascade_stacks(app: &mut App, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(HazardKind::Cascade);
    }
}

pub(super) fn install_cascade_config(app: &mut App, cfg: CascadeConfig) {
    app.world_mut().insert_resource(cfg);
}

/// Builder for Group G: wires `cascade_heal_on_death` before
/// `apply_heal::<Cell>` in `DmgSystems::ApplyHeal`.
///
/// `register_dmgable::<Cell>()` wires the crate's generic `apply_heal::<Cell>`
/// into `DmgSystems::ApplyHeal` automatically.
pub(super) fn test_app_pipeline() -> App {
    let mut app = test_app_playing();
    app.add_plugins(RantzDmgPlugin);
    let _ = app.register_dmgable::<Cell>();
    app.add_systems(
        FixedUpdate,
        cascade_heal_on_death.before(DmgSystems::ApplyHeal),
    );
    app
}
