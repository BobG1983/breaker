//! Shared test fixtures for Overcharge hazard tests.
//!
//! Mirrors Haste's helpers layout: state-hierarchy apps (`NodeState::Playing`
//! and a non-Playing variant), bolt/cell spawn helpers, fixed-timestep
//! tickers, `Destroyed<Cell>` / `BumpPerformed` writers, and
//! `ActiveHazards` / `OverchargeConfig` installers.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::super::system::{
    OverchargeConfig, OverchargeKillCount, overcharge_apply_speed, overcharge_count_kills,
    overcharge_reset_on_bump,
};
use crate::{
    bolt::components::Bolt,
    breaker::messages::{BumpGrade, BumpPerformed},
    cells::components::Cell,
    effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack},
    hazard::{definition::HazardKind, resources::ActiveHazards},
    prelude::*,
    shared::death_pipeline::Destroyed,
};

/// Default builder: state hierarchy driven into `NodeState::Playing`,
/// `ActiveHazards`, and both `Destroyed<Cell>` / `BumpPerformed` message
/// queues registered so every group's fixtures line up.
pub(super) fn test_app_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message::<Destroyed<Cell>>()
        .with_message::<BumpPerformed>()
        .build()
}

/// State-hierarchy app that is NOT driven into `NodeState::Playing`.
/// Mirrors the Haste `test_app_not_playing` pattern; used to test the
/// `in_state(NodeState::Playing)` run-condition gate in Group F.
pub(super) fn test_app_not_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveHazards>()
        .with_message::<Destroyed<Cell>>()
        .with_message::<BumpPerformed>()
        .build()
}

/// Spawns a `(Bolt,)` entity and returns its id.
pub(super) fn spawn_bolt(app: &mut App) -> Entity {
    app.world_mut().spawn(Bolt).id()
}

/// Spawns a `(Bolt, OverchargeKillCount(count))` entity. Used to pre-seed
/// a kill count without routing through `Destroyed<Cell>` messages.
pub(super) fn spawn_bolt_with_count(app: &mut App, count: u32) -> Entity {
    app.world_mut()
        .spawn((Bolt, OverchargeKillCount(count)))
        .id()
}

/// Spawns a `(Bolt, OverchargeKillCount(kills), EffectStack<SpeedBoostConfig>)`
/// entity with the given seed stack. Used by the reconcile/synergy tests.
pub(super) fn spawn_bolt_with_stack(
    app: &mut App,
    kills: u32,
    seed: EffectStack<SpeedBoostConfig>,
) -> Entity {
    app.world_mut()
        .spawn((Bolt, OverchargeKillCount(kills), seed))
        .id()
}

/// Spawns a `(Cell,)` entity and returns its id.
pub(super) fn spawn_cell(app: &mut App) -> Entity {
    app.world_mut().spawn(Cell).id()
}

/// Writes a `Destroyed<Cell>` message with zeroed positions and the given
/// victim/killer. Matches the helper originally inlined in `overcharge.rs`.
pub(super) fn write_cell_destroyed(app: &mut App, victim: Entity, killer: Option<Entity>) {
    app.world_mut()
        .resource_mut::<Messages<Destroyed<Cell>>>()
        .write(Destroyed::<Cell> {
            victim,
            killer,
            victim_pos: Vec2::ZERO,
            killer_pos: None,
            _marker: PhantomData,
        });
}

/// Writes a `BumpPerformed` message with the given bolt and a fixed
/// `BumpGrade::Perfect` grade (Overcharge ignores grade).
pub(super) fn write_bump(app: &mut App, bolt: Option<Entity>) {
    app.world_mut().write_message(BumpPerformed {
        grade: BumpGrade::Perfect,
        bolt,
        breaker: Entity::PLACEHOLDER,
    });
}

/// Runs one full `Time<Fixed>` timestep.
pub(super) fn run_fixed_update(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

/// Adds `count` Overcharge stacks to `ActiveHazards`.
pub(super) fn add_overcharge_stacks(app: &mut App, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(HazardKind::Overcharge);
    }
}

/// Inserts the given `OverchargeConfig` as a resource.
pub(super) fn install_overcharge_config(app: &mut App, cfg: OverchargeConfig) {
    app.world_mut().insert_resource(cfg);
}

/// Canonical tuning pair used across most tests:
/// `{ base_frac: 0.05, per_level_frac: 0.03 }`. Matches the design doc's
/// `5% + 3% * (stack - 1)` per-kill table.
pub(super) const fn canonical_config() -> OverchargeConfig {
    OverchargeConfig {
        base_frac:      0.05,
        per_level_frac: 0.03,
    }
}

/// Collects `(source, config)` entries from an `EffectStack<SpeedBoostConfig>`.
/// Used to assert source membership / cardinality / multiplier values.
pub(super) fn overcharge_entries(
    stack: &EffectStack<SpeedBoostConfig>,
) -> Vec<(String, SpeedBoostConfig)> {
    stack.iter().map(|(s, c)| (s.clone(), c.clone())).collect()
}

/// Wires only `overcharge_count_kills` in `FixedUpdate` — bypasses the
/// `hazard_active` / `in_state` gates `register` installs. Used by Group B.
pub(super) fn wire_count_only(app: &mut App) {
    app.add_systems(FixedUpdate, overcharge_count_kills);
}

/// Wires only `overcharge_reset_on_bump` — used by Group C.
pub(super) fn wire_reset_only(app: &mut App) {
    app.add_systems(FixedUpdate, overcharge_reset_on_bump);
}

/// Wires only `overcharge_apply_speed` — used by Group D.
pub(super) fn wire_apply_only(app: &mut App) {
    app.add_systems(FixedUpdate, overcharge_apply_speed);
}
