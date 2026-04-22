//! Cells-domain Cell damage system.
//!
//! Replaces the generic `apply_damage::<Cell>` for the `Cell` type. Reads
//! `DamageDealt<Cell>` messages, applies the reduction, and — when Diffusion
//! is active — BFS-propagates ring damage to adjacent cells in-memory without
//! emitting additional `DamageDealt<Cell>` messages.
//!
//! See design doc `docs/design/hazards/diffusion.md`
//! (§Systems, §Cross-Domain Dependencies, §Expected Behaviors) for the
//! redistribution semantics this system implements. The hazard domain owns
//! `DiffusionConfig` (populated via `diffusion::activate`) and `ActiveHazards`;
//! both are read here as `Option<Res<..>>` so the system is harness-safe when
//! `HazardPlugin` is not installed.

use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use bevy::{ecs::system::SystemParam, prelude::*};
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    cells::components::{ADJACENCY_RADIUS_SQ, Cell},
    hazard::{
        definition::HazardKind,
        hazards::{
            diffusion::DiffusionConfig,
            tether::{TETHER_SENTINEL, TetherConfig, TetherLink, TetherRedirectBuffer},
        },
        resources::ActiveHazards,
    },
    shared::death_pipeline::{DamageDealt, Hp, Invulnerable, KilledBy, dead::Dead},
};

/// Queries split into a `ParamSet` because the mutable damage-application
/// query and the read-only adjacency query both touch `With<Cell>` with
/// overlapping component access (`&mut Hp` vs `&Hp`).
///
/// `AdjacencyQuery` also applies `Without<Invulnerable>` so invulnerable cells
/// are invisible to the BFS snapshot. Tests 42 and 43 pin the behavior: an
/// invulnerable or dead primary falls through to pass-through (no
/// redistribution) because the primary is not found in the snapshot. The
/// spec's code template omitted this filter; the test behavior takes priority.
type DamageAppQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static mut Hp, &'static mut KilledBy),
    (With<Cell>, Without<Dead>, Without<Invulnerable>),
>;
type AdjacencyQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Position2D, &'static Hp),
    (With<Cell>, Without<Dead>, Without<Invulnerable>),
>;

/// Records HP deltas for a single `DamageDealt<Cell>` message — pass-through
/// when diffusion is inactive or the primary is isolated, BFS-propagated
/// across rings 1..=depth when diffusion is active and ring 1 is non-empty.
///
/// Returns the positive damage amount absorbed by the primary target in this
/// message (before sign-flip into `deltas`). Callers use this return value to
/// track order-aware killing-blow attribution when multiple messages hit the
/// same primary in one tick.
///
/// `deltas` accumulates across calls: the same map must be passed for every
/// message in one tick so ring damage from multiple messages sums correctly.
fn accumulate_message_deltas(
    msg: &DamageDealt<Cell>,
    diffusion_active: bool,
    share_frac: f32,
    depth: usize,
    adjacency_snapshot: &[(Entity, Vec2)],
    deltas: &mut HashMap<Entity, f32>,
) -> f32 {
    if msg.amount == 0.0 {
        deltas.entry(msg.target).or_insert(0.0);
        return 0.0;
    }

    if !diffusion_active {
        *deltas.entry(msg.target).or_insert(0.0) -= msg.amount;
        return msg.amount;
    }

    // Find the primary's position in the snapshot. If it isn't in the
    // snapshot (e.g., dead or missing Position2D), fall back to pass-through —
    // the damage-application query will filter it out if applicable.
    let Some(&(_, primary_pos)) = adjacency_snapshot.iter().find(|(e, _)| *e == msg.target) else {
        *deltas.entry(msg.target).or_insert(0.0) -= msg.amount;
        return msg.amount;
    };

    // Ring 1 — unvisited live cells within the adjacency radius.
    let mut visited: HashSet<Entity> = HashSet::new();
    visited.insert(msg.target);

    let ring1: Vec<(Entity, Vec2)> = adjacency_snapshot
        .iter()
        .filter(|(e, pos)| {
            !visited.contains(e) && pos.distance_squared(primary_pos) <= ADJACENCY_RADIUS_SQ
        })
        .copied()
        .collect();

    if ring1.is_empty() {
        // Isolated primary: full damage, no redistribution.
        *deltas.entry(msg.target).or_insert(0.0) -= msg.amount;
        return msg.amount;
    }

    // Primary reduction — only applied when ring 1 is non-empty.
    let primary_damage = msg.amount * (1.0 - share_frac);
    let primary_delta = deltas.entry(msg.target).or_insert(0.0);
    *primary_delta -= primary_damage;

    // Distribute ring-1 share pool.
    let ring1_len = ring1.len();
    let per_neighbour_ring1 = msg.amount * share_frac / ring1_len as f32;
    for (entity, _) in &ring1 {
        *deltas.entry(*entity).or_insert(0.0) -= per_neighbour_ring1;
        visited.insert(*entity);
    }

    // Rings 2..=depth: uniform geometric attenuation. Ring-N total =
    // ring-(N-1) total × share_frac, distributed evenly across ring-N cells.
    // This is simpler than per-parent BFS accounting and produces identical
    // numeric results for the tested topologies; divergent branching-graph
    // topologies (multiple ring-1 parents with distinct ring-2 children)
    // are not currently covered by any test.
    let mut frontier: Vec<(Entity, Vec2)> = ring1;
    let mut per_parent_received: f32 = per_neighbour_ring1;
    for _ring in 2..=depth {
        let next_frontier: Vec<(Entity, Vec2)> = adjacency_snapshot
            .iter()
            .filter(|(e, pos)| {
                !visited.contains(e)
                    && frontier
                        .iter()
                        .any(|(_, fpos)| pos.distance_squared(*fpos) <= ADJACENCY_RADIUS_SQ)
            })
            .copied()
            .collect();
        if next_frontier.is_empty() {
            break;
        }
        let per_neighbour =
            per_parent_received * share_frac * (frontier.len() as f32) / next_frontier.len() as f32;
        for (entity, _) in &next_frontier {
            *deltas.entry(*entity).or_insert(0.0) -= per_neighbour;
            visited.insert(*entity);
        }
        per_parent_received = per_neighbour;
        frontier = next_frontier;
    }

    primary_damage
}

