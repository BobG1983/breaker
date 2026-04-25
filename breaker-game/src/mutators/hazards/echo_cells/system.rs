use bevy::prelude::*;

use crate::{
    cells::components::{Cell, CellHeight, CellWidth},
    mutators::hazards::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
    shared::collision_layers::{BOLT_LAYER, CELL_LAYER},
};

/// Standard ghost dimensions — matches the default cell footprint used
/// across the project so ghosts collide like normal cells.
const GHOST_WIDTH: f32 = 70.0;
const GHOST_HEIGHT: f32 = 24.0;

/// Per-run tuning extracted from [`HazardTuning::EchoCells`].
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct EchoCellsConfig {
    /// Seconds between cell death and ghost spawn. Fixed — does not
    /// scale with stacks (intentional, per design).
    pub(crate) delay_secs:           f32,
    /// Ghost HP at stack 1.
    pub(crate) base_hp:              f32,
    /// Per-stack HP multiplier applied geometrically.
    pub(crate) per_level_multiplier: f32,
}

impl EchoCellsConfig {
    /// Ghost HP for the given stack count. Returns `0.0` when
    /// `stacks == 0` (hazard inactive / identity). For `stacks >= 1`,
    /// returns `base_hp * per_level_multiplier^(stacks - 1)` — geometric
    /// scaling in `stacks`, so stack 1 yields `base_hp`, stack 2 yields
    /// `base_hp * per_level_multiplier`, and so on. `saturating_sub` on
    /// the exponent prevents underflow when `stacks == 1` (extra = 0,
    /// so the multiplier term is `1.0` and the result is exactly
    /// `base_hp`). `#[must_use]` because callers must not drop the
    /// computed HP — it feeds directly into the ghost's `Hp`
    /// component. No upper cap — unlike Fracture's `splits_for`, ghost
    /// HP grows unboundedly with stack count, by design (the
    /// exponential curve is the steepest of all hazards).
    #[must_use]
    pub(crate) fn ghost_hp(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1).cast_signed();
        self.base_hp * self.per_level_multiplier.powi(extra)
    }
}

/// Marker entity spawned when a non-ghost cell dies. Carries the target
/// position and a countdown until the ghost materialises.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct PendingGhost {
    pub(crate) position: Vec2,
    pub(crate) timer:    f32,
}

/// Marks a spawned ghost cell. Prevents recursive ghost spawning and
/// lets other systems identify ghosts when needed.
#[derive(Component, Debug, Default, Clone, Copy)]
pub(crate) struct GhostCell;

/// Inserts `EchoCellsConfig` from `HazardTuning::EchoCells { delay_secs,
/// base_hp, per_level_multiplier }`. Called each time the player picks
/// Echo Cells from `hazards::activate`; last write wins (overwrites any
/// prior `EchoCellsConfig`; stack count is owned by `ActiveHazards`, not
/// the config; `PendingGhost` markers already in flight retain their
/// `timer` and `position` — the config only governs future ghost
/// spawns, i.e. the delay applied to new markers and the HP of future
/// ghost cells). Does not mutate `ActiveHazards`. Warns and no-ops on a
/// non-`EchoCells` tuning variant, leaving any existing
/// `EchoCellsConfig` intact.
pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::EchoCells {
        delay_secs,
        base_hp,
        per_level_multiplier,
    } = *tuning
    else {
        warn!("echo_cells::activate called with non-EchoCells tuning");
        return;
    };
    commands.insert_resource(EchoCellsConfig {
        delay_secs,
        base_hp,
        per_level_multiplier,
    });
}

/// Registers the two Echo Cells `FixedUpdate` systems
/// `echo_cells_track_deaths` and `echo_cells_spawn_ghosts`.
///
/// `echo_cells_track_deaths` is a reader system — it holds
/// `MessageReader<Destroyed<Cell>>`. Registered WITHOUT `.run_if(...)`;
/// it enforces the `ActiveHazards` / `NodeState::Playing` gate in-body
/// via an immediate `reader.clear()` + return when inactive, draining
/// the buffer every tick. A `.run_if(...)` gate suppresses execution
/// but does NOT advance the reader cursor, so messages buffered during
/// gated-off frames would get retroactively consumed the tick the gate
/// opens.
///
/// `echo_cells_spawn_ghosts` is a non-reader system (ticks
/// `PendingGhost` timers and spawns ghost `Cell` entities). It remains
/// gated on `hazard_active(HazardKind::EchoCells)` AND
/// `in_state(NodeState::Playing)`.
///
/// Ghost spawns are raw `commands.spawn(...)` calls rather than
/// death-pipeline emissions, so no `DmgSystems` ordering is
/// applied. The two systems operate on disjoint entity sets (the
/// tracker spawns new `PendingGhost` via deferred commands; the ghost
/// spawner reads existing `PendingGhost` entities), so explicit
/// `.chain()` ordering is no longer required.
pub(crate) fn register(app: &mut App) {
    app.add_systems(FixedUpdate, echo_cells_track_deaths)
        .add_systems(
            FixedUpdate,
            echo_cells_spawn_ghosts
                .run_if(hazard_active(HazardKind::EchoCells))
                .run_if(in_state(NodeState::Playing)),
        );
}

