//! Bolt-breaker collision detection and reflection via CCD.

use bevy::prelude::*;
use rantzsoft_physics2d::resources::CollisionQuadtree;

use crate::{
    bolt::{
        components::{ImpactSide, LastImpact, PiercingRemaining, ccd_normal_to_impact_side},
        filters::ActiveFilter,
        queries::{BoltCollisionData, apply_velocity_formula},
    },
    breaker::{
        components::PhantomBreaker, filters::CollisionFilterBreaker, queries::BreakerCollisionData,
    },
    effect_v3::{effects::PiercingConfig, stacking::EffectStack},
    prelude::*,
};

/// Data bundling a CCD sweep result for breaker reflection.
struct CcdHit {
    normal:   Vec2,
    safe_pos: Vec2,
    above_y:  f32,
}

/// Shared per-frame physics context for CCD sweeps.
struct SweepContext<'a> {
    quadtree: &'a CollisionQuadtree,
    dt:       f32,
}

/// Precomputed per-bolt geometry for collision checks.
struct BoltGeometry {
    entity:        Entity,
    bolt_pos:      Vec2,
    r:             f32,
    expanded_half: Vec2,
    above_y:       f32,
}

/// A swept ray-circle for CCD lookup against a single breaker.
struct SweepRay {
    origin:    Vec2,
    direction: Vec2,
    max_dist:  f32,
    radius:    f32,
}

/// Precomputed breaker surface properties used for bolt reflection.
///
/// `is_phantom` is per-breaker invariant state — phantom breakers suppress
/// tilt, spread, `LastImpact` stamping, and piercing reset while still
/// reflecting core velocity.
struct BreakerSurface {
    pos:        Vec2,
    half_w:     f32,
    half_h:     f32,
    tilt_angle: f32,
    max_angle:  f32,
    entity:     Entity,
    is_phantom: bool,
}

/// Mutable bolt state bundle threaded through the reflection helpers.
struct BoltReflectState<'a, 'w> {
    position:    &'a mut Position2D,
    velocity:    &'a mut Velocity2D,
    last_impact: &'a mut Option<Mut<'w, LastImpact>>,
}

impl BreakerSurface {
    /// Sets bolt velocity to a unit-vector direction based on the normalized hit
    /// position on the breaker's top surface. Speed is applied separately via
    /// [`apply_velocity_formula`].
    fn reflect_top_hit(&self, impact_x: f32, bolt_velocity: &mut Velocity2D) {
        let fraction = hit_fraction(impact_x, self.pos.x, self.half_w);
        let base_angle = if self.is_phantom {
            0.0
        } else {
            fraction * self.max_angle
        };
        let tilt_term = if self.is_phantom {
            0.0
        } else {
            self.tilt_angle
        };
        let total_angle = base_angle + tilt_term;
        let clamped_angle = total_angle.clamp(-self.max_angle, self.max_angle);
        bolt_velocity.0 = Vec2::new(clamped_angle.sin(), clamped_angle.cos());
    }

    /// Emits a [`BoltImpactBreaker`] message and resets piercing charges.
    fn emit_bump(
        &self,
        writer: &mut MessageWriter<BoltImpactBreaker>,
        bolt: Entity,
        piercing_remaining: &mut Option<Mut<'_, PiercingRemaining>>,
        active_piercings: Option<&EffectStack<PiercingConfig>>,
    ) {
        writer.write(BoltImpactBreaker {
            bolt,
            breaker: self.entity,
        });
        if self.is_phantom {
            return;
        }
        if let (Some(pr), Some(ap)) = (piercing_remaining, active_piercings) {
            pr.0 = piercing_aggregate_to_u32(ap.aggregate());
        }
    }

    /// Reflects a downward-moving bolt off the breaker top surface during overlap.
    ///
    /// Returns `true` if the bolt was reflected (and bump should be emitted),
    /// `false` if the bolt was moving upward (no reflection needed).
    fn reflect_overlap(
        &self,
        commands: &mut Commands,
        bolt_entity: Entity,
        state: &mut BoltReflectState<'_, '_>,
    ) -> bool {
        if state.velocity.0.y > 0.0 {
            return false;
        }
        let impact_x = state.position.0.x;
        self.reflect_top_hit(impact_x, state.velocity);
        if !self.is_phantom {
            let impact_pos = Vec2::new(impact_x, self.pos.y + self.half_h);
            stamp_last_impact(
                commands,
                bolt_entity,
                state.last_impact,
                impact_pos,
                ImpactSide::Top,
            );
        }
        true
    }

