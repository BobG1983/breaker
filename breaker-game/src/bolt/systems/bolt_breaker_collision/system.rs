//! Bolt-breaker collision detection and reflection via CCD.

use bevy::prelude::*;
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::{
        components::enforce_min_angle, filters::ActiveFilter, messages::BoltHitBreaker,
        queries::CollisionQueryBolt,
    },
    breaker::{filters::CollisionFilterBreaker, queries::CollisionQueryBreaker},
};

/// Overwrites bolt velocity based on a normalized hit position on the breaker's top surface.
///
/// - `hit_fraction`: hit position in `[-1.0, 1.0]` — left edge = -1, right edge = +1
/// - `bolt_velocity`: mutable bolt velocity to overwrite
/// - `tilt_angle`: current breaker tilt in radians (added to base angle)
/// - `max_angle`: maximum reflection angle in radians (used for clamping and scaling)
/// - `base_speed`: minimum bolt speed to enforce after reflection
/// - `min_angle_from_horizontal`: minimum angle from horizontal enforced on the result
fn reflect_top_hit(
    hit_fraction: f32,
    bolt_velocity: &mut Velocity2D,
    tilt_angle: f32,
    max_angle: f32,
    base_speed: f32,
    min_angle_from_horizontal: f32,
) {
    let base_angle = hit_fraction * max_angle;
    let total_angle = base_angle + tilt_angle;
    let clamped_angle = total_angle.clamp(-max_angle, max_angle);
    let new_speed = bolt_velocity.speed().max(base_speed);
    bolt_velocity.0 = Vec2::new(
        new_speed * clamped_angle.sin(),
        new_speed * clamped_angle.cos(),
    );
    enforce_min_angle(&mut bolt_velocity.0, min_angle_from_horizontal);
}

/// Returns `true` when `point` lies inside the axis-aligned box centred on
/// `center` with half-extents `half`.
fn is_inside_aabb(point: Vec2, center: Vec2, half: Vec2) -> bool {
    point.x > center.x - half.x
        && point.x < center.x + half.x
        && point.y > center.y - half.y
        && point.y < center.y + half.y
}

/// Detects bolt-breaker collisions via swept CCD and overwrites bolt direction.
///
/// Includes overlap resolution: if the breaker has moved into the bolt (e.g.,
/// bump pop), the bolt is pushed above the breaker and reflected if moving
/// downward. CCD alone cannot detect this case since it only sweeps bolt
/// movement.
pub(crate) fn bolt_breaker_collision(
    time: Res<Time<Fixed>>,
    mut bolt_query: Query<CollisionQueryBolt, ActiveFilter>,
    breaker_query: Query<CollisionQueryBreaker, CollisionFilterBreaker>,
    mut writer: MessageWriter<BoltHitBreaker>,
) {
    let Ok((
        breaker_position,
        breaker_tilt,
        breaker_w,
        breaker_h,
        max_angle,
        min_angle,
        tilt_boost,
        width_boost,
        breaker_entity_scale,
    )) = breaker_query.single()
    else {
        return;
    };

    let breaker_pos = breaker_position.0;
    let breaker_scale = breaker_entity_scale.map_or(1.0, |s| s.0);
    let half_w = (breaker_w.half_width() + width_boost.map_or(0.0, |b| b.0 / 2.0)) * breaker_scale;
    let half_h = breaker_h.half_height() * breaker_scale;
    let effective_max_angle = max_angle.0 + tilt_boost.map_or(0.0, |b| b.0.to_radians());
    let dt = time.delta_secs();

    for (
        bolt_entity,
        mut bolt_position,
        mut bolt_velocity,
        base_speed,
        bolt_radius,
        mut piercing_remaining,
        piercing,
        _damage_boost,
        bolt_entity_scale,
        _,
    ) in &mut bolt_query
    {
        let bolt_pos = bolt_position.0;
        let bolt_scale = bolt_entity_scale.map_or(1.0, |s| s.0);
        let r = bolt_radius.0 * bolt_scale;
        let expanded_half = Vec2::new(half_w + r, half_h + r);
        let above_y = breaker_pos.y + half_h + r;

        // Overlap resolution: breaker may have moved into the bolt (e.g., bump pop).
        // CCD can't detect this since it only sweeps bolt movement.
        if is_inside_aabb(bolt_pos, breaker_pos, expanded_half) {
            bolt_position.0.y = above_y;
            if bolt_velocity.0.y <= 0.0 {
                let hit_x = bolt_pos
                    .x
                    .clamp(breaker_pos.x - half_w, breaker_pos.x + half_w);
                let hit_fraction = ((hit_x - breaker_pos.x) / half_w).clamp(-1.0, 1.0);
                reflect_top_hit(
                    hit_fraction,
                    &mut bolt_velocity,
                    breaker_tilt.angle,
                    effective_max_angle,
                    base_speed.0,
                    min_angle.0,
                );
                writer.write(BoltHitBreaker { bolt: bolt_entity });
                if let (Some(pr), Some(p)) = (&mut piercing_remaining, piercing) {
                    pr.0 = p.0;
                }
            }
            continue;
        }

        let speed = bolt_velocity.0.length();
        if speed < f32::EPSILON {
            continue;
        }

        let (direction, max_dist) = (bolt_velocity.0 / speed, speed * dt);

        let expanded = Aabb2D::new(breaker_pos, expanded_half);
        let Some(hit) = expanded.ray_intersect(bolt_pos, direction, max_dist) else {
            continue;
        };

        // Only reflect downward-moving bolts; upward bolts pass through on all faces
        if bolt_velocity.0.y > 0.0 {
            continue;
        }

        // Determine if this is a side hit or top hit based on the normal
        if hit.normal.x.abs() > hit.normal.y.abs() {
            // Side hit — reflect X only, preserve Y velocity
            bolt_velocity.0.x = -bolt_velocity.0.x;

            bolt_position.0 = hit.safe_position(bolt_pos, direction);
        } else {
            // Top/bottom hit — move to impact point, reflect, push above breaker
            let impact_pos = hit.safe_position(bolt_pos, direction);
            let hit_fraction = ((impact_pos.x - breaker_pos.x) / half_w).clamp(-1.0, 1.0);

            reflect_top_hit(
                hit_fraction,
                &mut bolt_velocity,
                breaker_tilt.angle,
                effective_max_angle,
                base_speed.0,
                min_angle.0,
            );

            bolt_position.0.x = impact_pos.x;
            bolt_position.0.y = above_y;
        }

        writer.write(BoltHitBreaker { bolt: bolt_entity });
        if let (Some(pr), Some(p)) = (&mut piercing_remaining, piercing) {
            pr.0 = p.0;
        }
    }
}
