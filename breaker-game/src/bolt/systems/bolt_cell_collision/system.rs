//! Bolt-cell-wall collision detection via swept CCD (continuous collision detection).
//!
//! Instead of moving the bolt first and then checking for overlaps, this system
//! traces the bolt's path forward using ray-vs-expanded-AABB intersection.
//! On each hit, the bolt is placed just before the impact point, the velocity
//! is reflected, and the remaining movement continues. The bolt never overlaps
//! any cell or wall.
//!
//! A per-frame `MAX_BOUNCES` cap (4) prevents infinite bounce loops. Cell hits
//! are naturally bounded: after reflection, the bolt travels away from the hit
//! surface for the remainder of the frame budget.
//!
//! Piercing bolts (`PiercingRemaining > 0`) pass through cells without
//! reflecting, decrementing `PiercingRemaining` on each hit.
//!
//! Cell damage and destruction are handled by the cells domain via
//! [`BoltHitCell`] and [`DamageCell`] messages. Wall hits send
//! [`BoltHitWall`] messages for overclock triggers.

use bevy::prelude::*;
use rantzsoft_physics2d::{
    collision_layers::CollisionLayers,
    prelude::{SweepHit, reflect},
    resources::CollisionQuadtree,
};

/// Maximum number of bounces resolved per bolt per frame.
///
/// Prevents infinite loops in degenerate geometries.
pub(crate) const MAX_BOUNCES: u32 = 4;

use crate::{
    bolt::{
        BASE_BOLT_DAMAGE,
        components::Bolt,
        filters::ActiveFilter,
        messages::{BoltHitCell, BoltHitWall},
        queries::CollisionQueryBolt,
    },
    cells::{
        components::{Cell, CellHealth},
        messages::DamageCell,
    },
    shared::{CELL_LAYER, WALL_LAYER},
    wall::components::Wall,
};

/// Minimum remaining travel distance below which the CCD loop terminates.
///
/// If the bolt's remaining travel is at or below this threshold, the CCD loop
/// stops — there is not enough distance left for a meaningful collision.
const MIN_REMAINING: f32 = 0.01;

/// Message writers used by the bolt-cell-wall collision system.
type CollisionWriters<'a> = (
    MessageWriter<'a, BoltHitCell>,
    MessageWriter<'a, DamageCell>,
    MessageWriter<'a, BoltHitWall>,
);

/// Query for looking up game-specific data by entity ID after `cast_circle`
/// identifies a hit.
///
/// Excludes bolts to avoid query conflicts with the mutable `bolt_query`.
type CandidateLookup<'w, 's> =
    Query<'w, 's, (Has<Cell>, Has<Wall>, Option<&'static CellHealth>), Without<Bolt>>;

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

/// Advances bolts along their velocity, reflecting off cells and walls via swept CCD.
///
/// For each bolt, traces a ray from its current position in the velocity
/// direction. If a cell or wall is hit, the bolt is placed just before the
/// impact point, the velocity is reflected off the hit face, and tracing
/// continues with the remaining movement distance. Sends [`BoltHitCell`]
/// and [`DamageCell`] messages for each cell hit. Sends [`BoltHitWall`]
/// messages for each wall hit.
pub(crate) fn bolt_cell_collision(
    time: Res<Time<Fixed>>,
    quadtree: Res<CollisionQuadtree>,
    mut bolt_query: Query<CollisionQueryBolt, ActiveFilter>,
    candidate_lookup: CandidateLookup,
    mut writers: CollisionWriters,
    mut pierced_this_frame: Local<Vec<Entity>>,
) {
    let (ref mut hit_writer, ref mut damage_writer, ref mut wall_hit_writer) = writers;
    let dt = time.delta_secs();

    for (
        bolt_entity,
        mut bolt_position,
        mut bolt_vel,
        _,
        bolt_radius,
        mut piercing_remaining,
        piercing,
        damage_boost,
        bolt_entity_scale,
        spawned_by_evo,
    ) in &mut bolt_query
    {
        let bolt_scale = bolt_entity_scale.map_or(1.0, |s| s.0);
        let r = bolt_radius.0 * bolt_scale;
        let mut position = bolt_position.0;
        let mut velocity = bolt_vel.0;
        let mut remaining = velocity.length() * dt;

        // Effective damage for pierce lookahead (compared against cell HP).
        // must match `handle_cell_hit` damage formula
        let effective_damage = BASE_BOLT_DAMAGE * (1.0 + damage_boost.map_or(0.0, |b| b.0));

        // Clear per-bolt pierce skip set
        pierced_this_frame.clear();

        let collision_layers = CollisionLayers::new(0, CELL_LAYER | WALL_LAYER);

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
            let Ok((is_cell, _is_wall, cell_health)) = candidate_lookup.get(hit.entity) else {
                // Entity not in lookup (shouldn't happen) — skip
                continue;
            };

            if is_cell {
                // Check if this bolt can pierce this cell
                let can_pierce = piercing_remaining.as_deref().is_some_and(|pr| pr.0 > 0);
                let cell_hp = cell_health.map(|h| h.current);
                let would_destroy = cell_hp.is_some_and(|hp| hp <= effective_damage);

                if can_pierce && would_destroy {
                    // PIERCE: do NOT reflect; decrement remaining pierces
                    if let Some(ref mut pr) = piercing_remaining {
                        pr.0 = pr.0.saturating_sub(1);
                    }
                    pierced_this_frame.push(hit.entity);
                    // Continue CCD loop — velocity unchanged, direction unchanged
                } else {
                    // NORMAL: reflect
                    velocity = reflect(velocity, hit.normal);
                }
                hit_writer.write(BoltHitCell {
                    cell: hit.entity,
                    bolt: bolt_entity,
                });
                damage_writer.write(DamageCell {
                    cell: hit.entity,
                    damage: effective_damage,
                    source_chip: spawned_by_evo.map(|s| s.0.clone()),
                });
            } else {
                // WALL HIT: reflect and reset `PiercingRemaining`
                velocity = reflect(velocity, hit.normal);
                // Reset `PiercingRemaining` to `Piercing.0`
                if let (Some(pr), Some(p)) = (&mut piercing_remaining, piercing) {
                    pr.0 = p.0;
                }
                wall_hit_writer.write(BoltHitWall {
                    bolt: bolt_entity,
                    wall: hit.entity,
                });
            }
        }

        bolt_position.0 = position;
        bolt_vel.0 = velocity;
    }
}
