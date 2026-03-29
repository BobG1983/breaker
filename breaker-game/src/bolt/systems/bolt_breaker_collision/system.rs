//! Bolt-breaker collision detection and reflection via CCD.

use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::{
        components::{PiercingRemaining, enforce_min_angle},
        filters::ActiveFilter,
        messages::BoltImpactBreaker,
        queries::CollisionQueryBolt,
    },
    breaker::{filters::CollisionFilterBreaker, queries::CollisionQueryBreaker},
    effect::EffectivePiercing,
    shared::BREAKER_LAYER,
};

/// Precomputed breaker surface properties used for bolt reflection.
struct BreakerSurface {
    pos: Vec2,
    half_w: f32,
    half_h: f32,
    tilt_angle: f32,
    max_angle: f32,
    min_angle_from_horizontal: f32,
    entity: Entity,
}

impl BreakerSurface {
    /// Overwrites bolt velocity based on a normalized hit position on the breaker's top surface.
    fn reflect_top_hit(&self, impact_x: f32, bolt_velocity: &mut Velocity2D, base_speed: f32) {
        let fraction = hit_fraction(impact_x, self.pos.x, self.half_w);
        let base_angle = fraction * self.max_angle;
        let total_angle = base_angle + self.tilt_angle;
        let clamped_angle = total_angle.clamp(-self.max_angle, self.max_angle);
        let new_speed = bolt_velocity.speed().max(base_speed);
        bolt_velocity.0 = Vec2::new(
            new_speed * clamped_angle.sin(),
            new_speed * clamped_angle.cos(),
        );
        enforce_min_angle(&mut bolt_velocity.0, self.min_angle_from_horizontal);
    }

    /// Emits a [`BoltImpactBreaker`] message and resets piercing charges.
    fn emit_bump(
        &self,
        writer: &mut MessageWriter<BoltImpactBreaker>,
        bolt: Entity,
        piercing_remaining: &mut Option<Mut<'_, PiercingRemaining>>,
        effective_piercing: Option<&EffectivePiercing>,
    ) {
        writer.write(BoltImpactBreaker {
            bolt,
            breaker: self.entity,
        });
        if let (Some(pr), Some(ep)) = (piercing_remaining, effective_piercing) {
            pr.0 = ep.0;
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
    if let Some(qt_hit) = hits.first() {
        Some((qt_hit.normal, qt_hit.position))
    } else {
        let expanded = Aabb2D::new(breaker_pos, expanded_half);
        expanded
            .ray_intersect(bolt_pos, direction, max_dist)
            .map(|ray_hit| (ray_hit.normal, ray_hit.safe_position(bolt_pos, direction)))
    }
}

/// Detects bolt-breaker collisions via swept CCD and overwrites bolt direction.
///
/// Includes overlap resolution: if the breaker has moved into the bolt (e.g.,
/// bump pop), the bolt is pushed above the breaker and reflected if moving
/// downward. CCD alone cannot detect this case since it only sweeps bolt
/// movement.
pub(crate) fn bolt_breaker_collision(
    time: Res<Time<Fixed>>,
    quadtree: Res<CollisionQuadtree>,
    mut bolt_query: Query<CollisionQueryBolt, ActiveFilter>,
    breaker_query: Query<(Entity, CollisionQueryBreaker), CollisionFilterBreaker>,
    mut writer: MessageWriter<BoltImpactBreaker>,
) {
    let Ok((
        breaker_entity,
        (
            breaker_position,
            breaker_tilt,
            breaker_w,
            breaker_h,
            max_angle,
            min_angle,
            size_mult,
            breaker_entity_scale,
        ),
    )) = breaker_query.single()
    else {
        return;
    };

    let breaker_scale = breaker_entity_scale.map_or(1.0, |s| s.0);
    let surface = BreakerSurface {
        pos: breaker_position.0,
        half_w: breaker_w.half_width() * size_mult.map_or(1.0, |e| e.0) * breaker_scale,
        half_h: breaker_h.half_height() * breaker_scale,
        tilt_angle: breaker_tilt.angle,
        max_angle: max_angle.0,
        min_angle_from_horizontal: min_angle.0,
        entity: breaker_entity,
    };
    let dt = time.delta_secs();

    for (
        bolt_entity,
        mut bolt_position,
        mut bolt_velocity,
        base_speed,
        bolt_radius,
        mut piercing_remaining,
        effective_piercing,
        _damage_mult,
        bolt_entity_scale,
        _,
    ) in &mut bolt_query
    {
        let bolt_pos = bolt_position.0;
        let bolt_scale = bolt_entity_scale.map_or(1.0, |s| s.0);
        let r = bolt_radius.0 * bolt_scale;
        let expanded_half = Vec2::new(surface.half_w + r, surface.half_h + r);
        let above_y = surface.pos.y + surface.half_h + r;

        // Overlap resolution: breaker may have moved into the bolt (e.g., bump pop).
        // CCD can't detect this since it only sweeps bolt movement.
        if is_inside_aabb(bolt_pos, surface.pos, expanded_half) {
            bolt_position.0.y = above_y;
            if bolt_velocity.0.y <= 0.0 {
                surface.reflect_top_hit(bolt_pos.x, &mut bolt_velocity, base_speed.0);
                surface.emit_bump(
                    &mut writer,
                    bolt_entity,
                    &mut piercing_remaining,
                    effective_piercing,
                );
            }
            continue;
        }

        let speed = bolt_velocity.0.length();
        if speed < f32::EPSILON {
            continue;
        }

        let (direction, max_dist) = (bolt_velocity.0 / speed, speed * dt);

        let Some((normal, safe_pos)) = ccd_sweep_breaker(
            &quadtree,
            bolt_pos,
            direction,
            max_dist,
            r,
            surface.pos,
            expanded_half,
        ) else {
            continue;
        };

        // Only reflect downward-moving bolts; upward bolts pass through on all faces
        if bolt_velocity.0.y > 0.0 {
            continue;
        }

        // Determine if this is a side hit or top hit based on the normal
        if normal.x.abs() > normal.y.abs() {
            // Side hit — reflect X only, preserve Y velocity
            bolt_velocity.0.x = -bolt_velocity.0.x;
            bolt_position.0 = safe_pos;
        } else {
            // Top/bottom hit — move to impact point, reflect, push above breaker
            surface.reflect_top_hit(safe_pos.x, &mut bolt_velocity, base_speed.0);
            bolt_position.0.x = safe_pos.x;
            bolt_position.0.y = above_y;
        }

        surface.emit_bump(
            &mut writer,
            bolt_entity,
            &mut piercing_remaining,
            effective_piercing,
        );
    }
}
