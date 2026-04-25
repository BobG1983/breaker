//! Sympathy hazard — damage to a cell heals its neighbours.
//!
//! Design doc: `docs/design/hazards/sympathy.md`.
//!
//! Reads [`DamageDealt<Cell>`]. For every non-zero damage event, emits one
//! [`HealDealt<Cell>`] per live neighbour reached by a BFS outward from the
//! damaged cell, up to the stack-scaled cascade depth. Ring attenuation is
//! geometric: ring-N heal amount is `previous_amount * heal_percent / 100.0`.
//!
//! The damaged cell itself is seeded into the `visited` set so it never
//! self-heals. `Dead` and `Invulnerable` cells are excluded from the
//! adjacency snapshot and therefore cannot be healed or used as BFS
//! intermediaries.

use std::{collections::HashSet, marker::PhantomData};

use bevy::prelude::*;

use crate::{
    cells::components::ADJACENCY_RADIUS_SQ,
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::ActiveHazards,
    },
    prelude::*,
};

// ── SympathyConfig ──────────────────────────────────────────────────────────

/// Per-run Sympathy tuning extracted from [`HazardTuning::Sympathy`] at
/// activation time. Fractional authoring fields (`*_frac`) are translated to
/// percent units (`* 100.0`) on insert; `depth_every_levels` passes through
/// unchanged as `depth_increase_interval`.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct SympathyConfig {
    /// Heal percent per neighbour at stack 1 (e.g. 25.0 = 25%).
    pub(crate) base_heal_percent:       f32,
    /// Additional heal percent per neighbour per stack beyond the first.
    pub(crate) heal_per_level_percent:  f32,
    /// Every `depth_increase_interval` stacks, cascade depth increases by 1.
    pub(crate) depth_increase_interval: u32,
}

impl SympathyConfig {
    /// Heal percent per neighbour for the given stack count.
    ///
    /// - `stacks == 0` → `0.0` (short-circuits before any arithmetic).
    /// - `stacks >= 1` → `base_heal_percent + heal_per_level_percent * (stacks - 1)`.
    ///   Not capped — high-stack overheal is intentional.
    #[must_use]
    pub(crate) const fn heal_percent(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        self.heal_per_level_percent
            .mul_add(extra, self.base_heal_percent)
    }

    /// BFS cascade depth (number of rings) for the given stack count.
    ///
    /// - `stacks == 0` → `0` (hazard inactive short-circuit).
    /// - `depth_increase_interval == 0` → `1` for any `stacks >= 1`
    ///   (divide-by-zero clamp guard; degenerate tuning never produced by
    ///   valid RON).
    /// - Otherwise → `1 + (stacks - 1) / depth_increase_interval`.
    #[must_use]
    pub(crate) const fn cascade_depth(self, stacks: u32) -> u32 {
        if stacks == 0 {
            return 0;
        }
        if self.depth_increase_interval == 0 {
            return 1;
        }
        let extra = stacks.saturating_sub(1);
        1 + extra / self.depth_increase_interval
    }
}

// ── activate ────────────────────────────────────────────────────────────────

/// Inserts [`SympathyConfig`] from [`HazardTuning::Sympathy`]. Called each
/// time the player picks Sympathy; last write wins (overwrites any prior
/// config). Warns and no-ops on a non-Sympathy tuning variant, leaving any
/// existing `SympathyConfig` intact. Fractional fields are multiplied by
/// `100.0` to produce percent-unit config fields.
pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Sympathy {
        base_heal_frac,
        per_level_heal_frac,
        depth_every_levels,
    } = *tuning
    else {
        warn!("sympathy::activate called with non-Sympathy tuning");
        return;
    };
    commands.insert_resource(SympathyConfig {
        base_heal_percent:       base_heal_frac * 100.0,
        heal_per_level_percent:  per_level_heal_frac * 100.0,
        depth_increase_interval: depth_every_levels,
    });
}

// ── register ────────────────────────────────────────────────────────────────

/// Registers Sympathy's runtime system.
///
/// `sympathy_heal_adjacent` is a reader system — it holds
/// `MessageReader<DamageDealt<Cell>>`. Runs in `FixedUpdate`, in
/// [`DmgSystems::PostApplyDamage`] — AFTER `ApplyDamage` (so `msg.amount`
/// reflects the final applied value, including vuln/invulnerable-filter
/// mutation), but BEFORE `EmitKill`/`ApplyKill` (so the damaged primary
/// cell is still in the live snapshot to anchor the BFS). Heals emitted
/// here reach `apply_heal::<Cell>` later in the same tick when the chain
/// flows through `EmitHeal → ApplyHeal`.
///
/// Registered WITHOUT `.run_if(...)`; it enforces the `ActiveHazards` /
/// `NodeState::Playing` gate in-body via an immediate `reader.clear()` +
/// return when inactive, draining the buffer every tick. A `.run_if(...)`
/// gate suppresses execution but does NOT advance the reader cursor, so
/// messages buffered during gated-off frames would get retroactively
/// consumed the tick the gate opens.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        sympathy_heal_adjacent.in_set(DmgSystems::PostApplyDamage),
    );
}

