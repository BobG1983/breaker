//! Bolt-cell collision detection via swept CCD (continuous collision detection).
//!
//! Instead of moving the bolt first and then checking for overlaps, this system
//! traces the bolt's path forward using ray-vs-expanded-AABB intersection.
//! On each hit, the bolt is placed just before the impact point, the velocity
//! is reflected, and the remaining movement continues. The bolt never overlaps
//! any cell.
//!
//! A per-frame `MAX_BOUNCES` cap (4) prevents infinite bounce loops. Cell hits
//! are naturally bounded: after reflection, the bolt travels away from the hit
//! surface for the remainder of the frame budget.
//!
//! Piercing bolts (`PiercingRemaining > 0`) pass through cells without
//! reflecting, decrementing `PiercingRemaining` on each hit.
//!
//! Cell damage and destruction are handled by the unified death pipeline via
//! [`BoltImpactCell`] and [`DamageDealt<Cell>`] messages. Wall overlap
//! detection is handled by `bolt_wall_collision`.

use std::marker::PhantomData;

use bevy::{ecs::system::SystemParam, prelude::*};
use rantzsoft_physics2d::{
    prelude::{SweepHit, reflect},
    resources::CollisionQuadtree,
};

/// Maximum number of bounces resolved per bolt per frame.
///
/// Prevents infinite loops in degenerate geometries.
pub(crate) const MAX_BOUNCES: u32 = 4;

use crate::{
    bolt::{
        components::{LastImpact, ccd_normal_to_impact_side},
        filters::ActiveFilter,
        queries::{BoltCollisionData, BoltCollisionDataItem, apply_velocity_formula},
        resources::DEFAULT_BOLT_BASE_DAMAGE,
    },
    effect_v3::{effects::phantom_bolt::components::PhantomBolt, stacking::EffectStack},
    prelude::*,
};

/// Minimum remaining travel distance below which the CCD loop terminates.
///
/// If the bolt's remaining travel is at or below this threshold, the CCD loop
/// stops — there is not enough distance left for a meaningful collision.
const MIN_REMAINING: f32 = 0.01;

/// Message writers used by the bolt-cell collision system.
type CollisionWriters<'a> = (
    MessageWriter<'a, BoltImpactCell>,
    MessageWriter<'a, DamageDealt<Cell>>,
);

/// Bundled read-only queries consumed by `bolt_cell_collision`.
///
/// Groups the hit-candidate lookup (cell marker, HP, vulnerability stack)
/// alongside the phantom-bolt marker query so the outer system signature
/// stays under the clippy argument-count threshold.
///
/// Excludes bolts from the candidate query to avoid conflicts with the
/// mutable `bolt_query`.
type CandidateQuery<'w, 's> = Query<
    'w,
    's,
    (
        Has<Cell>,
        Option<&'static Hp>,
        Option<&'static VulnerableStack>,
    ),
    Without<Bolt>,
>;

/// Triple returned by `CandidateLookup::get`:
/// `(is_cell_marker_present, optional_hp, optional_vulnerability_stack)`.
type CandidateData<'a> = (bool, Option<&'a Hp>, Option<&'a VulnerableStack>);

#[derive(SystemParam)]
pub(crate) struct CandidateLookup<'w, 's> {
    candidates:    CandidateQuery<'w, 's>,
    phantom_bolts: Query<'w, 's, (), With<PhantomBolt>>,
}

impl CandidateLookup<'_, '_> {
    /// Looks up the hit-candidate data for `entity`.
    ///
    /// Returns the same tuple the underlying candidate query produces:
    /// `(is_cell, optional Hp, optional vulnerability stack)`.
    fn get(&self, entity: Entity) -> Result<CandidateData<'_>, bevy::ecs::query::QueryEntityError> {
        self.candidates.get(entity)
    }

    /// Returns whether `entity` carries the `PhantomBolt` marker.
    fn is_phantom(&self, entity: Entity) -> bool {
        self.phantom_bolts.contains(entity)
    }
}

