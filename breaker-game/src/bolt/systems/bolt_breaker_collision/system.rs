//! Bolt-breaker collision detection and reflection via CCD.

use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use crate::{
    bolt::{
        components::{ImpactSide, LastImpact, PiercingRemaining, ccd_normal_to_impact_side},
        filters::ActiveFilter,
        messages::BoltImpactBreaker,
        queries::{BoltCollisionData, apply_velocity_formula},
    },
    breaker::{filters::CollisionFilterBreaker, queries::BreakerCollisionData},
    effect::effects::{piercing::ActivePiercings, size_boost::ActiveSizeBoosts},
    shared::BREAKER_LAYER,
};

/// Data bundling a CCD sweep result for breaker reflection.
struct CcdHit {
    normal: Vec2,
    safe_pos: Vec2,
    above_y: f32,
}

/// Shared per-frame physics context for CCD sweeps.
struct SweepContext<'a> {
    quadtree: &'a CollisionQuadtree,
    dt: f32,
}

/// Precomputed per-bolt geometry for collision checks.
struct BoltGeometry {
    entity: Entity,
    bolt_pos: Vec2,
    r: f32,
    expanded_half: Vec2,
    above_y: f32,
}

/// Precomputed breaker surface properties used for bolt reflection.
struct BreakerSurface {
    pos: Vec2,
    half_w: f32,
    half_h: f32,
    tilt_angle: f32,
    max_angle: f32,
    entity: Entity,
}

impl BreakerSurface {
    /// Sets bolt velocity to a unit-vector direction based on the normalized hit
    /// position on the breaker's top surface. Speed is applied separately via
    /// [`apply_velocity_formula`].
    fn reflect_top_hit(&self, impact_x: f32, bolt_velocity: &mut Velocity2D) {
        let fraction = hit_fraction(impact_x, self.pos.x, self.half_w);
        let base_angle = fraction * self.max_angle;
        let total_angle = base_angle + self.tilt_angle;
        let clamped_angle = total_angle.clamp(-self.max_angle, self.max_angle);
        bolt_velocity.0 = Vec2::new(clamped_angle.sin(), clamped_angle.cos());
    }

