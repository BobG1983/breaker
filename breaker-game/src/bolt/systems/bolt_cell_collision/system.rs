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

use bevy::prelude::*;
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
        queries::{BoltCollisionData, apply_velocity_formula},
        resources::DEFAULT_BOLT_BASE_DAMAGE,
    },
    effect_v3::{effects::VulnerableConfig, stacking::EffectStack},
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

/// Query for looking up game-specific data by entity ID after `cast_circle`
/// identifies a hit.
///
/// Excludes bolts to avoid query conflicts with the mutable `bolt_query`.
type CandidateLookup<'w, 's> = Query<
    'w,
    's,
    (
        Has<Cell>,
        Option<&'static Hp>,
        Option<&'static EffectStack<VulnerableConfig>>,
    ),
    Without<Bolt>,
>;

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
        let bolt_scale = bolt.collision.node_scale.map_or(1.0, |s| s.0);
        let r = bolt.collision.radius.0 * bolt_scale;
        let mut position = bolt.spatial.position.0;
        let mut velocity = bolt.spatial.velocity.0;
        let mut remaining = velocity.length() * dt;

        // Effective damage for pierce lookahead (compared against cell HP).
        // Must match the damage formula applied by `apply_damage::<Cell>`.
        let base_damage = bolt
            .collision
            .base_damage
            .map_or(DEFAULT_BOLT_BASE_DAMAGE, |d| d.0);
        let effective_damage = base_damage
            * bolt
                .collision
                .active_damage_boosts
                .map_or(1.0, EffectStack::aggregate);

        // Clear per-bolt pierce skip set
        pierced_this_frame.clear();

        let collision_layers = CollisionLayers::new(0, CELL_LAYER);

        for _ in 0..MAX_BOUNCES {
            if remaining <= MIN_REMAINING {
                break;
            }

            let direction = velocity.normalize_or_zero();
            if direction == Vec2::ZERO {
                break;
            }

            // Swept circle cast: broad-phase + narrow-phase in one call.
            // Returns hits sorted nearest-first with safe position (epsilon applied).
            let hits =
                quadtree
                    .quadtree
                    .cast_circle(position, direction, remaining, r, collision_layers);

            // Find the first hit that is not a pierced cell
            let best = find_first_non_pierced(&hits, &candidate_lookup, &pierced_this_frame);

            let Some(hit) = best else {
                // No target in path — move the full remaining distance
                position += direction * remaining;
                break;
            };

            // Advance to the safe position (epsilon already applied by cast_circle)
            position = hit.position;
            remaining = hit.remaining;

            // Look up game-specific data for the hit entity
            let Ok((is_cell, cell_hp, vulnerability)) = candidate_lookup.get(hit.entity) else {
                // Entity not in lookup (shouldn't happen) — skip
                continue;
            };

            if !is_cell {
                // Non-cell entity in cell-only query — skip
                continue;
            }

            // Per-cell damage including vulnerability
            let cell_damage = effective_damage * vulnerability.map_or(1.0, EffectStack::aggregate);

            // Check if this bolt can pierce this cell
            let can_pierce = bolt
                .collision
                .piercing_remaining
                .as_deref()
                .is_some_and(|pr| pr.0 > 0);
            let cell_hp_value = cell_hp.map(|h| h.current);
            let would_destroy = cell_hp_value.is_some_and(|hp| hp <= cell_damage);

            // Capture piercing charges BEFORE any pierce-through decrement so
            // the armor check system sees the same value the bolt had at the
            // instant of impact.
            let piercing_at_impact = bolt
                .collision
                .piercing_remaining
                .as_deref()
                .map_or(0, |pr| pr.0);

            if can_pierce && would_destroy {
                // PIERCE: do NOT reflect; decrement remaining pierces
                // Do NOT stamp LastImpact on pierce-through.
                if let Some(ref mut pr) = bolt.collision.piercing_remaining {
                    pr.0 = pr.0.saturating_sub(1);
                }
                pierced_this_frame.push(hit.entity);
                // Continue CCD loop — velocity unchanged, direction unchanged
            } else {
                // NORMAL: reflect
                velocity = reflect(velocity, hit.normal);
                // Stamp LastImpact on reflect only
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
            }
            hit_writer.write(BoltImpactCell {
                cell:               hit.entity,
                bolt:               bolt.entity,
                impact_normal:      hit.normal,
                piercing_remaining: piercing_at_impact,
            });
            damage_writer.write(DamageDealt {
                dealer:      Some(bolt.entity),
                target:      hit.entity,
                amount:      cell_damage,
                source_chip: bolt.collision.spawned_by_evolution.map(|s| s.0.clone()),
                _marker:     PhantomData::<Cell>,
            });
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
