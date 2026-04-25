//! Momentum hazard — non-lethal damage bolsters cell HP; cells split at 2x starting HP.
//!
//! Design doc: `docs/design/hazards/momentum.md`.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    cells::components::{CellHeight, CellWidth},
    mutators::hazards::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
    shared::collision_layers::{BOLT_LAYER, CELL_LAYER},
};

/// Split HP multiplier — a cell splits when
/// `hp.current >= hp.starting * MOMENTUM_SPLIT_MULTIPLIER`.
/// Design-doc authoritative: 2.0 (cells split at 2x their starting HP).
pub(crate) const MOMENTUM_SPLIT_MULTIPLIER: f32 = 2.0;

/// Split-cell width. Matches Fracture debris — the pristine cell footprint.
pub(crate) const MOMENTUM_CELL_WIDTH: f32 = 70.0;

/// Split-cell height. Matches Fracture debris.
pub(crate) const MOMENTUM_CELL_HEIGHT: f32 = 24.0;

/// Cardinal offsets at which split cells attempt to spawn, in the order
/// checked (right, left, up, down). Up to two empty slots are used per split.
const MOMENTUM_SPLIT_OFFSETS: [Vec2; 4] = [
    Vec2::new(MOMENTUM_CELL_WIDTH, 0.0),
    Vec2::new(-MOMENTUM_CELL_WIDTH, 0.0),
    Vec2::new(0.0, MOMENTUM_CELL_HEIGHT),
    Vec2::new(0.0, -MOMENTUM_CELL_HEIGHT),
];

/// Maximum new cells spawned per split. Design-doc authoritative: 2.
const MOMENTUM_SPLIT_MAX_NEW_CELLS: usize = 2;

/// Squared occupancy radius for the "is a cell AT this candidate slot" check.
///
/// Set to `MOMENTUM_CELL_HEIGHT.pow(2) = 576.0` — tighter than the cells
/// domain's `ADJACENCY_RADIUS_SQ` (4900.0). Must stay strictly less than
/// `(2 * MOMENTUM_CELL_HEIGHT).pow(2) = 2304.0` so the up-slot cell at
/// `(0, +24)` does not falsely block the down candidate at `(0, -24)`
/// (separation 48.0 → 2304.0). The inclusive `<=` boundary is kept so a
/// cell at the exact cardinal position registers as occupying it (distance
/// 0.0 ≤ 576.0).
const MOMENTUM_OCCUPANCY_RADIUS_SQ: f32 = MOMENTUM_CELL_HEIGHT * MOMENTUM_CELL_HEIGHT;

// ── MomentumConfig ──────────────────────────────────────────────────────────

/// Per-run Momentum tuning extracted from [`HazardTuning::Momentum`] at
/// activation time. Structure mirrors `HazardTuning::Momentum` exactly —
/// exactly two fields. The split-threshold multiplier is the module-level
/// [`MOMENTUM_SPLIT_MULTIPLIER`] const, not stored here.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct MomentumConfig {
    /// HP healed to a cell per non-lethal hit at stack 1.
    pub(crate) base_hp_per_hit:      f32,
    /// Additional HP healed per stack beyond the first.
    pub(crate) per_level_hp_per_hit: f32,
}

impl MomentumConfig {
    /// HP healed per non-lethal hit for the given stack count.
    /// Returns 0.0 when `stacks == 0` (short-circuits arithmetic).
    /// For `stacks >= 1`: `base_hp_per_hit + per_level_hp_per_hit * (stacks - 1)`.
    #[must_use]
    pub(crate) const fn heal_per_hit(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        self.per_level_hp_per_hit
            .mul_add(extra, self.base_hp_per_hit)
    }
}

// ── activate ────────────────────────────────────────────────────────────────

/// Inserts `MomentumConfig` from `HazardTuning::Momentum`. Called each time
/// the player picks Momentum; last write wins (overwrites any prior config).
/// Warns and no-ops on a non-Momentum tuning variant.
pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Momentum {
        base_hp_per_hit,
        per_level_hp_per_hit,
    } = *tuning
    else {
        warn!("momentum::activate called with non-Momentum tuning");
        return;
    };
    commands.insert_resource(MomentumConfig {
        base_hp_per_hit,
        per_level_hp_per_hit,
    });
}

// ── wire ────────────────────────────────────────────────────────────────

