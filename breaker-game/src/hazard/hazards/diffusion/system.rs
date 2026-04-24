//! Diffusion hazard — multi-frame damage-share redistribution.
//!
//! Design doc: `docs/design/hazards/diffusion.md`.
//!
//! Pipeline split:
//! - `diffusion_reduce_primary` in `DmgSystems::MutateDamage` reduces the
//!   primary `DamageDealt<Cell>.amount` by `share_frac` and queues a
//!   `PendingEmission` describing the ring to emit. Seeds / reuses
//!   instance ids via `DiffusionInstances`.
//! - `diffusion_emit_rings` in `DmgSystems::PostApplyDamage` drains
//!   `PendingDiffusionEmissions.queue`, splits each pending share across
//!   candidate neighbors, applies the attenuation floor
//!   (per-neighbor `>= 1.0`), and emits ring `DamageDealt<Cell>` messages
//!   with source `"hazard:diffusion:{instance_id}"`.

use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    cells::components::{ADJACENCY_RADIUS_SQ, Cell},
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
};

/// Module-level cap on diffusion share — prevents the "surrounded cell takes 0
/// damage" degenerate case at high stacks. Percentage units. Lives at module
/// scope (not `impl DiffusionConfig`) so callers (including tests) can import
/// it directly without going through the resource type.
pub(crate) const DIFFUSION_SHARE_CAP_PERCENT: f32 = 95.0;

/// Source-id prefix for diffusion-derived ring messages. Rings are tagged
/// `"hazard:diffusion:{instance_id}"` so `diffusion_reduce_primary` can
/// identify reentering messages and reuse their `instance_id`.
pub(crate) const DIFFUSION_SOURCE_PREFIX: &str = "hazard:diffusion";

/// Prefix with trailing `:` — matched against `DamageDealt<Cell>.source`
/// to detect a ring re-entry (`"hazard:diffusion:{instance_id}"`). Avoids
/// per-tick `format!` allocation.
const DIFFUSION_SOURCE_PREFIX_COLON: &str = "hazard:diffusion:";

/// Per-run Diffusion tuning, in percentage units.
///
/// Translates from [`HazardTuning::Diffusion`]'s fractional authoring fields
/// via a `* 100.0` conversion at activation time; this resource stores percents
/// because the design-doc formula uses percent-unit math.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct DiffusionConfig {
    /// Share percent added by the hazard at stack 1 (e.g. `20.0` for 20%).
    pub(crate) base_share_percent:      f32,
    /// Additional share percent per stack beyond the first.
    pub(crate) share_per_level_percent: f32,
    /// Stacks required to escalate the BFS depth by one. Must be > 0 in config;
    /// pathological `0` is clamped to `1` inside [`DiffusionConfig::depth`].
    pub(crate) depth_increase_interval: u32,
}

impl DiffusionConfig {
    /// Share percent for the given stack count.
    ///
    /// - `stacks == 0` → `0.0` (hazard inactive short-circuit).
    /// - `stacks >= 1` → `base_share_percent + share_per_level_percent *
    ///   (stacks - 1)`, clamped to [`DIFFUSION_SHARE_CAP_PERCENT`].
    ///
    /// Not `const fn` — `f32::min` is not `const` on Bevy 0.18's MSRV.
    #[must_use]
    pub(crate) fn share_percent(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        let raw = self
            .share_per_level_percent
            .mul_add(extra, self.base_share_percent);
        if raw > DIFFUSION_SHARE_CAP_PERCENT {
            DIFFUSION_SHARE_CAP_PERCENT
        } else {
            raw
        }
    }