    /// Processes a single bolt-breaker collision: overlap resolution then CCD sweep.
    ///
    /// Returns `true` if a bump occurred and the caller should emit [`BoltImpactBreaker`].
    fn process_bolt(
        &self,
        commands: &mut Commands,
        sweep: &SweepContext,
        geom: &BoltGeometry,
        state: &mut BoltReflectState<'_, '_>,
    ) -> bool {
        // Overlap resolution: breaker may have moved into the bolt (e.g., bump pop).
        if is_inside_aabb(geom.bolt_pos, self.pos, geom.expanded_half) {
            state.position.0.y = geom.above_y;
            return self.reflect_overlap(commands, geom.entity, state);
        }

        let speed = state.velocity.0.length();
        if speed < f32::EPSILON {
            return false;
        }
        let ray = SweepRay {
            origin:    geom.bolt_pos,
            direction: state.velocity.0 / speed,
            max_dist:  speed * sweep.dt,
            radius:    geom.r,
        };
        let Some((normal, safe_pos)) = ccd_sweep_breaker(
            sweep.quadtree,
            self.entity,
            &ray,
            self.pos,
            geom.expanded_half,
        ) else {
            return false;
        };
        // Only reflect downward-moving bolts; upward bolts pass through.
        if state.velocity.0.y > 0.0 {
            return false;
        }

        let hit = CcdHit {
            normal,
            safe_pos,
            above_y: geom.above_y,
        };
        self.reflect_ccd_hit(commands, geom.entity, hit, state);
        true
    }

    /// Reflects a bolt off this breaker surface based on a CCD hit normal and
    /// repositions it at the safe position.
    fn reflect_ccd_hit(
        &self,
        commands: &mut Commands,
        bolt_entity: Entity,
        hit: CcdHit,
        state: &mut BoltReflectState<'_, '_>,
    ) {
        if hit.normal.x.abs() > hit.normal.y.abs() {
            state.velocity.0.x = -state.velocity.0.x;
            state.position.0 = hit.safe_pos;
            if !self.is_phantom {
                let side = ccd_normal_to_impact_side(hit.normal);
                stamp_last_impact(commands, bolt_entity, state.last_impact, hit.safe_pos, side);
            }
        } else {
            self.reflect_top_hit(hit.safe_pos.x, state.velocity);
            state.position.0.x = hit.safe_pos.x;
            state.position.0.y = hit.above_y;
            if !self.is_phantom {
                let impact_pos = Vec2::new(hit.safe_pos.x, self.pos.y + self.half_h);
                stamp_last_impact(
                    commands,
                    bolt_entity,
                    state.last_impact,
                    impact_pos,
                    ImpactSide::Top,
                );
            }
        }
    }
}

/// Converts the piercing aggregate (f32 sum of small u32 charges) back to u32.
///
/// Piercing charges are always non-negative and small enough for lossless
/// round-trip through f32.
const fn piercing_aggregate_to_u32(aggregate: f32) -> u32 {
    aggregate.round().max(0.0) as u32
}

/// Returns `true` when `point` lies inside the axis-aligned box centred on
/// `center` with half-extents `half`.
fn is_inside_aabb(point: Vec2, center: Vec2, half: Vec2) -> bool {
    point.x > center.x - half.x
        && point.x < center.x + half.x
        && point.y > center.y - half.y
        && point.y < center.y + half.y
}

/// Computes the normalized hit position on the breaker surface.
///
/// Returns a value in `[-1.0, 1.0]` where -1 is the left edge and +1 is the
/// right edge of the breaker.
fn hit_fraction(impact_x: f32, breaker_x: f32, half_w: f32) -> f32 {
    let clamped_x = impact_x.clamp(breaker_x - half_w, breaker_x + half_w);
    ((clamped_x - breaker_x) / half_w).clamp(-1.0, 1.0)
}

/// CCD sweep via quadtree, with fallback to manual AABB ray intersection
/// for breakers not yet in the quadtree (e.g., missing `Aabb2D`/`CollisionLayers`).
///
/// Filters quadtree results to hits against `breaker_entity` so each outer-loop
/// iteration only sees its own breaker — required when multiple breakers
/// (e.g., real + phantom) co-exist in the same `BREAKER_LAYER`.
///
/// Returns `Some((normal, safe_position))` on hit, `None` otherwise.
fn ccd_sweep_breaker(
    quadtree: &CollisionQuadtree,
    breaker_entity: Entity,
    ray: &SweepRay,
    breaker_pos: Vec2,
    expanded_half: Vec2,
) -> Option<(Vec2, Vec2)> {
    let hits = quadtree.quadtree.cast_circle(
        ray.origin,
        ray.direction,
        ray.max_dist,
        ray.radius,
        CollisionLayers::new(0, BREAKER_LAYER),
    );
    if let Some(qt_hit) = hits.iter().find(|h| h.entity == breaker_entity) {
        return Some((qt_hit.normal, qt_hit.position));
    }
    let expanded = Aabb2D::new(breaker_pos, expanded_half);
    expanded
        .ray_intersect(ray.origin, ray.direction, ray.max_dist)
        .map(|ray_hit| {
            (
                ray_hit.normal,
                ray_hit.safe_position(ray.origin, ray.direction),
            )
        })
}