/// Returns the first `SweepHit` whose entity is not a pierced cell.
///
/// `cast_circle` returns hits sorted nearest-first. For each hit, check if the
/// entity is a cell that has already been pierced this frame — if so, skip it.
fn find_first_non_pierced<'a>(
    hits: &'a [SweepHit],
    candidate_lookup: &CandidateLookup,
    pierced_this_frame: &[Entity],
) -> Option<&'a SweepHit> {
    hits.iter().find(|hit| {
        let Ok((is_cell, ..)) = candidate_lookup.get(hit.entity) else {
            return false;
        };
        !(is_cell && pierced_this_frame.contains(&hit.entity))
    })
}

/// Advances bolts along their velocity, reflecting off cells via swept CCD.
///
/// For each bolt, traces a ray from its current position in the velocity
/// direction. If a cell is hit, the bolt is placed just before the
/// impact point, the velocity is reflected off the hit face, and tracing
/// continues with the remaining movement distance. Sends [`BoltImpactCell`]
/// and [`DamageDealt<Cell>`] messages for each cell hit.
pub(crate) fn bolt_cell_collision(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    quadtree: Res<CollisionQuadtree>,
    mut bolt_query: Query<BoltCollisionData, ActiveFilter>,
    candidate_lookup: CandidateLookup,
    mut writers: CollisionWriters,
    mut pierced_this_frame: Local<Vec<Entity>>,
) {
    let (ref mut hit_writer, ref mut damage_writer) = writers;
    let dt = time.delta_secs();

    for mut bolt in &mut bolt_query {
        let is_phantom = candidate_lookup.is_phantom(bolt.entity);
        let bolt_scale = bolt.collision.node_scale.map_or(1.0, |s| s.0);
        let r = bolt.collision.radius.0 * bolt_scale;
        let mut position = bolt.spatial.position.0;
        let mut velocity = bolt.spatial.velocity.0;
        let mut remaining_px = velocity.length() * dt;

        // `base_damage` is emitted raw in `DamageDealt<Cell>.amount`; the
        // `rantzsoft_dmg` pipeline applies `DamageBoostStack` and
        // `VulnerableStack` downstream.
        //
        // The pierce/reflect decision needs a predicted post-pipeline damage
        // to ask "would this hit kill the cell?". `effective_damage` here and
        // `cell_damage` in `resolve_bolt_cell_hit` duplicate the pipeline's
        // multiplier chain using `aggregate_persistent` (one-shots excluded).
        // This is a known duplication with a subtle one-shot divergence —
        // tracked as cleanup (see bolt_cell_collision pierce-decision task).
        let base_damage = bolt
            .collision
            .base_damage
            .map_or(DEFAULT_BOLT_BASE_DAMAGE, |d| d.0);
        let effective_damage = base_damage
            * bolt
                .collision
                .active_damage_boosts
                .map_or(1.0, DamageBoostStack::aggregate_persistent);

        // Clear per-bolt pierce skip set
        pierced_this_frame.clear();

        let collision_layers = CollisionLayers::new(0, CELL_LAYER);

        for _ in 0..MAX_BOUNCES {
            if remaining_px <= MIN_REMAINING {
                break;
            }

            let direction = velocity.normalize_or_zero();
            if direction == Vec2::ZERO {
                break;
            }

            // Swept circle cast: broad-phase + narrow-phase in one call.
            // Returns hits sorted nearest-first with safe position (epsilon applied).
            let hits = quadtree.quadtree.cast_circle(
                position,
                direction,
                remaining_px,
                r,
                collision_layers,
            );

            // Find the first hit that is not a pierced cell
            let first_hit = find_first_non_pierced(&hits, &candidate_lookup, &pierced_this_frame);

            let Some(hit) = first_hit else {
                // No target in path — move the full remaining distance
                position += direction * remaining_px;
                break;
            };

            // Advance to the safe position (epsilon already applied by cast_circle)
            position = hit.position;
            remaining_px = hit.remaining;

            let Some(outcome) =
                resolve_bolt_cell_hit(hit, &bolt, is_phantom, effective_damage, &candidate_lookup)
            else {
                continue;
            };

            let mut state = HitApplyState {
                velocity: &mut velocity,
                pierced_this_frame: &mut pierced_this_frame,
                base_damage,
            };
            apply_hit_outcome(
                &outcome,
                hit,
                &mut bolt,
                &mut state,
                &mut commands,
                (&mut *hit_writer, &mut *damage_writer),
            );
        }

        bolt.spatial.position.0 = position;
        bolt.spatial.velocity.0 = velocity;

        // Apply the canonical velocity formula after all CCD bounces are resolved
        apply_velocity_formula(
            &mut bolt.spatial,
            bolt.collision
                .active_speed_boosts
                .map_or(1.0, EffectStack::aggregate),
        );
    }
}