    /// BFS cascade depth for the given stack count.
    ///
    /// - `stacks == 0` → `0` (hazard inactive — short-circuits the BFS).
    /// - `stacks >= 1` → `1 + (stacks - 1) / depth_increase_interval.max(1)`
    ///   so a pathological `depth_increase_interval == 0` pins depth at 1
    ///   rather than dividing by zero.
    ///
    /// `const fn` — integer arithmetic only (`saturating_sub`, `max`, integer
    /// division).
    #[must_use]
    pub(crate) const fn depth(self, stacks: u32) -> u32 {
        if stacks == 0 {
            return 0;
        }
        if self.depth_increase_interval == 0 {
            // Pathological config: pin depth at 1 regardless of stack.
            return 1;
        }
        1 + stacks.saturating_sub(1) / self.depth_increase_interval
    }
}

/// Per-run Diffusion instance tracker. A fresh `instance_id` is allocated for
/// every external (non-diffusion-sourced) `DamageDealt<Cell>` message entering
/// `diffusion_reduce_primary`. All ring-emissions derived from that primary
/// share the `instance_id` so subsequent hops dedupe via the visited set.
///
/// Reset on `OnExit(NodeState::Playing)` via `reset_diffusion_state`.
#[derive(Resource, Default, Debug)]
pub(crate) struct DiffusionInstances {
    /// Next unused instance id.
    pub(crate) next_id: u64,
    /// Per-instance visited cell set.
    pub(crate) visited: HashMap<u64, HashSet<Entity>>,
}

/// Handoff queue between `diffusion_reduce_primary` (`MutateDamage`) and
/// `diffusion_emit_rings` (`PostApplyDamage`). Drained and re-filled each tick.
#[derive(Resource, Default, Debug)]
pub(crate) struct PendingDiffusionEmissions {
    /// In-flight pending emissions.
    pub(crate) queue: Vec<PendingEmission>,
}

/// A single pending ring emission queued by `diffusion_reduce_primary`.
///
/// `shared` is the pre-computed absolute damage amount (not a fraction) to be
/// split across `candidate_neighbors` in `PostApplyDamage`. `diffusion_emit_rings`
/// does NOT re-read the primary message from `Messages<DamageDealt<Cell>>` —
/// same-tick multi-hit correlation is captured in this struct directly.
#[derive(Debug, Clone)]
pub(crate) struct PendingEmission {
    /// The primary target that just absorbed damage. Used by
    /// `diffusion_emit_rings` solely to query `Invulnerable` on the target
    /// (ripple-source invulnerability skip).
    pub(crate) target:              Entity,
    /// Instance id from `DiffusionInstances`.
    pub(crate) instance_id:         u64,
    /// Total damage amount to split across `candidate_neighbors`. Equals
    /// `primary_amount * share_frac` computed at reduce-time.
    pub(crate) shared:              f32,
    /// Non-visited, non-dead, non-invulnerable in-range cells.
    pub(crate) candidate_neighbors: Vec<Entity>,
    /// Forwarded `attributed_to` from the primary message — equals
    /// `msg.attributed_to.or(msg.dealer)` at reduce time. Ring emissions set
    /// their `attributed_to` to this value so kill attribution travels.
    pub(crate) attributed_to:       Option<Entity>,
}

/// Inserts [`DiffusionConfig`] from [`HazardTuning::Diffusion`], translating
/// the fractional authoring fields to percentage units (× 100). Called each
/// time the player picks Diffusion from `hazards::activate`; last write wins
/// (overwrites any prior [`DiffusionConfig`]; stack count is owned by
/// `ActiveHazards`, not the config). Warns and no-ops on a non-Diffusion
/// tuning variant, leaving any existing [`DiffusionConfig`] intact.
pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Diffusion {
        base_share_frac,
        per_level_share_frac,
        depth_every_levels,
    } = *tuning
    else {
        warn!("diffusion::activate called with non-Diffusion tuning");
        return;
    };
    commands.insert_resource(DiffusionConfig {
        base_share_percent:      base_share_frac * 100.0,
        share_per_level_percent: per_level_share_frac * 100.0,
        depth_increase_interval: depth_every_levels,
    });
}

