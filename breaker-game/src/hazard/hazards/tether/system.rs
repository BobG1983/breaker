//! Tether hazard — pairs of cells share damage.
//!
//! Design doc: `docs/design/hazards/tether.md`.
//!
//! The hazard domain owns:
//! - [`TetherConfig`] — per-run tuning (stack-scaled damage + coverage percents).
//! - [`TetherLink`] — component marker on linked cells, pointing at the partner.
//! - [`establish_tether_links`] — `OnEnter`(`Playing`) pair selection via shared
//!   `GameRng` with mutual-exclusion matching over adjacency pairs.
//! - [`cleanup_broken_tether_links`] — `FixedUpdate`, removes dangling links
//!   whose partner despawned or was `Dead`-marked.
//! - [`tether_emit_partner`] — `FixedUpdate` in `DmgSystems::PostApplyDamage`, reads
//!   post-apply `DamageDealt<Cell>` messages and emits a partner sibling
//!   message when the primary target has a `TetherLink` and the primary
//!   amount is positive.

use std::{collections::HashSet, marker::PhantomData};

use bevy::{ecs::message::MessageCursor, prelude::*};
use rand::seq::SliceRandom;

use crate::{
    cells::components::ADJACENCY_RADIUS_SQ,
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
};

/// Module-level cap on Tether link coverage — 100% means every eligible pair
/// may be linked (subject to mutual-exclusion matching).
pub(crate) const TETHER_COVERAGE_CAP_PERCENT: f32 = 100.0;

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
    ///
    /// `const fn` is safe here — `f32::mul_add` is `const` on Bevy 0.18's
    /// MSRV, unlike `f32::min` (which is why `diffusion::share_percent` is
    /// not `const`).
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
    ///
    /// `const fn` is safe here — both `f32::mul_add` and direct `>` comparison
    /// are `const` on Bevy 0.18's MSRV. Note that `f32::min` is not, which is
    /// why the cap is expressed as an `if`/`else` rather than `raw.min(cap)`.
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