/// Per-hit decision produced by `resolve_bolt_cell_hit`. Each variant carries a snapshot
/// of the bolt's piercing charge at the instant of impact. Pierce-decision computation
/// (using fully-multiplied damage) remains in `resolve_bolt_cell_hit`; this enum carries
/// no damage value — emission supplies raw `base_damage` and the `rantzsoft_dmg` pipeline
/// applies boost/vulnerability.
enum HitOutcome {
    /// Phantom bolt passed through a cell — no reflect, no pierce
    /// decrement, no `LastImpact` stamp.
    PhantomPierce { piercing_at_impact: u32 },
    /// Normal bolt pierced a destroy-worthy cell — no reflect, pierce
    /// charge consumed, no `LastImpact` stamp.
    Pierce { piercing_at_impact: u32 },
    /// Normal bolt reflected off a cell face — velocity reflected,
    /// `LastImpact` stamped with the impact side.
    Reflect { piercing_at_impact: u32 },
}

/// Decides how a confirmed `SweepHit` against a candidate entity should be
/// resolved: phantom-pierce, pierce-through, or normal reflect.
///
/// Returns `None` when the hit entity is absent from `candidate_lookup` or is
/// not a cell — the CCD loop skips such hits.
///
/// Pure function: reads bolt, cell, and stack state but performs no
/// mutations and emits no messages. Mutation and message emission live in
/// [`apply_hit_outcome`].
fn resolve_bolt_cell_hit(
    hit: &SweepHit,
    bolt: &BoltCollisionDataItem<'_, '_>,
    is_phantom: bool,
    effective_damage: f32,
    candidate_lookup: &CandidateLookup,
) -> Option<HitOutcome> {
    let Ok((is_cell, cell_hp, vulnerability)) = candidate_lookup.get(hit.entity) else {
        return None;
    };
    if !is_cell {
        return None;
    }

    let cell_damage =
        effective_damage * vulnerability.map_or(1.0, VulnerableStack::aggregate_persistent);

    let can_pierce = bolt
        .collision
        .piercing_remaining
        .as_deref()
        .is_some_and(|pr| pr.0 > 0);
    let would_destroy = cell_hp.is_some_and(|h| h.current <= cell_damage);

    let piercing_at_impact = bolt
        .collision
        .piercing_remaining
        .as_deref()
        .map_or(0, |pr| pr.0);

    Some(if is_phantom {
        HitOutcome::PhantomPierce { piercing_at_impact }
    } else if can_pierce && would_destroy {
        HitOutcome::Pierce { piercing_at_impact }
    } else {
        HitOutcome::Reflect { piercing_at_impact }
    })
}

