//! Shared test fixtures for Momentum hazard tests.
//!
//! App builders (Playing / not-Playing), cell spawn helpers with HP variants,
//! damage/heal message writers, `MomentumConfig` and `ActiveHazards` installers,
//! and a `FixedUpdate` ticker.

#![allow(
    dead_code,
    reason = "shared helpers; not every test file uses every helper"
)]

use std::{marker::PhantomData, time::Duration};

use bevy::{
    ecs::{message::Messages, world::CommandQueue},
    prelude::*,
};

use super::super::system::{MomentumConfig, activate};
use crate::{
    cells::components::Cell,
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::ActiveHazards,
    },
    prelude::*,
    shared::death_pipeline::{
        DamageDealt, Dead, HealCap, Hp, Invulnerable, KilledBy, heal_dealt::HealDealt,
    },
};

// ── App builders ────────────────────────────────────────────────────────────

/// Default builder: state hierarchy driven into `NodeState::Playing`,
/// `ActiveHazards`, `DamageDealt<Cell>` registered, `HealDealt<Cell>` captured.
pub(super) fn test_app_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message::<DamageDealt<Cell>>()
        .with_message_capture::<HealDealt<Cell>>()
        .build()
}

/// State-hierarchy app NOT driven into `Playing`. Used to test the
/// `in_state(NodeState::Playing)` gate.
pub(super) fn test_app_not_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveHazards>()
        .with_message::<DamageDealt<Cell>>()
        .with_message_capture::<HealDealt<Cell>>()
        .build()
}

// ── Config / stack helpers ──────────────────────────────────────────────────

/// Canonical design-doc config: base = 10.0, per-level = 10.0.
pub(super) const fn canonical_momentum_config() -> MomentumConfig {
    MomentumConfig {
        base_hp_per_hit:      10.0,
        per_level_hp_per_hit: 10.0,
    }
}

/// Inserts a `MomentumConfig` as a resource.
pub(super) fn install_momentum_config(app: &mut App, cfg: MomentumConfig) {
    app.world_mut().insert_resource(cfg);
}

/// Adds `count` Momentum stacks to `ActiveHazards`.
pub(super) fn add_momentum_stacks(app: &mut App, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(HazardKind::Momentum);
    }
}

/// Adds `count` stacks of the given hazard kind to `ActiveHazards`.
pub(super) fn add_hazard_stacks(app: &mut App, kind: HazardKind, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(kind);
    }
}

/// Invokes `activate` via a fresh `CommandQueue` so each call has an
/// independent, deterministic flush. Mirrors other hazards' `activate_now`.
pub(super) fn activate_now(app: &mut App, tuning: &HazardTuning) {
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate(tuning, &mut commands);
    }
    queue.apply(app.world_mut());
}

// ── Cell spawn helpers ──────────────────────────────────────────────────────

/// Spawns a cell with explicit `Hp { current, starting, max: None }`.
pub(super) fn spawn_cell_at(app: &mut App, pos: Vec2, current: f32, starting: f32) -> Entity {
    spawn_cell_at_with_max(app, pos, current, starting, None)
}

/// Spawns a cell with explicit `Hp` and optional `max`.
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
            KilledBy::default(),
        ))
        .id()
}

/// Spawns a cell with `Invulnerable` marker and explicit HP.
pub(super) fn spawn_cell_invulnerable_at(
    app: &mut App,
    pos: Vec2,
    current: f32,
    starting: f32,
) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            Hp {
                current,
                starting,
                max: None,
            },
            KilledBy::default(),
            Invulnerable,
        ))
        .id()
}

/// Spawns a cell with `Dead` marker and explicit HP.
pub(super) fn spawn_cell_dead_at(app: &mut App, pos: Vec2, current: f32, starting: f32) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            Hp {
                current,
                starting,
                max: None,
            },
            KilledBy::default(),
            Dead,
        ))
        .id()
}

// ── Message writers ─────────────────────────────────────────────────────────

/// Writes a `DamageDealt<Cell>` targeting `target` for `amount`.
pub(super) fn write_cell_damage(app: &mut App, target: Entity, amount: f32) {
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer: None,
            target,
            amount,
            source_chip: None,
            _marker: PhantomData,
        });
}

/// Writes a `HealDealt<Cell>` targeting `target` for `amount` with
/// `HealCap::Max` and no source attribution.
pub(super) fn write_cell_heal_max(app: &mut App, target: Entity, amount: f32) {
    app.world_mut()
        .resource_mut::<Messages<HealDealt<Cell>>>()
        .write(HealDealt::<Cell> {
            healer: None,
            target,
            amount,
            cap: HealCap::Max,
            source: None,
            _marker: PhantomData,
        });
}

// ── Tickers ─────────────────────────────────────────────────────────────────

/// Runs exactly one `FixedUpdate` tick with the default timestep.
pub(super) fn run_fixed_update(app: &mut App) {
    tick(app);
}

/// Runs exactly one `FixedUpdate` tick with a specific `dt`.
pub(super) fn tick_with_dt(app: &mut App, dt: Duration) {
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .set_timestep(dt);
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(dt);
    app.update();
}

// ── Assertion helpers ───────────────────────────────────────────────────────

/// Returns every captured `HealDealt<Cell>` targeted at `target`.
pub(super) fn heals_for_cell(app: &App, target: Entity) -> Vec<HealDealt<Cell>> {
    app.world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0
        .iter()
        .filter(|m| m.target == target)
        .cloned()
        .collect()
}

/// Returns the total number of captured `HealDealt<Cell>` messages.
pub(super) fn heal_collector_len(app: &App) -> usize {
    app.world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0
        .len()
}

/// Returns every spawned `Cell` entity with its `Position2D` and `Hp`.
pub(super) fn all_cells(app: &mut App) -> Vec<(Entity, Vec2, Hp)> {
    let mut q = app.world_mut().query::<(Entity, &Position2D, &Hp, &Cell)>();
    q.iter(app.world())
        .map(|(e, p, hp, _)| (e, p.0, hp.clone()))
        .collect()
}

/// Count of live `Cell` entities in the world.
pub(super) fn cell_count(app: &mut App) -> usize {
    let mut q = app.world_mut().query::<&Cell>();
    q.iter(app.world()).count()
}

/// Approximately equal check for `Vec2`.
pub(super) fn approx_eq_vec2(a: Vec2, b: Vec2, tol: f32) -> bool {
    (a - b).length() < tol
}

// ── NodeState-driver for mid-run activation ─────────────────────────────────

/// Drives a not-yet-playing app all the way into `NodeState::Playing` without
/// rebuilding. Used when a test builds a not-playing app but later needs the
/// system's `in_state(NodeState::Playing)` gate to flip.
pub(super) fn enter_playing(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Game);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Run);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<RunState>>()
        .set(RunState::Node);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();
}
