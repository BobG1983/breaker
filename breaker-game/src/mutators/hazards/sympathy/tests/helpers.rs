//! Shared test fixtures for Sympathy hazard tests.
//!
//! App builders (Playing / not-Playing), cell spawn helpers (live / Dead /
//! Invulnerable), damage/heal message writers, `SympathyConfig` and
//! `ActiveHazards` installers, a `FixedUpdate` ticker, and result filters.

#![allow(
    dead_code,
    reason = "shared helpers; not every test file uses every helper"
)]

use std::marker::PhantomData;

use bevy::{
    ecs::{message::Messages, world::CommandQueue},
    prelude::*,
};

use super::super::system::{SympathyConfig, activate};
use crate::{
    cells::components::Cell,
    mutators::hazards::{
        definition::{HazardKind, HazardTuning},
        resources::ActiveHazards,
    },
    prelude::{HealDealt, *},
};

// ── App builders ────────────────────────────────────────────────────────────

/// State hierarchy driven into `NodeState::Playing`, `ActiveHazards`
/// installed, `DamageDealt<Cell>` registered, `HealDealt<Cell>` captured
/// via a `MessageCollector`.
pub(super) fn test_app_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message::<DamageDealt<Cell>>()
        .with_message_capture::<HealDealt<Cell>>()
        .build()
}

/// State hierarchy NOT in `Playing`. Used to test the
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

/// Canonical design-doc config: base 25%, per-level 5%, depth interval 5.
pub(super) const fn canonical_sympathy_config() -> SympathyConfig {
    SympathyConfig {
        base_heal_percent:       25.0,
        heal_per_level_percent:  5.0,
        depth_increase_interval: 5,
    }
}

/// Inserts the given `SympathyConfig` as a resource.
pub(super) fn install_sympathy_config(app: &mut App, cfg: SympathyConfig) {
    app.world_mut().insert_resource(cfg);
}

/// Adds `count` Sympathy stacks to `ActiveHazards`.
pub(super) fn add_sympathy_stacks(app: &mut App, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(HazardKind::Sympathy);
    }
}

/// Adds `count` stacks of any `HazardKind` for run-if-gate tests.
pub(super) fn add_hazard_stacks(app: &mut App, kind: HazardKind, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(kind);
    }
}

/// Invokes `activate` via a fresh `CommandQueue` so each call has an
/// independent, deterministic flush.
pub(super) fn activate_now(app: &mut App, tuning: &HazardTuning) {
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate(tuning, &mut commands);
    }
    queue.apply(app.world_mut());
}

// ── Cell spawn helpers ──────────────────────────────────────────────────────

/// Spawns a `Cell` at `pos` with explicit `Hp { current, starting, max: None }`.
pub(super) fn spawn_cell_at(app: &mut App, pos: Vec2, current: f32, starting: f32) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            Hp {
                current,
                starting,
                max: None,
            },
            KilledBy { killer: None },
        ))
        .id()
}

/// Convenience — calls [`spawn_cell_at`] with `current = starting = 50.0`.
pub(super) fn spawn_cell_at_default(app: &mut App, pos: Vec2) -> Entity {
    spawn_cell_at(app, pos, 50.0, 50.0)
}

/// Spawns a `Cell` at `pos` with `Hp::new(starting)` and `max = Some(max)`.
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
            KilledBy { killer: None },
        ))
        .id()
}

/// Spawns a `Cell` with the `Dead` marker. HP defaults to `(50.0, 50.0)`.
pub(super) fn spawn_cell_dead_at(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            Hp {
                current:  50.0,
                starting: 50.0,
                max:      None,
            },
            KilledBy { killer: None },
            Dead,
        ))
        .id()
}

/// Spawns a `Cell` with the `Invulnerable` marker. HP defaults to `(50.0, 50.0)`.
pub(super) fn spawn_cell_invulnerable_at(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            Hp {
                current:  50.0,
                starting: 50.0,
                max:      None,
            },
            KilledBy { killer: None },
            Invulnerable,
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
            attributed_to: None,
            target,
            amount,
            source: None,
            _marker: PhantomData,
        });
}

// ── Tickers ─────────────────────────────────────────────────────────────────

/// Runs exactly one `FixedUpdate` tick with the default timestep.
pub(super) fn run_fixed_update(app: &mut App) {
    tick(app);
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

/// All captured `HealDealt<Cell>` messages, in order.
pub(super) fn all_heals(app: &App) -> Vec<HealDealt<Cell>> {
    app.world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0
        .clone()
}

/// Drives a not-yet-playing app all the way into `NodeState::Playing`.
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