/// Bundled message writers for [`apply_hit_outcome`] — keeps the helper's
/// argument count under the `too_many_arguments` threshold without carrying
/// `&mut Commands` in the same struct (which would cause lifetime
/// invariance conflicts across independent `SystemParam` lifetimes).
type HitWriters<'a, 'w> = (
    &'a mut MessageWriter<'w, BoltImpactCell>,
    &'a mut MessageWriter<'w, DamageDealt<Cell>>,
);

/// Per-bolt mutable state threaded through [`apply_hit_outcome`].
///
/// Bundles the per-bolt velocity, the frame-local pierce skip set, and the
/// raw `base_damage` emitted on `DamageDealt<Cell>`. Keeps the helper
/// signature under clippy's `too_many_arguments` threshold without adding
/// `#[allow(...)]`.
struct HitApplyState<'a> {
    velocity:           &'a mut Vec2,
    pierced_this_frame: &'a mut Vec<Entity>,
    /// RAW per-hit damage emitted in `DamageDealt<Cell>.amount`. The
    /// `rantzsoft_dmg` pipeline applies `DamageBoostStack` and
    /// `VulnerableStack` — this value is NEVER pre-multiplied here.
    base_damage:        f32,
}

/// Applies a resolved [`HitOutcome`] to bolt state and emits the associated
/// `BoltImpactCell` + `DamageDealt<Cell>` messages.
///
/// Mirrors the original resolve-helper's side-effects in order:
/// 1. `PhantomPierce`: push to `pierced_this_frame`.
/// 2. `Pierce`: decrement `piercing_remaining`, push to `pierced_this_frame`.
/// 3. `Reflect`: reflect velocity off `hit.normal`, stamp `LastImpact`.
///    Then always: write `BoltImpactCell`, then `DamageDealt<Cell>` with
///    `amount = state.base_damage` (RAW — the pipeline multiplies).
fn apply_hit_outcome(
    outcome: &HitOutcome,
    hit: &SweepHit,
    bolt: &mut BoltCollisionDataItem<'_, '_>,
    state: &mut HitApplyState<'_>,
    commands: &mut Commands,
    writers: HitWriters<'_, '_>,
) {
    let (hit_writer, damage_writer) = writers;
    let piercing_at_impact = match outcome {
        HitOutcome::PhantomPierce { piercing_at_impact } => {
            state.pierced_this_frame.push(hit.entity);
            *piercing_at_impact
        }
        HitOutcome::Pierce { piercing_at_impact } => {
            if let Some(ref mut pr) = bolt.collision.piercing_remaining {
                pr.0 = pr.0.saturating_sub(1);
            }
            state.pierced_this_frame.push(hit.entity);
            *piercing_at_impact
        }
        HitOutcome::Reflect { piercing_at_impact } => {
            *state.velocity = reflect(*state.velocity, hit.normal);
            let side = ccd_normal_to_impact_side(hit.normal);
            if let Some(li) = bolt.collision.last_impact.as_mut() {
                li.position = hit.position;
                li.side = side;
            } else {
                commands.entity(bolt.entity).insert(LastImpact {
                    position: hit.position,
                    side,
                });
            }
            *piercing_at_impact
        }
    };
    hit_writer.write(BoltImpactCell {
        cell:               hit.entity,
        bolt:               bolt.entity,
        impact_normal:      hit.normal,
        piercing_remaining: piercing_at_impact,
    });
    // Emit RAW `base_damage`. The `rantzsoft_dmg` pipeline applies
    // `DamageBoostStack` (in `apply_damage_boosts::<Cell>`) and
    // `VulnerableStack` (in `apply_vulnerable::<Cell>`) exactly once before
    // `apply_damage::<Cell>` (in `DmgSystems::ApplyDamage`) consumes it.
    damage_writer.write(DamageDealt {
        dealer:        Some(bolt.entity),
        attributed_to: None,
        target:        hit.entity,
        amount:        state.base_damage,
        source:        bolt.collision.spawned_by_evolution.map(|s| s.0.clone()),
        _marker:       PhantomData::<Cell>,
    });
}