/// Registers Tether's runtime systems.
///
/// - `establish_tether_links` → `OnEnter(NodeState::Playing)`,
///   `hazard_active(Tether)`.
/// - `cleanup_broken_tether_links` → `FixedUpdate`, after `ApplyKill`,
///   `hazard_active(Tether)` AND `in_state(Playing)`.
/// - `tether_emit_partner` → `FixedUpdate`, in `DmgSystems::PostApplyDamage`,
///   `hazard_active(Tether)` AND `in_state(Playing)`.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        OnEnter(NodeState::Playing),
        establish_tether_links.run_if(hazard_active(HazardKind::Tether)),
    );
    app.add_systems(
        FixedUpdate,
        cleanup_broken_tether_links
            .after(DmgSystems::ApplyKill)
            .run_if(hazard_active(HazardKind::Tether))
            .run_if(in_state(NodeState::Playing)),
    );
    // Deliberate late emitter: reads DamageDealt<Cell> / Dead state from the
    // current tick to cascade follow-up damage. MUST stay in
    // DmgSystems::PostApplyDamage so it runs after the primary damage emitters
    // in DmgSystems::EmitDamage and the applicators in DmgSystems::ApplyDamage.
    app.add_systems(
        FixedUpdate,
        tether_emit_partner
            .in_set(DmgSystems::PostApplyDamage)
            .in_set(crate::game::PostApplyRipple::Tether)
            .run_if(hazard_active(HazardKind::Tether))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// Query alias for alive cells with their positions, used by
/// `establish_tether_links` for pair selection. Invulnerability is
/// intentionally NOT filtered here: invulnerable cells are eligible
/// tether partners, and a tether pair where one or both endpoints
/// are invulnerable is INERT — the `tether_emit_partner` `amount <= 0.0`
/// guard (itself upheld by `invulnerable_filter::<Cell>` in
/// `DmgSystems::ApplyDamage`) stops ripple emission when the primary
/// is invulnerable, and the invulnerable partner's sibling message is
/// zeroed by the same filter before `apply_damage::<Cell>` runs.
///
/// Game-feel trade-off (user-approved): inert tether pairs may appear
/// on the board. Aligning with the pipeline-owns-filtering principle
/// is preferred over the prior "invulnerable cells can never be
/// partners" rule.
type LiveCellPositions<'w, 's> =
    Query<'w, 's, (Entity, &'static Position2D), (With<Cell>, Without<Dead>)>;

/// Selects pairs of cells to link via mutual-exclusion matching over shuffled
/// adjacency pairs. Runs once on `OnEnter(NodeState::Playing)` behind the
/// `hazard_active(Tether)` run-if.
///
/// Invulnerability is NOT a pair-selection filter — invulnerable cells are
/// eligible partners. A pair whose endpoint is invulnerable is inert:
/// `tether_emit_partner` skips ripple emission when the primary's post-apply
/// `amount` is `<= 0.0` (which the `invulnerable_filter::<Cell>` mutator
/// guarantees for invulnerable primaries), and any partner sibling emitted
/// against an invulnerable partner is zeroed by the same filter before
/// `apply_damage::<Cell>` runs. Inert pairs are an accepted game-feel
/// trade-off for uniform pipeline-owned filtering.
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

    // Collect eligible cells — `With<Cell>, Without<Dead>` applied by the query.
    // Invulnerable cells are eligible partners by design; see the
    // `LiveCellPositions` doc comment for the inert-pair trade-off.
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
/// [`DmgSystems::ApplyKill`] so `Dead` markers are visible.
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

/// `DmgSystems::PostApplyDamage` — emits a partner sibling `DamageDealt<Cell>` for
/// every post-apply message on a tethered primary cell.
///
/// Rules:
/// - Skip messages whose `source` already carries the `hazard:tether`
///   source (loop protection).
/// - Skip messages with `amount <= 0.0` — the invulnerable-filter has
///   zeroed the primary; enforces the uniform "invulnerable source → no
///   ripple" rule alongside diffusion and echo strike.
/// - Skip messages whose target has no `TetherLink`.
/// - Emit a new `DamageDealt<Cell>` to the partner with
///   `amount = msg.amount * damage_pct / 100.0`, `dealer = None`, and
///   `attributed_to = msg.attributed_to.or(msg.dealer)` so kill
///   attribution travels. The partner sibling traverses the full damage
///   pipeline on the next `FixedUpdate` tick (1-frame delay).
///
/// Does NOT gate on "is `link.partner` live" — the downstream
/// `apply_damage::<Cell>` filters `Without<Dead>`, and Bevy's entity
/// queries skip despawned entities gracefully. Adding a guard here would
/// duplicate work for zero benefit.
pub(crate) fn tether_emit_partner(
    config: Option<Res<TetherConfig>>,
    active: Option<Res<ActiveHazards>>,
    mut cursor: Local<MessageCursor<DamageDealt<Cell>>>,
    mut messages: ResMut<Messages<DamageDealt<Cell>>>,
    tethered: Query<&TetherLink, (With<Cell>, Without<Dead>)>,
) {
    let Some(config) = config else { return };
    let Some(active) = active else { return };
    let stacks = active.stacks(HazardKind::Tether);
    if stacks == 0 {
        return;
    }
    let damage_pct = config.damage_percent(stacks);
    if damage_pct <= 0.0 {
        return;
    }

    let tether_source = SourceId::hazard(HazardKind::Tether).build();

    // Snapshot every unread `DamageDealt<Cell>` message via a local cursor.
    // Using `MessageCursor` gives us a standard `MessageReader`-style traversal
    // that spans both internal message buffers (current + previous) without
    // consuming the messages — downstream pipeline stages next frame still
    // see the primaries. The cursor tracks position in `Local`, so each
    // tick advances it past already-seen messages (no duplicate emits).
    let snapshot: Vec<DamageDealt<Cell>> = cursor.read(&*messages).cloned().collect();

    for msg in snapshot {
        if msg.source.as_ref() == Some(&tether_source) {
            continue;
        }
        if msg.amount <= 0.0 {
            continue;
        }
        let Ok(link) = tethered.get(msg.target) else {
            continue;
        };
        let partner_amount = msg.amount * damage_pct / 100.0;
        if partner_amount <= 0.0 {
            continue;
        }
        messages.write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: msg.attributed_to.or(msg.dealer),
            target:        link.partner,
            amount:        partner_amount,
            source:        Some(tether_source.clone()),
            _marker:       PhantomData,
        });
    }
}