// ── sympathy_heal_adjacent ──────────────────────────────────────────────────

/// Snapshot source for the BFS: every live cell's entity + position,
/// excluding `Dead` (the damage pipeline has marked for despawn) and
/// `Invulnerable` (consistent with other hazards: invulnerable cells
/// neither receive nor propagate Sympathy heals).
type LiveCellPositions<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Position2D),
    (With<Cell>, Without<Dead>, Without<Invulnerable>),
>;

/// Reads every `DamageDealt<Cell>` this tick and, for each message whose
/// `amount > 0.0` targeting a live cell, emits one `HealDealt<Cell>` per
/// live neighbour reached by BFS outward from the damaged cell up to
/// `config.cascade_depth(stacks)` rings. Ring-N heal amount is
/// `msg.amount * (heal_percent / 100.0)^N`.
///
/// Every emitted heal carries `cap: HealCap::Starting`,
/// `source: Some(SourceId::hazard(HazardKind::Sympathy).build())`, and `healer: None`.
///
/// Gated in-body: this system runs every `FixedUpdate` tick. When
/// Sympathy is not active or `NodeState` is not `Playing`, it drains the
/// `MessageReader` via `reader.clear()` and returns so buffered
/// `DamageDealt<Cell>` messages cannot leak retroactively when the hazard
/// activates on a later frame.
///
/// Harness-safe early returns:
/// - No `SympathyConfig` resource → `reader.clear()`; return.
/// - `heal_percent(stacks) <= 0.0` or `depth == 0` → `reader.clear()`; return.
/// - Empty live-cell snapshot → `reader.clear()`; return.
///
/// The damaged cell is seeded into the BFS `visited` set so it never
/// receives a heal from its own damage message. `Dead` and `Invulnerable`
/// cells are excluded from the snapshot by the query filter and therefore
/// can neither be healed nor act as BFS intermediaries.
pub(crate) fn sympathy_heal_adjacent(
    mut reader: MessageReader<DamageDealt<Cell>>,
    active_hazards: Option<Res<ActiveHazards>>,
    node_state: Option<Res<State<NodeState>>>,
    config: Option<Res<SympathyConfig>>,
    cells: LiveCellPositions,
    mut writer: MessageWriter<HealDealt<Cell>>,
) {
    if active_hazards
        .as_ref()
        .is_none_or(|ah| !ah.is_active(HazardKind::Sympathy))
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

    let stacks = active.stacks(HazardKind::Sympathy);
    let heal_percent = config.heal_percent(stacks);
    let depth = config.cascade_depth(stacks);
    if heal_percent <= 0.0 || depth == 0 {
        reader.clear();
        return;
    }

    // Peek the reader: if there are no damage messages, avoid the full
    // live-cell snapshot allocation on quiet ticks.
    if reader.is_empty() {
        return;
    }

    let snapshot: Vec<(Entity, Vec2)> = cells.iter().map(|(e, pos)| (e, pos.0)).collect();
    if snapshot.is_empty() {
        reader.clear();
        return;
    }

    // Convert authored percent (e.g. 25.0 for "25%") into a dimensionless
    // multiplier (0.25) used both for ring 1 and geometric attenuation.
    let factor = heal_percent / 100.0;
    let source = SourceId::hazard(HazardKind::Sympathy).build();

    for msg in reader.read() {
        if msg.amount <= 0.0 {
            continue;
        }

        // Primary cell must be in the live snapshot; otherwise no ring
        // anchor exists. Covers despawned / Dead / Invulnerable primaries.
        let Some(&(_, primary_pos)) = snapshot.iter().find(|(e, _)| *e == msg.target) else {
            continue;
        };

        let mut visited: HashSet<Entity> = HashSet::new();
        visited.insert(msg.target);

        let mut frontier: Vec<(Entity, Vec2)> = vec![(msg.target, primary_pos)];
        let mut ring_amount = msg.amount * factor;

        for _ring in 1..=depth {
            let mut next: Vec<(Entity, Vec2)> = Vec::new();
            for (entity, pos) in &snapshot {
                if visited.contains(entity) {
                    continue;
                }
                let adjacent = frontier
                    .iter()
                    .any(|(_, fpos)| pos.distance_squared(*fpos) <= ADJACENCY_RADIUS_SQ);
                if !adjacent {
                    continue;
                }
                next.push((*entity, *pos));
            }
            if next.is_empty() {
                break;
            }
            for (entity, _) in &next {
                visited.insert(*entity);
                writer.write(HealDealt::<Cell> {
                    healer:        None,
                    attributed_to: None,
                    target:        *entity,
                    amount:        ring_amount,
                    cap:           HealCap::Starting,
                    source:        Some(source.clone()),
                    _marker:       PhantomData,
                });
            }
            frontier = next;
            ring_amount *= factor;
        }
    }
}