/// Reads `Destroyed<Cell>`; for each destroyed non-`GhostCell` victim,
/// spawns a `PendingGhost` marker carrying the victim's `victim_pos` and
/// `config.delay_secs` as the countdown. Victims that carry `GhostCell`
/// are skipped — ghost deaths must not recurse into new ghosts, so the
/// marker is not emitted for them.
///
/// Gated in-body: this system runs every `FixedUpdate` tick. When
/// `EchoCells` is not active or `NodeState` is not `Playing`, it drains
/// the `MessageReader` via `reader.clear()` and returns so buffered
/// `Destroyed<Cell>` messages cannot leak retroactively when the hazard
/// activates on a later frame. Early-returns (draining the reader via
/// `reader.clear()`) also when `EchoCellsConfig` is absent OR
/// `config.delay_secs <= 0.0`.
pub(crate) fn echo_cells_track_deaths(
    mut reader: MessageReader<Destroyed<Cell>>,
    active_hazards: Option<Res<ActiveHazards>>,
    node_state: Option<Res<State<NodeState>>>,
    config: Option<Res<EchoCellsConfig>>,
    ghosts: Query<(), With<GhostCell>>,
    mut commands: Commands,
) {
    if active_hazards
        .as_ref()
        .is_none_or(|ah| !ah.is_active(HazardKind::EchoCells))
        || node_state
            .as_ref()
            .is_none_or(|s| *s.get() != NodeState::Playing)
    {
        reader.clear();
        return;
    }
    let Some(config) = config else {
        reader.clear();
        return;
    };
    if config.delay_secs <= 0.0 {
        reader.clear();
        return;
    }
    for destroyed in reader.read() {
        if ghosts.contains(destroyed.victim) {
            // Ghost deaths must not recurse.
            continue;
        }
        commands.spawn((
            PendingGhost {
                position: destroyed.victim_pos,
                timer:    config.delay_secs,
            },
            // Node-end cleanup: in-flight `PendingGhost` entities do NOT
            // carry `Cell`, so unlike spawned ghosts they do not inherit
            // `CleanupOnExit<NodeState>` via `#[require]`. Must be added
            // explicitly or they leak across `NodeState` transitions.
            CleanupOnExit::<NodeState>::default(),
        ));
    }
}

/// Ticks every `PendingGhost.timer` by `time.delta_secs()` each
/// `FixedUpdate` frame. When a timer reaches `<= 0.0`, computes
/// `hp = config.ghost_hp(active.stacks(HazardKind::EchoCells))` once
/// before iterating and, if `hp > 0.0`, spawns a new entity with
/// `Cell`, `GhostCell`, `Position2D` at the marker's stored position,
/// `Scale2D { x: GHOST_WIDTH, y: GHOST_HEIGHT }`,
/// `Aabb2D::new(Vec2::ZERO, Vec2::new(GHOST_WIDTH / 2.0,
/// GHOST_HEIGHT / 2.0))`, `CollisionLayers::new(CELL_LAYER,
/// BOLT_LAYER)`, `CellWidth::new(GHOST_WIDTH)`,
/// `CellHeight::new(GHOST_HEIGHT)`, `Hp::new(hp)`, and
/// `KilledBy { killer: None }`. The `PendingGhost` marker is despawned in
/// either branch (even when `hp == 0.0`) — the marker is always
/// consumed on expiry so it does not retry on the next frame.
/// Early-returns when `EchoCellsConfig` is absent; no reader drain is
/// needed here, unlike the tracker, because this system does not read
/// a `MessageReader`. `CleanupOnExit<NodeState>` is NOT listed in the
/// spawn tuple — `Cell` carries `#[require(Spatial2D,
/// CleanupOnExit<NodeState>)]`, so ghosts inherit node-exit cleanup
/// automatically.
pub(crate) fn echo_cells_spawn_ghosts(
    time: Res<Time<Fixed>>,
    active: Res<ActiveHazards>,
    config: Option<Res<EchoCellsConfig>>,
    mut pending: Query<(Entity, &mut PendingGhost)>,
    mut commands: Commands,
) {
    let Some(config) = config else { return };
    let stacks = active.stacks(HazardKind::EchoCells);
    let hp = config.ghost_hp(stacks);
    let dt = time.delta_secs();

    for (entity, mut p) in &mut pending {
        p.timer -= dt;
        if p.timer > 0.0 {
            continue;
        }
        if hp > 0.0 {
            commands.spawn((
                Cell,
                GhostCell,
                Position2D(p.position),
                Scale2D {
                    x: GHOST_WIDTH,
                    y: GHOST_HEIGHT,
                },
                Aabb2D::new(Vec2::ZERO, Vec2::new(GHOST_WIDTH / 2.0, GHOST_HEIGHT / 2.0)),
                CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
                CellWidth::new(GHOST_WIDTH),
                CellHeight::new(GHOST_HEIGHT),
                Hp::new(hp),
                KilledBy { killer: None },
            ));
        }
        commands.entity(entity).despawn();
    }
}