/// Registers Momentum's `FixedUpdate` systems.
///
/// Non-reader systems — gated by `hazard_active(HazardKind::Momentum)`
/// AND `in_state(NodeState::Playing)`:
/// - `attach_momentum_ceiling` — before `momentum_heal_on_nonlethal`.
/// - `momentum_split_check` — after `DmgSystems::ApplyHeal`.
///
/// Reader system — intentionally ungated at the tuple level:
/// - `momentum_heal_on_nonlethal` — holds `MessageReader<DamageDealt<Cell>>`,
///   enforces the `ActiveHazards` / `NodeState::Playing` gate in-body via
///   an immediate `reader.clear()` + return when inactive. A `.run_if(...)`
///   gate suppresses execution but does NOT advance the reader cursor, so
///   messages buffered during gated-off frames would get retroactively
///   consumed the tick the gate opens. Runs in
///   `DmgSystems::ApplyHeal`.
pub(crate) fn wire(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        attach_momentum_ceiling
            .before(momentum_heal_on_nonlethal)
            .run_if(hazard_active(HazardKind::Momentum))
            .run_if(in_state(NodeState::Playing)),
    );
    app.add_systems(
        FixedUpdate,
        momentum_heal_on_nonlethal.in_set(DmgSystems::PostApplyDamage),
    );
    app.add_systems(
        FixedUpdate,
        momentum_split_check
            .after(DmgSystems::ApplyHeal)
            .run_if(hazard_active(HazardKind::Momentum))
            .run_if(in_state(NodeState::Playing)),
    );
}

// Shared filter used by every Momentum query that cares about "live, participating
// cells" — excludes Dead and Invulnerable markers.
type LiveCellFilter = (With<Cell>, Without<Dead>, Without<Invulnerable>);

type CellHpMut<'w, 's> = Query<'w, 's, &'static mut Hp, LiveCellFilter>;
type CellHpRead<'w, 's> = Query<'w, 's, &'static Hp, LiveCellFilter>;
type CellSplitParents<'w, 's> =
    Query<'w, 's, (Entity, &'static Position2D, &'static mut Hp), LiveCellFilter>;