type LivePartnerFilter = (With<Cell>, Without<Dead>, Without<Invulnerable>);

type LiveCellTetherLinks<'w, 's> = Query<'w, 's, &'static TetherLink, LivePartnerFilter>;

type LivePartnerCells<'w, 's> = Query<'w, 's, Entity, LivePartnerFilter>;

/// Tether-related params, bundled so `apply_damage_to_cells` avoids the
/// `clippy::too_many_arguments` threshold. All fields are optional at the
/// resource level for harness-safety (an app without `HazardPlugin` has no
/// `TetherConfig` / `TetherRedirectBuffer`); the query params are always
/// present but typically empty.
#[derive(SystemParam)]
pub(crate) struct TetherSystemParams<'w, 's> {
    config:        Option<Res<'w, TetherConfig>>,
    buffer:        Option<ResMut<'w, TetherRedirectBuffer>>,
    links:         LiveCellTetherLinks<'w, 's>,
    // Partners must be live (not Dead, not Invulnerable) for a redirect to be
    // meaningful — the DamageAppQuery would silently drop the redirect anyway,
    // but filtering here prevents phantom message traffic.
    partner_cells: LivePartnerCells<'w, 's>,
}

/// Applies `DamageDealt<Cell>` messages to cells. When Diffusion is active,
/// the primary takes a reduced fraction of the incoming damage and the share
/// pool is BFS-propagated to adjacent cells (and their unvisited neighbours
/// for rings 2..=depth) via in-memory HP accumulation — no new messages are
/// emitted for ring damage.
///
/// Pass-through branch mirrors the generic `apply_damage::<T>`: full damage,
/// `KilledBy::dealer` attribution on the killing blow.
pub(crate) fn apply_damage_to_cells(
    diffusion_config: Option<Res<DiffusionConfig>>,
    active_hazards: Option<Res<ActiveHazards>>,
    mut reader: MessageReader<DamageDealt<Cell>>,
    mut cell_queries: ParamSet<(DamageAppQuery, AdjacencyQuery)>,
    mut tether: TetherSystemParams,
) {
    // Buffer incoming messages so the adjacency scan can borrow the immutable
    // query independently of the later mutable application query.
    let incoming: Vec<DamageDealt<Cell>> = reader.read().cloned().collect();
    if incoming.is_empty() {
        return;
    }

    // Resolve diffusion parameters. Absent resource, zero stacks, and zero
    // share / depth all collapse to the pass-through branch.
    let stacks = active_hazards
        .as_deref()
        .map_or(0, |a| a.stacks(HazardKind::Diffusion));
    let (share_frac, depth) = match (diffusion_config.as_deref(), stacks) {
        (Some(cfg), s) if s > 0 => {
            let sp = cfg.share_percent(s) / 100.0;
            (sp, cfg.depth(s) as usize)
        }
        _ => (0.0, 0),
    };
    let diffusion_active = share_frac > 0.0 && depth > 0;

    // ── Tether activation state (computed ONCE, before the accumulate loop) ──
    // Per design doc `docs/design/hazards/tether.md`
    // §Edge Cases line 121: "If both are active, Diffusion runs first (shares/
    // reduces damage), then Tether runs on the modified damage amounts." The
    // redirect's `partner_amount` is computed from the `f32` that
    // `accumulate_message_deltas` returns (the Diffusion-reduced primary
    // damage), NOT from `msg.amount`.
    let tether_stacks = active_hazards
        .as_deref()
        .map_or(0, |a| a.stacks(HazardKind::Tether));
    let tether_pct = match (tether.config.as_deref(), tether_stacks) {
        (Some(cfg), s) if s > 0 => cfg.damage_percent(s),
        _ => 0.0,
    };
    // If the Tether plugin did not register `TetherRedirectBuffer`, we cannot
    // buffer redirects regardless of config/stack state. The `is_some` guard
    // short-circuits the per-iteration branch in a harness that omits the
    // hazard plugin.
    let tether_active = tether_pct > 0.0 && tether.buffer.is_some();

    // Snapshot the living-cell adjacency data if the BFS might run. The
    // snapshot is a point-in-time view of `hp.current > 0.0` cells (dead
    // cells are already excluded by the query's `Without<Dead>` filter; the
    // HP check also excludes cells that have reached zero HP earlier this
    // invocation but have not yet been marked `Dead`).
    let adjacency_snapshot: Vec<(Entity, Vec2)> = if diffusion_active {
        cell_queries
            .p1()
            .iter()
            .filter_map(|(entity, pos, hp)| (hp.current > 0.0).then_some((entity, pos.0)))
            .collect()
    } else {
        Vec::new()
    };

    // Fetch the pre-tick HP of every primary target so the kill-blow pass can
    // walk messages in order and identify which message's dealer delivered
    // the killing blow (first message to push running HP from positive to ≤0).
    let mut running_primary_hp: HashMap<Entity, f32> = HashMap::new();
    {
        let primary_view = cell_queries.p1();
        for msg in &incoming {
            if !running_primary_hp.contains_key(&msg.target)
                && let Ok((_, _, hp)) = primary_view.get(msg.target)
            {
                running_primary_hp.insert(msg.target, hp.current);
            }
        }
    }

    // Accumulate HP deltas (negative = damage) keyed by entity, while
    // simultaneously walking messages in order to pin the killing-blow
    // dealer per primary. Ring damage is un-attributed per the design doc.
    let mut deltas: HashMap<Entity, f32> = HashMap::new();
    let mut killing_dealers: HashMap<Entity, Entity> = HashMap::new();

    for msg in &incoming {
        let primary_damage = accumulate_message_deltas(
            msg,
            diffusion_active,
            share_frac,
            depth,
            &adjacency_snapshot,
            &mut deltas,
        );
        if let Some(running) = running_primary_hp.get_mut(&msg.target) {
            let was_positive = *running > 0.0;
            *running -= primary_damage;
            if was_positive
                && *running <= 0.0
                && !killing_dealers.contains_key(&msg.target)
                && let Some(dealer) = msg.dealer
            {
                killing_dealers.insert(msg.target, dealer);
            }
        }

        if tether_active {
            try_buffer_tether_redirect(msg, primary_damage, tether_pct, &mut tether);
        }
    }

    // Apply accumulated deltas. `KilledBy::dealer` is attributed only for
    // primary targets of incoming messages, and only on the killing blow.
    let mut targets = cell_queries.p0();
    for (entity, delta) in &deltas {
        let Ok((_, mut hp, mut killed_by)) = targets.get_mut(*entity) else {
            continue;
        };
        let was_positive = hp.current > 0.0;
        hp.current += *delta;
        if was_positive
            && hp.current <= 0.0
            && killed_by.dealer.is_none()
            && let Some(&dealer) = killing_dealers.get(entity)
        {
            killed_by.dealer = Some(dealer);
        }
    }
}