/// Wires the two Diffusion pipeline systems, the instance-tracker resources,
/// and the node-exit cleanup.
pub(crate) fn register(app: &mut App) {
    app.init_resource::<DiffusionInstances>();
    app.init_resource::<PendingDiffusionEmissions>();

    // Deliberate late emitter: reads the `PendingDiffusionEmissions` queue
    // (populated in `DmgSystems::MutateDamage`) to cascade follow-up damage.
    // MUST stay in DmgSystems::PostApplyDamage so it runs after the primary
    // damage emitters in DmgSystems::EmitDamage and the applicators in
    // DmgSystems::ApplyDamage.
    app.add_systems(
        FixedUpdate,
        (
            diffusion_reduce_primary
                .in_set(DmgSystems::MutateDamage)
                .run_if(in_state(NodeState::Playing))
                .run_if(hazard_active(HazardKind::Diffusion)),
            diffusion_emit_rings
                .in_set(DmgSystems::PostApplyDamage)
                .in_set(crate::game::PostApplyRipple::Diffusion)
                .run_if(in_state(NodeState::Playing))
                .run_if(hazard_active(HazardKind::Diffusion)),
        ),
    );

    app.add_systems(OnExit(NodeState::Playing), reset_diffusion_state);
}

/// `OnExit(NodeState::Playing)` — resets `DiffusionInstances` and
/// `PendingDiffusionEmissions` to `Default::default()` so state doesn't leak
/// across nodes.
pub(crate) fn reset_diffusion_state(
    mut instances: ResMut<DiffusionInstances>,
    mut pending: ResMut<PendingDiffusionEmissions>,
) {
    *instances = DiffusionInstances::default();
    *pending = PendingDiffusionEmissions::default();
}

type DiffusionAdjacencyQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Position2D),
    (With<Cell>, Without<Dead>, Without<Invulnerable>),
>;

/// `DmgSystems::MutateDamage` pass. For each `DamageDealt<Cell>` message:
///
/// 1. Resolve `instance_id`:
///    - `source` starts with `"hazard:diffusion:"` → parse suffix as `u64`
///      and reuse that instance's visited set (or resurrect orphan).
///    - Else → allocate a fresh id from `instances.next_id`, seed visited
///      with `msg.target`.
/// 2. Compute `candidate_neighbors`: live cells in the adjacency snapshot
///    excluding visited, dead, invulnerable, within `ADJACENCY_RADIUS_SQ`
///    of `msg.target`.
/// 3. If empty → pass-through (no reduction, no pending emission).
/// 4. Else:
///    - `shared = msg.amount * share_frac`.
///    - `msg.amount *= 1 - share_frac`.
///    - Push `PendingEmission` with `attributed_to = msg.attributed_to.or(msg.dealer)`.
pub(crate) fn diffusion_reduce_primary(
    config: Option<Res<DiffusionConfig>>,
    active: Option<Res<ActiveHazards>>,
    mut mutator: MessageMutator<DamageDealt<Cell>>,
    mut instances: ResMut<DiffusionInstances>,
    mut pending: ResMut<PendingDiffusionEmissions>,
    cells: DiffusionAdjacencyQuery,
) {
    let Some(config) = config else { return };
    let Some(active) = active else { return };
    let stacks = active.stacks(HazardKind::Diffusion);
    if stacks == 0 {
        return;
    }
    let share_frac = config.share_percent(stacks) / 100.0;
    if share_frac <= 0.0 {
        return;
    }

    // Snapshot the adjacency map once per tick.
    let snapshot: Vec<(Entity, Vec2)> = cells.iter().map(|(e, p)| (e, p.0)).collect();

    for msg in mutator.read() {
        let instance_id_opt = msg
            .source
            .as_ref()
            .and_then(|s| s.0.as_ref().strip_prefix(DIFFUSION_SOURCE_PREFIX_COLON))
            .and_then(|suffix| suffix.parse::<u64>().ok());

        let instance_id = if let Some(id) = instance_id_opt {
            // Reuse / resurrect orphan instance.
            instances
                .visited
                .entry(id)
                .or_insert_with(|| HashSet::from([msg.target]));
            // Bump next_id so future fresh allocations don't collide.
            if id >= instances.next_id {
                instances.next_id = id + 1;
            }
            id
        } else {
            // Allocate fresh instance.
            let id = instances.next_id;
            instances.next_id += 1;
            instances.visited.insert(id, HashSet::from([msg.target]));
            id
        };

        // Find target position.
        let target_pos_opt = snapshot
            .iter()
            .find(|(e, _)| *e == msg.target)
            .map(|(_, p)| *p);

        let mut candidate_neighbors: Vec<Entity> = Vec::new();
        if let Some(target_pos) = target_pos_opt {
            let visited = instances.visited.get(&instance_id);
            for (e, pos) in &snapshot {
                if *e == msg.target {
                    continue;
                }
                if visited.is_some_and(|v| v.contains(e)) {
                    continue;
                }
                if pos.distance_squared(target_pos) <= ADJACENCY_RADIUS_SQ {
                    candidate_neighbors.push(*e);
                }
            }
        }

        if candidate_neighbors.is_empty() {
            // Pass-through — no reduction, no pending.
            continue;
        }

        let shared = msg.amount * share_frac;
        let attributed_to = msg.attributed_to.or(msg.dealer);
        let target = msg.target;
        msg.amount *= 1.0 - share_frac;

        pending.queue.push(PendingEmission {
            target,
            instance_id,
            shared,
            candidate_neighbors,
            attributed_to,
        });
    }
}

