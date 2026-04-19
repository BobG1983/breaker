//! Tether hazard — pairs of cells share damage.
//!
//! Design doc: `docs/todos/detail/mod-system-design/hazards/tether.md`.
//!
//! The hazard domain owns:
//! - [`TetherConfig`] — per-run tuning (stack-scaled damage + coverage percents).
//! - [`TetherLink`] — component marker on linked cells, pointing at the partner.
//! - [`TetherRedirectBuffer`] — per-tick buffer of redirect `DamageDealt<Cell>`
//!   pushed by `apply_damage_to_cells` and drained by [`emit_tether_redirects`].
//! - [`establish_tether_links`] — `OnEnter`(`Playing`) pair selection via shared
//!   `GameRng` with mutual-exclusion matching over adjacency pairs.
//! - [`cleanup_broken_tether_links`] — `FixedUpdate`, removes dangling links
//!   whose partner despawned or was `Dead`-marked.
//! - [`emit_tether_redirects`] — `FixedUpdate`, drains `TetherRedirectBuffer`
//!   into `MessageWriter<DamageDealt<Cell>>`.
//!
//! The redirect *computation* lives inside `cells::systems::apply_damage_to_cells`
//! so it can read the Diffusion-reduced primary damage returned by
//! `accumulate_message_deltas`. See design-doc §Edge Cases line 121 (Tether +
//! Diffusion ordering).

use std::collections::HashSet;

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::{
    cells::components::ADJACENCY_RADIUS_SQ,
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
    shared::death_pipeline::sets::DeathPipelineSystems,
};

/// Module-level cap on Tether link coverage — 100% means every eligible pair
/// may be linked (subject to mutual-exclusion matching).
pub(crate) const TETHER_COVERAGE_CAP_PERCENT: f32 = 100.0;

/// Sentinel `source_chip` value tagged on Tether redirect `DamageDealt<Cell>`
/// messages. The cells-domain `apply_damage_to_cells` skips re-emission when a
/// message carries this source, preventing infinite redirect loops.
pub(crate) const TETHER_SENTINEL: &str = "hazard:tether";

/// Per-run Tether tuning, in percentage units.
///
/// Translates from [`HazardTuning::Tether`]'s fractional authoring fields via
/// `* 100.0` at activation time. Stored in percent because the design-doc
/// formulas read in percent units.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct TetherConfig {
    /// Damage share percent delivered to the partner at stack 1 (e.g. 25.0).
    pub(crate) base_damage:        f32,
    /// Additional damage share percent per stack beyond the first.
    pub(crate) damage_per_level:   f32,
    /// Link coverage percent at stack 1 (e.g. 40.0 for 40% of eligible pairs).
    pub(crate) base_coverage:      f32,
    /// Additional coverage percent per stack beyond the first.
    pub(crate) coverage_per_level: f32,
}

impl TetherConfig {
    /// Damage-share percent for the given stack count. NOT capped (design doc
    /// §Stacking Behavior: stack 9 yields 105%, intentional for high-stack
    /// punishment).
    ///
    /// - `stacks == 0` → `0.0` (short-circuits before any arithmetic).
    /// - `stacks >= 1` → `base_damage + damage_per_level * (stacks - 1)`.
    #[must_use]
    pub(crate) const fn damage_percent(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        self.damage_per_level.mul_add(extra, self.base_damage)
    }

    /// Link-coverage percent for the given stack count. Capped at
    /// [`TETHER_COVERAGE_CAP_PERCENT`] (100%).
    ///
    /// - `stacks == 0` → `0.0` (short-circuits before cap logic).
    /// - `stacks >= 1` → `base_coverage + coverage_per_level * (stacks - 1)`,
    ///   clamped to the cap.
    #[must_use]
    pub(crate) const fn coverage_percent(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        let raw = self.coverage_per_level.mul_add(extra, self.base_coverage);
        if raw > TETHER_COVERAGE_CAP_PERCENT {
            TETHER_COVERAGE_CAP_PERCENT
        } else {
            raw
        }
    }
}

/// Marks a cell as part of a tether pair. Points to the partner cell entity.
///
/// Each cell in a linked pair has a `TetherLink` pointing to the other. When
/// one partner dies or despawns, the surviving cell's `TetherLink` is removed
/// by [`cleanup_broken_tether_links`].
#[derive(Component, Debug)]
pub(crate) struct TetherLink {
    /// The partner cell entity this cell is tethered to.
    pub(crate) partner: Entity,
}

/// Buffer of Tether redirect messages produced by `apply_damage_to_cells`
/// during the current tick. Drained by [`emit_tether_redirects`] immediately
/// after the damage-apply system in the same
/// [`DeathPipelineSystems::ApplyDamage`] set.
///
/// Lives in the hazard domain because Tether owns the buffering mechanism and
/// the emit system; the cells-domain `apply_damage_to_cells` reads+mutates
/// this resource via `Option<ResMut<TetherRedirectBuffer>>` under the
/// harness-safe `Option`-wrapped-resource pattern.
#[derive(Resource, Default, Debug)]
pub(crate) struct TetherRedirectBuffer(pub(crate) Vec<DamageDealt<Cell>>);

/// Inserts [`TetherConfig`] from [`HazardTuning::Tether`], translating the
/// fractional authoring fields to percentage units (× 100). Called each time
/// the player picks Tether; last write wins (overwrites any prior config).
/// Warns and no-ops on a non-Tether tuning variant, leaving any existing
/// `TetherConfig` intact.
pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Tether {
        base_share_frac,
        per_level_share_frac,
        base_coverage_frac,
        per_level_coverage_frac,
    } = *tuning
    else {
        warn!("tether::activate called with non-Tether tuning");
        return;
    };
    commands.insert_resource(TetherConfig {
        base_damage:        base_share_frac * 100.0,
        damage_per_level:   per_level_share_frac * 100.0,
        base_coverage:      base_coverage_frac * 100.0,
        coverage_per_level: per_level_coverage_frac * 100.0,
    });
}

