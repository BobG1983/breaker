//! Shared test fixtures for Renewal hazard tests. Mirrors Cascade's helper
//! block: state-hierarchy app at `NodeState::Playing`, `ActiveHazards`, and
//! `MessageCollector<HealDealt<Cell>>` capture.

use std::time::Duration;

use bevy::prelude::*;

use super::super::system::{RenewalConfig, RenewalTimer};
use crate::{
    cells::components::Cell,
    mutators::hazards::{definition::HazardKind, resources::ActiveHazards},
    prelude::*,
};

/// Default builder: state hierarchy at `NodeState::Playing`, `ActiveHazards`,
/// `HealDealt<Cell>` captured.
pub(super) fn test_app_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message_capture::<HealDealt<Cell>>()
        .build()
}

/// Spawns a `Cell` with the given `current` / `starting` HP and `max = None`.
pub(super) fn spawn_cell(app: &mut App, current: f32, starting: f32) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Hp {
                current,
                starting,
                max: None,
            },
        ))
        .id()
}

/// Spawns a `Cell` with an explicit `max` upper bound. Callers pass a bare
/// `f32` — the helper wraps it internally as `Some(max)`.
pub(super) fn spawn_cell_with_max(app: &mut App, current: f32, starting: f32, max: f32) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Hp {
                current,
                starting,
                max: Some(max),
            },
        ))
        .id()
}

/// Sets `Time<Fixed>` timestep to `dt`, accumulates `dt` of overstep,
/// runs one `app.update()`.
pub(super) fn tick_with_dt(app: &mut App, dt: Duration) {
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .set_timestep(dt);
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(dt);
    app.update();
}

/// Filters the `MessageCollector<HealDealt<Cell>>` for messages targeting
/// `target`. Returns clones of each matching message.
pub(super) fn heals_for_cell(app: &App, target: Entity) -> Vec<HealDealt<Cell>> {
    app.world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0
        .iter()
        .filter(|m| m.target == target)
        .cloned()
        .collect()
}

/// Length of the `MessageCollector<HealDealt<Cell>>`.
pub(super) fn heal_collector_len(app: &App) -> usize {
    app.world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0
        .len()
}

/// Adds `count` Renewal stacks to `ActiveHazards`.
pub(super) fn add_renewal_stacks(app: &mut App, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(HazardKind::Renewal);
    }
}

/// Inserts the given `RenewalConfig` as a resource.
pub(super) fn install_renewal_config(app: &mut App, cfg: RenewalConfig) {
    app.world_mut().insert_resource(cfg);
}

/// Inserts a `RenewalTimer { remaining }` on an existing cell entity.
pub(super) fn attach_timer(app: &mut App, cell: Entity, remaining: f32) {
    app.world_mut()
        .entity_mut(cell)
        .insert(RenewalTimer { remaining });
}

/// The canonical config used by most tests.
pub(super) const fn canonical_config() -> RenewalConfig {
    RenewalConfig {
        base_period_secs:         10.0,
        per_level_reduction_frac: 0.2,
    }
}