/// Inserts or updates `LastImpact` on a bolt entity.
fn stamp_last_impact(
    commands: &mut Commands,
    bolt_entity: Entity,
    last_impact: &mut Option<Mut<'_, LastImpact>>,
    position: Vec2,
    side: ImpactSide,
) {
    if let Some(li) = last_impact {
        li.position = position;
        li.side = side;
    } else {
        commands
            .entity(bolt_entity)
            .insert(LastImpact { position, side });
    }
}

/// Detects bolt-breaker collisions via swept CCD and overwrites bolt direction.
///
/// Includes overlap resolution: if the breaker has moved into the bolt (e.g.,
/// bump pop), the bolt is pushed above the breaker and reflected if moving
/// downward. CCD alone cannot detect this case since it only sweeps bolt
/// movement.
pub(crate) fn bolt_breaker_collision(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    quadtree: Res<CollisionQuadtree>,
    mut bolt_query: Query<BoltCollisionData, ActiveFilter>,
    breaker_query: Query<(Entity, BreakerCollisionData), CollisionFilterBreaker>,
    phantom_query: Query<(), With<PhantomBreaker>>,
    mut writer: MessageWriter<BoltImpactBreaker>,
) {
    let sweep = SweepContext {
        quadtree: &quadtree,
        dt:       time.delta_secs(),
    };

    for (breaker_entity, breaker) in &breaker_query {
        let breaker_scale = breaker.node_scale.map_or(1.0, |s| s.0);
        let surface = BreakerSurface {
            pos:        breaker.position.0,
            half_w:     breaker.base_width.half_width()
                * breaker.size_boosts.map_or(1.0, EffectStack::aggregate)
                * breaker_scale,
            half_h:     breaker.base_height.half_height() * breaker_scale,
            tilt_angle: breaker.tilt.angle,
            max_angle:  breaker.reflection_spread.0,
            entity:     breaker_entity,
            is_phantom: phantom_query.contains(breaker_entity),
        };

        for mut bolt in &mut bolt_query {
            let bolt_scale = bolt.collision.node_scale.map_or(1.0, |s| s.0);
            let r = bolt.collision.radius.0 * bolt_scale;
            let geom = BoltGeometry {
                entity: bolt.entity,
                bolt_pos: bolt.spatial.position.0,
                r,
                expanded_half: Vec2::new(surface.half_w + r, surface.half_h + r),
                above_y: surface.pos.y + surface.half_h + r,
            };

            let bumped = {
                let mut state = BoltReflectState {
                    position:    &mut bolt.spatial.position,
                    velocity:    &mut bolt.spatial.velocity,
                    last_impact: &mut bolt.collision.last_impact,
                };
                surface.process_bolt(&mut commands, &sweep, &geom, &mut state)
            };
            if bumped {
                let speed_mult = bolt
                    .collision
                    .active_speed_boosts
                    .map_or(1.0, EffectStack::aggregate);
                if surface.is_phantom {
                    // Phantom hits must remain pure-vertical: scale the unit-vector
                    // velocity by the clamped base speed without `apply_velocity_formula`,
                    // which would otherwise re-introduce horizontal motion via the
                    // `min_angle_v` clamp.
                    let effective_base = bolt.spatial.base_speed.0 * speed_mult;
                    let min = bolt.spatial.min_speed.map_or(0.0, |s| s.0);
                    let max = bolt.spatial.max_speed.map_or(f32::MAX, |s| s.0);
                    let speed = effective_base.clamp(min, max);
                    bolt.spatial.velocity.0 *= speed;
                } else {
                    apply_velocity_formula(&mut bolt.spatial, speed_mult);
                }
                surface.emit_bump(
                    &mut writer,
                    bolt.entity,
                    &mut bolt.collision.piercing_remaining,
                    bolt.collision.active_piercings,
                );
            }
        }
    }
}
