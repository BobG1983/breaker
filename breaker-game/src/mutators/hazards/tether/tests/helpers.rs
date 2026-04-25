//! Shared test fixtures for Tether hazard tests.
//!
//! State-hierarchy apps (driven and not-yet-driven into `NodeState::Playing`),
//! cell-row / cell-grid spawn helpers, link-pair builders, a seeded `GameRng`
//! installer, a fixed-step ticker, and `ActiveHazards` / `TetherConfig`
//! installers.

use std::collections::BTreeSet;

use bevy::{ecs::world::CommandQueue, prelude::*};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use super::super::system::{TetherConfig, TetherLink, activate, wire};
use crate::{
    mutators::hazards::{
        definition::{HazardKind, HazardTuning},
        resources::ActiveHazards,
    },
    prelude::*,
};

// ── App builders ────────────────────────────────────────────────────────────

/// Default builder: state hierarchy NOT yet in `Playing`, `ActiveHazards`,
/// `DamageDealt<Cell>` message registered, `wire` wired.
///
/// Tests seed `GameRng`, add Tether stacks, install `TetherConfig`, spawn
/// cells, then drive the state into `NodeState::Playing` to fire `OnEnter`.
pub(super) fn build_establish_tether_app(seed: u64) -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveHazards>()
        .with_message::<DamageDealt<Cell>>()
        .build();
    app.world_mut()
        .insert_resource(GameRng(ChaCha8Rng::seed_from_u64(seed)));
    wire(&mut app);
    app
}

/// Same as `build_establish_tether_app` but without the seeded `GameRng`.
pub(super) fn build_establish_tether_app_no_rng() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveHazards>()
        .with_message::<DamageDealt<Cell>>()
        .build();
    wire(&mut app);
    app
}

/// State-hierarchy app driven into `NodeState::Playing` with `ActiveHazards`,
/// canonical `TetherConfig`, 1 Tether stack, seeded `GameRng`, `wire`
/// wired, and `MessageCollector<DamageDealt<Cell>>` installed. Intended for
/// cleanup-system tests that do not need the establish step.
pub(super) fn build_cleanup_tether_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut()
        .insert_resource(GameRng(ChaCha8Rng::seed_from_u64(42)));
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);
    wire(&mut app);
    app
}

/// State-hierarchy app NOT driven into `Playing`. Used to test the
/// `in_state(NodeState::Playing)` gate in Section D and Section F.
pub(super) fn build_cleanup_tether_app_not_playing() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveHazards>()
        .with_message::<DamageDealt<Cell>>()
        .build();
    app.world_mut()
        .insert_resource(GameRng(ChaCha8Rng::seed_from_u64(42)));
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);
    wire(&mut app);
    app
}

// ── Config / stack helpers ──────────────────────────────────────────────────

/// Canonical tuning: `{ 25.0, 10.0, 40.0, 10.0 }` (design-doc values).
pub(super) const fn canonical_tether_config() -> TetherConfig {
    TetherConfig {
        base_damage:        25.0,
        damage_per_level:   10.0,
        base_coverage:      40.0,
        coverage_per_level: 10.0,
    }
}

/// Inserts the given `TetherConfig` as a resource.
pub(super) fn install_tether_config(app: &mut App, cfg: TetherConfig) {
    app.world_mut().insert_resource(cfg);
}

/// Adds `count` Tether stacks to `ActiveHazards`.
pub(super) fn add_tether_stacks(app: &mut App, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(HazardKind::Tether);
    }
}

/// Adds `count` stacks of the given hazard kind to `ActiveHazards`.
pub(super) fn add_hazard_stacks(app: &mut App, kind: HazardKind, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(kind);
    }
}

/// Invokes `activate` directly via a fresh `CommandQueue` so each call has
/// an independent, deterministic flush. Mirrors the `activate_now` helper in
/// other hazards.
pub(super) fn activate_now(app: &mut App, tuning: &HazardTuning) {
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate(tuning, &mut commands);
    }
    queue.apply(app.world_mut());
}

// ── Cell spawn helpers ──────────────────────────────────────────────────────

/// Spawns `count` cells at `(i * spacing, 0.0)` for `i in 0..count`. Returns
/// the entities in column-index order.
pub(super) fn spawn_cell_row(app: &mut App, count: usize, spacing: f32) -> Vec<Entity> {
    (0..count)
        .map(|i| {
            app.world_mut()
                .spawn((
                    Cell,
                    Position2D(Vec2::new(i as f32 * spacing, 0.0)),
                    Hp::new(100.0),
                    KilledBy { killer: None },
                ))
                .id()
        })
        .collect()
}

/// Spawns a single cell at a given position with 100 HP. Used by Section D.
pub(super) fn spawn_cell_at(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            Hp::new(100.0),
            KilledBy { killer: None },
        ))
        .id()
}

/// Spawns a cell with the `Invulnerable` marker at a given position.
pub(super) fn spawn_cell_invulnerable_at(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            Hp::new(100.0),
            KilledBy { killer: None },
            Invulnerable,
        ))
        .id()
}

/// Spawns a cell with the `Dead` marker at a given position.
pub(super) fn spawn_cell_dead_at(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            Hp::new(100.0),
            KilledBy { killer: None },
            Dead,
        ))
        .id()
}

/// Spawns two cells at `pos_a` / `pos_b` with mutual `TetherLink` components.
pub(super) fn spawn_linked_pair(app: &mut App, pos_a: Vec2, pos_b: Vec2) -> (Entity, Entity) {
    let a = spawn_cell_at(app, pos_a);
    let b = spawn_cell_at(app, pos_b);
    app.world_mut()
        .entity_mut(a)
        .insert(TetherLink { partner: b });
    app.world_mut()
        .entity_mut(b)
        .insert(TetherLink { partner: a });
    (a, b)
}

// ── Tickers / state drivers ─────────────────────────────────────────────────

/// Drives the state hierarchy into `NodeState::Playing` and fires the
/// `OnEnter(Playing)` schedule (via the normal state-transition machinery).
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

/// Runs exactly one `FixedUpdate` tick.
pub(super) fn run_fixed_update(app: &mut App) {
    tick(app);
}

// ── Assertions / counters ───────────────────────────────────────────────────

/// Returns the deduplicated set of `(col_a, col_b)` column-index pairs of
/// linked cells. `col = round(pos.x / 50.0)`. Used for determinism
/// comparisons across apps with different `Entity` ids.
pub(super) fn link_partners(app: &mut App) -> BTreeSet<(usize, usize)> {
    let mut links_q = app
        .world_mut()
        .query::<(Entity, &TetherLink, &Position2D)>();
    let rows: Vec<(Entity, Entity, Vec2)> = links_q
        .iter(app.world())
        .map(|(e, link, pos)| (e, link.partner, pos.0))
        .collect();

    // Map each entity to its column index via its own position.
    let mut pos_q = app.world_mut().query::<(Entity, &Position2D)>();
    let mut col_of = std::collections::HashMap::<Entity, usize>::new();
    for (e, pos) in pos_q.iter(app.world()) {
        let col = (pos.0.x / 50.0).round() as usize;
        col_of.insert(e, col);
    }

    let mut out = BTreeSet::new();
    for (e, partner, _pos) in rows {
        let a = *col_of.get(&e).expect("col index for entity");
        let b = *col_of.get(&partner).expect("col index for partner");
        let pair = if a < b { (a, b) } else { (b, a) };
        out.insert(pair);
    }
    out
}