    /// Emits a [`BoltImpactBreaker`] message and resets piercing charges.
    fn emit_bump(
        &self,
        writer: &mut MessageWriter<BoltImpactBreaker>,
        bolt: Entity,
        piercing_remaining: &mut Option<Mut<'_, PiercingRemaining>>,
        active_piercings: Option<&ActivePiercings>,
    ) {
        writer.write(BoltImpactBreaker {
            bolt,
            breaker: self.entity,
        });
        if let (Some(pr), Some(ap)) = (piercing_remaining, active_piercings) {
            pr.0 = ap.total();
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
        bolt_pos: Vec2,
        bolt_velocity: &mut Velocity2D,
        last_impact: &mut Option<Mut<'_, LastImpact>>,
    ) -> bool {
        if bolt_velocity.0.y > 0.0 {
            return false;
        }
        self.reflect_top_hit(bolt_pos.x, bolt_velocity);
        let impact_pos = Vec2::new(bolt_pos.x, self.pos.y + self.half_h);
        stamp_last_impact(
            commands,
            bolt_entity,
            last_impact,
            impact_pos,
            ImpactSide::Top,
        );
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
        bolt_position: &mut Position2D,
        bolt_velocity: &mut Velocity2D,
        last_impact: &mut Option<Mut<'_, LastImpact>>,
    ) -> bool {
        // Overlap resolution: breaker may have moved into the bolt (e.g., bump pop).
        if is_inside_aabb(geom.bolt_pos, self.pos, geom.expanded_half) {
            bolt_position.0.y = geom.above_y;
            return self.reflect_overlap(
                commands,
                geom.entity,
                geom.bolt_pos,
                bolt_velocity,
                last_impact,
            );
        }

        let speed = bolt_velocity.0.length();
        if speed < f32::EPSILON {
            return false;
        }
        let (direction, max_dist) = (bolt_velocity.0 / speed, speed * sweep.dt);
        let Some((normal, safe_pos)) = ccd_sweep_breaker(
            sweep.quadtree,
            geom.bolt_pos,
            direction,
            max_dist,
            geom.r,
            self.pos,
            geom.expanded_half,
        ) else {
            return false;
        };
        // Only reflect downward-moving bolts; upward bolts pass through.
        if bolt_velocity.0.y > 0.0 {
            return false;
        }

        let hit = CcdHit {
            normal,
            safe_pos,
            above_y: geom.above_y,
        };
        self.reflect_ccd_hit(
            commands,
            geom.entity,
            &mut bolt_position.0,
            bolt_velocity,
            hit,
            last_impact,
        );
        true
    }

    /// Reflects a bolt off this breaker surface based on a CCD hit normal and
    /// repositions it at the safe position.
    fn reflect_ccd_hit(
        &self,
        commands: &mut Commands,
        bolt_entity: Entity,
        bolt_position: &mut Vec2,
        bolt_velocity: &mut Velocity2D,
        hit: CcdHit,
        last_impact: &mut Option<Mut<'_, LastImpact>>,
    ) {
        if hit.normal.x.abs() > hit.normal.y.abs() {
            bolt_velocity.0.x = -bolt_velocity.0.x;
            *bolt_position = hit.safe_pos;
            let side = ccd_normal_to_impact_side(hit.normal);
            stamp_last_impact(commands, bolt_entity, last_impact, hit.safe_pos, side);
        } else {
            self.reflect_top_hit(hit.safe_pos.x, bolt_velocity);
            bolt_position.x = hit.safe_pos.x;
            bolt_position.y = hit.above_y;
            let impact_pos = Vec2::new(hit.safe_pos.x, self.pos.y + self.half_h);
            stamp_last_impact(
                commands,
                bolt_entity,
                last_impact,
                impact_pos,
                ImpactSide::Top,
            );
        }
    }
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
/// Returns `Some((normal, safe_position))` on hit, `None` otherwise.
fn ccd_sweep_breaker(
    quadtree: &CollisionQuadtree,
    bolt_pos: Vec2,
    direction: Vec2,
    max_dist: f32,
    bolt_radius: f32,
    breaker_pos: Vec2,
    expanded_half: Vec2,
) -> Option<(Vec2, Vec2)> {
    let hits = quadtree.quadtree.cast_circle(
        bolt_pos,
        direction,
        max_dist,
        bolt_radius,
        CollisionLayers::new(0, BREAKER_LAYER),
    );
    hits.first().map_or_else(
        || {
            let expanded = Aabb2D::new(breaker_pos, expanded_half);
            expanded
                .ray_intersect(bolt_pos, direction, max_dist)
                .map(|ray_hit| (ray_hit.normal, ray_hit.safe_position(bolt_pos, direction)))
        },
        |qt_hit| Some((qt_hit.normal, qt_hit.position)),
    )
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
    mut writer: MessageWriter<BoltImpactBreaker>,
) {
    let Ok((breaker_entity, breaker)) = breaker_query.single() else {
        return;
    };

    let breaker_scale = breaker.node_scale.map_or(1.0, |s| s.0);
    let surface = BreakerSurface {
        pos: breaker.position.0,
        half_w: breaker.base_width.half_width()
            * breaker
                .size_boosts
                .map_or(1.0, ActiveSizeBoosts::multiplier)
            * breaker_scale,
        half_h: breaker.base_height.half_height() * breaker_scale,
        tilt_angle: breaker.tilt.angle,
        max_angle: breaker.reflection_spread.0,
        entity: breaker_entity,
    };
    let sweep = SweepContext {
        quadtree: &quadtree,
        dt: time.delta_secs(),
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

        if surface.process_bolt(
            &mut commands,
            &sweep,
            &geom,
            &mut bolt.spatial.position,
            &mut bolt.spatial.velocity,
            &mut bolt.collision.last_impact,
        ) {
            // Apply the canonical velocity formula after reflection
            apply_velocity_formula(&mut bolt.spatial, bolt.collision.active_speed_boosts);
            surface.emit_bump(
                &mut writer,
                bolt.entity,
                &mut bolt.collision.piercing_remaining,
                bolt.collision.active_piercings,
            );
        }
    }
}