/// Registers Tether's runtime systems and the redirect buffer resource.
///
/// - `establish_tether_links` → `OnEnter(NodeState::Playing)`,
///   `hazard_active(Tether)`.
/// - `cleanup_broken_tether_links` → `FixedUpdate`, after `HandleKill`,
///   `hazard_active(Tether)` AND `in_state(Playing)`.
/// - `emit_tether_redirects` → `FixedUpdate`, in `ApplyDamage`, after
///   `apply_damage_to_cells`, `hazard_active(Tether)`.
pub(crate) fn register(app: &mut App) {
    app.init_resource::<TetherRedirectBuffer>();

    app.add_systems(
        OnEnter(NodeState::Playing),
        establish_tether_links.run_if(hazard_active(HazardKind::Tether)),
    );
    app.add_systems(
        FixedUpdate,
        cleanup_broken_tether_links
            .after(DeathPipelineSystems::HandleKill)
            .run_if(hazard_active(HazardKind::Tether))
            .run_if(in_state(NodeState::Playing)),
    );
    app.add_systems(
        FixedUpdate,
        emit_tether_redirects
            .in_set(DeathPipelineSystems::ApplyDamage)
            .after(crate::cells::systems::apply_damage_to_cells)
            .run_if(hazard_active(HazardKind::Tether)),
    );
}

type LiveCellPositions<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Position2D),
    (With<Cell>, Without<Dead>, Without<Invulnerable>),
>;

/// Selects pairs of cells to link via mutual-exclusion matching over shuffled
/// adjacency pairs. Runs once on `OnEnter(NodeState::Playing)` behind the
/// `hazard_active(Tether)` run-if.
///
/// Determinism comes from the shared [`GameRng`]: two identical runs with the
/// same seed produce identical link sets. Harness-safe — early-returns if
/// [`TetherConfig`] or `GameRng` is absent.
pub(crate) fn establish_tether_links(
    active: Res<ActiveHazards>,
    config: Option<Res<TetherConfig>>,
    cells: LiveCellPositions,
    rng: Option<ResMut<GameRng>>,
    mut commands: Commands,
) {
    let Some(config) = config else { return };
    let Some(mut rng) = rng else { return };

    let stacks = active.stacks(HazardKind::Tether);
    if stacks == 0 {
        return;
    }

    // Collect eligible cells — filters already applied by the query.
    let cell_vec: Vec<(Entity, Vec2)> = cells.iter().map(|(e, pos)| (e, pos.0)).collect();

    // Compute all unordered eligible adjacent pairs.
    let mut pairs: Vec<(Entity, Entity)> = Vec::new();
    for (i, (e_a, pos_a)) in cell_vec.iter().enumerate() {
        for (e_b, pos_b) in &cell_vec[i + 1..] {
            if pos_a.distance_squared(*pos_b) <= ADJACENCY_RADIUS_SQ {
                pairs.push((*e_a, *e_b));
            }
        }
    }

    let total = pairs.len();
    if total == 0 {
        return;
    }

    let coverage_pct = config.coverage_percent(stacks);
    let raw = (coverage_pct / 100.0) * total as f32;
    let target_count = (raw.round() as usize).min(total);

    // Shuffle via the shared GameRng (deterministic under a fixed seed).
    pairs.shuffle(&mut rng.0);

    // Mutual-exclusion matching: walk shuffled pairs left-to-right; each cell
    // participates in AT MOST one pair. Skip pairs whose endpoint is already
    // claimed (skips do not advance `inserted`).
    let mut claimed: HashSet<Entity> = HashSet::new();
    let mut inserted: usize = 0;
    for (a, b) in &pairs {
        if inserted >= target_count {
            break;
        }
        if claimed.contains(a) || claimed.contains(b) {
            continue;
        }
        commands.entity(*a).insert(TetherLink { partner: *b });
        commands.entity(*b).insert(TetherLink { partner: *a });
        claimed.insert(*a);
        claimed.insert(*b);
        inserted += 1;
    }
}

/// Removes `TetherLink` from any surviving cell whose partner has been
/// despawned or marked `Dead`. Runs in `FixedUpdate` after
/// [`DeathPipelineSystems::HandleKill`] so `Dead` markers are visible.
pub(crate) fn cleanup_broken_tether_links(
    links: Query<(Entity, &TetherLink)>,
    partners: Query<(), (With<Cell>, Without<Dead>)>,
    mut commands: Commands,
) {
    for (entity, link) in &links {
        if partners.get(link.partner).is_err() {
            commands.entity(entity).remove::<TetherLink>();
        }
    }
}

/// Drains [`TetherRedirectBuffer`] into `MessageWriter<DamageDealt<Cell>>`.
///
/// Holds ONLY the writer for `DamageDealt<Cell>` (no reader) to avoid the
/// Bevy 0.18 scheduler panic when a single system holds both
/// `MessageReader<T>` and `MessageWriter<T>` for the same `T`. The paired
/// `apply_damage_to_cells` (cells domain) computes redirects and pushes them
/// onto the buffer; this system runs immediately after it in the same
/// [`DeathPipelineSystems::ApplyDamage`] set.
pub(crate) fn emit_tether_redirects(
    mut buffer: ResMut<TetherRedirectBuffer>,
    mut writer: MessageWriter<DamageDealt<Cell>>,
) {
    for msg in buffer.0.drain(..) {
        writer.write(msg);
    }
}