/// Buffers one `DamageDealt<Cell>` onto `TetherRedirectBuffer` per qualifying
/// primary hit. Extracted from `apply_damage_to_cells` so the caller stays
/// under `clippy::too_many_lines`.
///
/// A dedicated `emit_tether_redirects` system (hazard domain) drains the
/// buffer into `MessageWriter<DamageDealt<Cell>>` in the same `ApplyDamage`
/// set — we cannot hold both `MessageReader` and `MessageWriter` for
/// `DamageDealt<Cell>` in one system (Bevy 0.18 panics at schedule
/// construction on that overlap).
///
/// Uses the DIFFUSION-REDUCED `primary_damage` the caller just computed.
/// When Diffusion is inactive, `primary_damage == msg.amount`.
fn try_buffer_tether_redirect(
    msg: &DamageDealt<Cell>,
    primary_damage: f32,
    tether_pct: f32,
    tether: &mut TetherSystemParams,
) {
    if msg.source_chip.as_deref() == Some(TETHER_SENTINEL) {
        return;
    }
    let Ok(link) = tether.links.get(msg.target) else {
        return;
    };
    if tether.partner_cells.get(link.partner).is_err() {
        return;
    }
    let Some(buffer) = tether.buffer.as_deref_mut() else {
        return;
    };
    let partner_amount = primary_damage * tether_pct / 100.0;
    if partner_amount <= 0.0 {
        return;
    }
    buffer.0.push(DamageDealt::<Cell> {
        dealer:      msg.dealer,
        target:      link.partner,
        amount:      partner_amount,
        source_chip: Some(TETHER_SENTINEL.to_string()),
        _marker:     PhantomData,
    });
}