/// `DmgSystems::PostApplyDamage` pass. Drains `PendingDiffusionEmissions.queue`.
/// For each pending emission:
///
/// 1. If `pending.target` currently has `Invulnerable` → skip entirely
///    (unified ripple-source invulnerability rule).
/// 2. Else compute `per_neighbor = pending.shared / pending.candidate_neighbors.len()`.
/// 3. If `per_neighbor < 1.0` → skip (attenuation floor, inclusive at 1.0).
/// 4. Emit one `DamageDealt<Cell>` per neighbor with `source =
///    "hazard:diffusion:{instance_id}"`, `dealer = None`,
///    `attributed_to = pending.attributed_to`.
/// 5. Insert every emitted neighbor entity into
///    `instances.visited[pending.instance_id]`.
pub(crate) fn diffusion_emit_rings(
    mut messages: ResMut<Messages<DamageDealt<Cell>>>,
    mut pending: ResMut<PendingDiffusionEmissions>,
    mut instances: ResMut<DiffusionInstances>,
    invulnerable: Query<(), With<Invulnerable>>,
) {
    let drained: Vec<PendingEmission> = std::mem::take(&mut pending.queue);
    for p in drained {
        if invulnerable.get(p.target).is_ok() {
            continue;
        }
        let n = p.candidate_neighbors.len();
        if n == 0 {
            continue;
        }
        let per_neighbor = p.shared / n as f32;
        if per_neighbor < 1.0 {
            continue;
        }
        // Build the `SourceId` once per pending emission and clone the
        // (cheap `Cow<'static, str>`-backed) `SourceId` per neighbor
        // instead of allocating a new `String` every iteration.
        let source = SourceId::from(format!("{DIFFUSION_SOURCE_PREFIX}:{}", p.instance_id));
        for &neighbor in &p.candidate_neighbors {
            messages.write(DamageDealt::<Cell> {
                dealer:        None,
                attributed_to: p.attributed_to,
                target:        neighbor,
                amount:        per_neighbor,
                source:        Some(source.clone()),
                _marker:       PhantomData,
            });
        }
        let visited = instances.visited.entry(p.instance_id).or_default();
        for &neighbor in &p.candidate_neighbors {
            visited.insert(neighbor);
        }
    }
}