type CellOccupants<'w, 's> =
    Query<'w, 's, (Entity, &'static Position2D), (With<Cell>, Without<Dead>)>;

// ── attach_momentum_ceiling ─────────────────────────────────────────────────

/// Lifts `hp.max` on every eligible cell to at least
/// `hp.starting * MOMENTUM_SPLIT_MULTIPLIER`. Mirrors Volatility's
/// `attach_volatility_timers` pattern.
///
/// Runs BEFORE `momentum_heal_on_nonlethal` so that by the time
/// `apply_heal::<Cell>` processes a Momentum heal, the ceiling is already
/// high enough for `HealCap::Max` to let the heal land past `hp.starting`.
///
/// Skips cells with `hp.starting <= 0.0` to avoid writing `hp.max = Some(0.0)`
/// which would permanently clamp all future heals at 0.
pub(crate) fn attach_momentum_ceiling(config: Option<Res<MomentumConfig>>, mut cells: CellHpMut) {
    // `_config` is the presence-gate: if the hazard hasn't activated, skip.
    // The split-threshold multiplier comes from `MOMENTUM_SPLIT_MULTIPLIER`
    // (module-level const), not from any `MomentumConfig` field, so none of
    // the config's fields are read here.
    let Some(_config) = config else { return };
    // Direct reads + guarded write, matching `attach_volatility_timers` exactly
    // (see `hazard/hazards/volatility.rs:101-120`). Reading through `.` does
    // NOT trigger `Changed<Hp>`; only the guarded assignment does.
    for mut hp in &mut cells {
        if hp.starting <= 0.0 {
            continue;
        }
        let target = hp.starting * MOMENTUM_SPLIT_MULTIPLIER;
        let new_max = Some(hp.max.map_or(target, |m| m.max(target)));
        if hp.max != new_max {
            hp.max = new_max;
        }
    }
}

// ── momentum_heal_on_nonlethal ──────────────────────────────────────────────

/// Reads every `DamageDealt<Cell>` this tick and, for each message whose
/// target is a live cell with `hp.current > 0.0` (the damage was non-lethal),
/// emits one `HealDealt<Cell>` with the stack-scaled heal amount,
/// `HealCap::Max`, and the builder-produced `"hazard:momentum"` source tag.
///
/// Does NOT mutate `hp.max` — that is `attach_momentum_ceiling`'s responsibility.
///
/// Gated in-body: this system runs every `FixedUpdate` tick. When
/// Momentum is not active or `NodeState` is not `Playing`, it drains the
/// `MessageReader` via `reader.clear()` and returns so buffered
/// `DamageDealt<Cell>` messages cannot leak retroactively when the
/// hazard activates on a later frame.
pub(crate) fn momentum_heal_on_nonlethal(
    mut reader: MessageReader<DamageDealt<Cell>>,
    active_hazards: Option<Res<ActiveHazards>>,
    node_state: Option<Res<State<NodeState>>>,
    config: Option<Res<MomentumConfig>>,
    targets: CellHpRead,
    mut writer: MessageWriter<HealDealt<Cell>>,
) {
    if active_hazards
        .as_ref()
        .is_none_or(|ah| !ah.is_active(HazardKind::Momentum))
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
    let Some(active) = active_hazards else {
        reader.clear();
        return;
    };
    let stacks = active.stacks(HazardKind::Momentum);
    let heal = config.heal_per_hit(stacks);
    if heal <= 0.0 {
        reader.clear();
        return;
    }
    let source = SourceId::hazard(HazardKind::Momentum).build();

    for msg in reader.read() {
        // Zero- or negative-magnitude damage is a no-op — matches Cascade's
        // pattern for zero-magnitude events. Prevents phantom heals from
        // `DamageDealt` messages with `amount == 0.0`.
        if msg.amount <= 0.0 {
            continue;
        }
        let Ok(hp) = targets.get(msg.target) else {
            continue;
        };
        if hp.current <= 0.0 {
            continue;
        }

        writer.write(HealDealt::<Cell> {
            healer:        None,
            attributed_to: None,
            target:        msg.target,
            amount:        heal,
            source:        Some(source.clone()),
            cap:           HealCap::Max,
            _marker:       PhantomData,
        });
    }
}

// ── momentum_split_check ────────────────────────────────────────────────────

/// Scans every live cell with `hp.current >= hp.starting * MOMENTUM_SPLIT_MULTIPLIER`
/// and attempts to spawn up to 2 new basic cells at cardinal offsets
/// (right, left, up, down — first-two-empty).
///
/// When at least 1 cell is spawned, the parent's `hp.current` resets to
/// `hp.starting`; `hp.max` is NOT reset. When 0 cardinal slots are empty, the
/// split does not fire and the parent retains its over-threshold HP (re-checked
/// next tick — self-regulating).
///
/// Guards against `hp.starting <= 0.0` to avoid infinite-spawn loops on
/// degenerate cells.
pub(crate) fn momentum_split_check(
    config: Option<Res<MomentumConfig>>,
    mut parents: CellSplitParents,
    occupants: CellOccupants,
    mut commands: Commands,
) {
    let Some(_config) = config else { return };

    // Step 1: collect split candidates (release the parents borrow before mutation).
    let mut splits: Vec<(Entity, Vec2, f32)> = Vec::new();
    for (entity, &Position2D(pos), hp) in &parents {
        if hp.starting <= 0.0 {
            continue;
        }
        let threshold = hp.starting * MOMENTUM_SPLIT_MULTIPLIER;
        if hp.current >= threshold {
            splits.push((entity, pos, hp.starting));
        }
    }

    if splits.is_empty() {
        return;
    }

    // Step 2: snapshot live occupant entities + positions BEFORE any spawn this
    // tick. Entity is captured so each split can exclude its own parent — the
    // parent sits at every offset's base position, so it would otherwise
    // self-block its own spawn slots (distance 70.0 → 4900.0 in the horizontal
    // cardinals, still within any reasonable `CellHeight`-based occupancy
    // radius for the vertical cardinals).
    let live_positions: Vec<(Entity, Vec2)> = occupants.iter().map(|(e, p)| (e, p.0)).collect();

    // Step 3: for each split candidate, attempt up to 2 spawns at empty cardinals.
    for (parent_entity, parent_pos, parent_starting) in splits {
        let mut spawned = 0usize;
        for offset in &MOMENTUM_SPLIT_OFFSETS {
            if spawned >= MOMENTUM_SPLIT_MAX_NEW_CELLS {
                break;
            }
            let candidate = parent_pos + *offset;
            // STRICT `<` — a diagonal neighbour at exactly
            // `MOMENTUM_CELL_HEIGHT` distance from a candidate (e.g., parent
            // at (0,0), diagonal occupant at (70,24), right candidate at
            // (70,0) — distance² == 576 == MOMENTUM_OCCUPANCY_RADIUS_SQ)
            // would else falsely block the candidate under `<=`.
            // reviewer-correctness 2026-04-19.
            let occupied = live_positions.iter().any(|(e, p)| {
                *e != parent_entity && p.distance_squared(candidate) < MOMENTUM_OCCUPANCY_RADIUS_SQ
            });
            if occupied {
                continue;
            }
            commands.spawn((
                Cell,
                Position2D(candidate),
                Scale2D {
                    x: MOMENTUM_CELL_WIDTH,
                    y: MOMENTUM_CELL_HEIGHT,
                },
                Aabb2D::new(
                    Vec2::ZERO,
                    Vec2::new(MOMENTUM_CELL_WIDTH / 2.0, MOMENTUM_CELL_HEIGHT / 2.0),
                ),
                CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
                CellWidth::new(MOMENTUM_CELL_WIDTH),
                CellHeight::new(MOMENTUM_CELL_HEIGHT),
                Hp::new(parent_starting),
                KilledBy { killer: None },
            ));
            spawned += 1;
        }

        // Reset parent current HP only if we actually spawned at least one cell.
        // When `spawned == 0` the parent retains its over-threshold HP and the
        // check re-runs next tick (design-doc: self-regulating).
        //
        // `hp.max` intentionally NOT reset — `attach_momentum_ceiling` keeps it
        // at target; resetting it would break Behavior 71 (hp.max preservation).
        if spawned == 0 {
            continue;
        }
        if let Ok((_, _, mut hp)) = parents.get_mut(parent_entity) {
            hp.current = hp.starting;
        }
    }
}
