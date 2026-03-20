//! Bolt-breaker collision detection and reflection via CCD.

use bevy::prelude::*;

use crate::{
    bolt::{components::BoltVelocity, filters::ActiveFilter},
    physics::{
        filters::CollisionFilterBreaker,
        messages::BoltHitBreaker,
        queries::{CollisionQueryBolt, CollisionQueryBreaker},
    },
    shared::math::{CCD_EPSILON, ray_vs_aabb},
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
    bolt_velocity: &mut BoltVelocity,
    tilt_angle: f32,
    max_angle: f32,
    base_speed: f32,
    min_angle_from_horizontal: f32,
) {
    let base_angle = hit_fraction * max_angle;
    let total_angle = base_angle + tilt_angle;
    let clamped_angle = total_angle.clamp(-max_angle, max_angle);
    let new_speed = bolt_velocity.speed().max(base_speed);
    bolt_velocity.value = Vec2::new(
        new_speed * clamped_angle.sin(),
        new_speed * clamped_angle.cos(),
    );
    bolt_velocity.enforce_min_angle(min_angle_from_horizontal);
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
        breaker_transform,
        breaker_tilt,
        breaker_w,
        breaker_h,
        max_angle,
        min_angle,
        tilt_boost,
        width_boost,
    )) = breaker_query.single()
    else {
        return;
    };

    let breaker_pos = breaker_transform.translation.truncate();
    let half_w = breaker_w.half_width() + width_boost.map_or(0.0, |b| b.0 / 2.0);
    let half_h = breaker_h.half_height();
    let effective_max_angle = max_angle.0 + tilt_boost.map_or(0.0, |b| b.0.to_radians());
    let dt = time.delta_secs();

    for (
        bolt_entity,
        mut bolt_transform,
        mut bolt_velocity,
        base_speed,
        bolt_radius,
        mut piercing_remaining,
        piercing,
        _damage_boost,
    ) in &mut bolt_query
    {
        let bolt_pos = bolt_transform.translation.truncate();
        let r = bolt_radius.0;
        let expanded_half = Vec2::new(half_w + r, half_h + r);
        let above_y = breaker_pos.y + half_h + r;

        // Overlap resolution: breaker may have moved into the bolt (e.g., bump pop).
        // CCD can't detect this since it only sweeps bolt movement.
        let inside = bolt_pos.x > breaker_pos.x - expanded_half.x
            && bolt_pos.x < breaker_pos.x + expanded_half.x
            && bolt_pos.y > breaker_pos.y - expanded_half.y
            && bolt_pos.y < breaker_pos.y + expanded_half.y;

        if inside {
            bolt_transform.translation.y = above_y;

            if bolt_velocity.value.y <= 0.0 {
                // Clamp hit X to the actual top-face width before computing the
                // reflection angle. The overlap check uses expanded_half (which
                // includes the bolt radius), so bolt_pos.x can sit slightly
                // outside [breaker_pos.x ± half_w]. Clamping here matches the
                // CCD path, which uses the exact ray-surface impact point.
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

        let speed = bolt_velocity.value.length();
        if speed < f32::EPSILON {
            continue;
        }

        let direction = bolt_velocity.value / speed;
        let max_dist = speed * dt;

        let Some(hit) = ray_vs_aabb(bolt_pos, direction, max_dist, breaker_pos, expanded_half)
        else {
            continue;
        };

        // Only reflect downward-moving bolts; upward bolts pass through on all faces
        if bolt_velocity.value.y > 0.0 {
            continue;
        }

        // Determine if this is a side hit or top hit based on the normal
        if hit.normal.x.abs() > hit.normal.y.abs() {
            // Side hit — reflect X only, preserve Y velocity
            bolt_velocity.value.x = -bolt_velocity.value.x;

            // Move to impact point
            let advance = (hit.distance - CCD_EPSILON).max(0.0);
            let new_pos = bolt_pos + direction * advance;
            bolt_transform.translation.x = new_pos.x;
            bolt_transform.translation.y = new_pos.y;
        } else {
            // Top/bottom hit — move to impact point, reflect, push above breaker
            let advance = (hit.distance - CCD_EPSILON).max(0.0);
            let impact_pos = bolt_pos + direction * advance;
            let hit_fraction = ((impact_pos.x - breaker_pos.x) / half_w).clamp(-1.0, 1.0);

            reflect_top_hit(
                hit_fraction,
                &mut bolt_velocity,
                breaker_tilt.angle,
                effective_max_angle,
                base_speed.0,
                min_angle.0,
            );

            bolt_transform.translation.x = impact_pos.x;
            bolt_transform.translation.y = above_y;
        }

        writer.write(BoltHitBreaker { bolt: bolt_entity });
        if let (Some(pr), Some(p)) = (&mut piercing_remaining, piercing) {
            pr.0 = p.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bolt::{
            BoltConfig,
            components::{Bolt, BoltBaseSpeed, BoltRadius, BoltVelocity},
        },
        breaker::{
            components::{
                Breaker, BreakerHeight, BreakerTilt, BreakerWidth, MaxReflectionAngle,
                MinAngleFromHorizontal,
            },
            resources::BreakerConfig,
        },
        chips::components::{Piercing, PiercingRemaining, TiltControlBoost, WidthBoost},
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitBreaker>()
            .add_systems(FixedUpdate, bolt_breaker_collision);
        app
    }

    fn default_breaker_width() -> BreakerWidth {
        BreakerWidth(120.0)
    }

    fn default_breaker_height() -> BreakerHeight {
        BreakerHeight(20.0)
    }

    fn default_bolt_radius() -> BoltRadius {
        BoltRadius(BoltConfig::default().radius)
    }

    fn default_max_reflection_angle() -> MaxReflectionAngle {
        MaxReflectionAngle(BreakerConfig::default().max_reflection_angle.to_radians())
    }

    fn default_min_angle() -> MinAngleFromHorizontal {
        MinAngleFromHorizontal(
            BreakerConfig::default()
                .min_angle_from_horizontal
                .to_radians(),
        )
    }

    fn bolt_param_bundle() -> (BoltBaseSpeed, BoltRadius) {
        let bolt_config = BoltConfig::default();
        (
            BoltBaseSpeed(bolt_config.base_speed),
            BoltRadius(bolt_config.radius),
        )
    }

    fn spawn_breaker_at(app: &mut App, x: f32, y: f32) {
        app.world_mut().spawn((
            Breaker,
            BreakerTilt::default(),
            default_breaker_width(),
            default_breaker_height(),
            default_max_reflection_angle(),
            default_min_angle(),
            Transform::from_xyz(x, y, 0.0),
        ));
    }

    /// Accumulates one fixed timestep of overstep, then runs one update.
    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn spawn_bolt(app: &mut App, x: f32, y: f32, vx: f32, vy: f32) -> Entity {
        app.world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(vx, vy),
                bolt_param_bundle(),
                Transform::from_xyz(x, y, 0.0),
            ))
            .id()
    }

    #[test]
    fn bolt_reflects_upward_on_center_hit() {
        let mut app = test_app();
        let hh = default_breaker_height();
        let y_pos = -250.0;
        spawn_breaker_at(&mut app, 0.0, y_pos);

        let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
        spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);
        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(vel.value.y > 0.0, "bolt should reflect upward");
    }

    #[test]
    fn left_hit_reflects_leftward() {
        let mut app = test_app();
        let hw = default_breaker_width();
        let hh = default_breaker_height();
        let y_pos = -250.0;
        spawn_breaker_at(&mut app, 0.0, y_pos);

        let hit_x = -hw.half_width() + 5.0;
        let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
        spawn_bolt(&mut app, hit_x, start_y, 0.0, -400.0);
        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(vel.value.x < 0.0, "left hit should angle bolt leftward");
        assert!(vel.value.y > 0.0, "bolt should still go upward");
    }

    #[test]
    fn right_hit_reflects_rightward() {
        let mut app = test_app();
        let hw = default_breaker_width();
        let hh = default_breaker_height();
        let y_pos = -250.0;
        spawn_breaker_at(&mut app, 0.0, y_pos);

        let hit_x = hw.half_width() - 5.0;
        let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
        spawn_bolt(&mut app, hit_x, start_y, 0.0, -400.0);
        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(vel.value.x > 0.0, "right hit should angle bolt rightward");
    }

    #[test]
    fn tilt_affects_reflection() {
        let mut app = test_app();
        let hh = default_breaker_height();
        let y_pos = -250.0;

        app.world_mut().spawn((
            Breaker,
            BreakerTilt {
                angle: 0.3,
                ease_start: 0.0,
                ease_target: 0.0,
            },
            default_breaker_width(),
            default_breaker_height(),
            default_max_reflection_angle(),
            default_min_angle(),
            Transform::from_xyz(0.0, y_pos, 0.0),
        ));

        let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
        spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);
        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.value.x > 0.0,
            "right tilt should push bolt rightward even on center hit"
        );
    }

    #[test]
    fn no_collision_when_bolt_above() {
        let mut app = test_app();
        let y_pos = -250.0;
        spawn_breaker_at(&mut app, 0.0, y_pos);

        spawn_bolt(&mut app, 0.0, 200.0, 0.0, -400.0);
        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.value.y < 0.0,
            "bolt should not be reflected when far above"
        );
    }

    #[test]
    fn upward_bolt_not_reflected() {
        let mut app = test_app();
        let hh = default_breaker_height();
        let y_pos = -250.0;
        spawn_breaker_at(&mut app, 0.0, y_pos);

        let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
        spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.value.y > 0.0,
            "upward-moving bolt should not be reflected"
        );
    }

    #[derive(Resource, Default)]
    struct HitBreakers(Vec<Entity>);

    fn collect_breaker_hits(
        mut reader: MessageReader<BoltHitBreaker>,
        mut hits: ResMut<HitBreakers>,
    ) {
        for msg in reader.read() {
            hits.0.push(msg.bolt);
        }
    }

    #[test]
    fn overlap_resolved_when_bolt_inside_breaker() {
        let mut app = test_app();
        let hh = default_breaker_height();
        let y_pos = -250.0;
        app.insert_resource(HitBreakers::default()).add_systems(
            FixedUpdate,
            collect_breaker_hits.after(bolt_breaker_collision),
        );

        let animated_y = y_pos + 10.0;
        spawn_breaker_at(&mut app, 0.0, animated_y);

        let bolt_entity = spawn_bolt(&mut app, 0.0, y_pos, 0.0, -400.0);
        tick(&mut app);

        let vel = app.world().get::<BoltVelocity>(bolt_entity).unwrap();
        assert!(
            vel.value.y > 0.0,
            "overlap should reflect bolt upward, got vy={:.1}",
            vel.value.y
        );

        let tf = app.world().get::<Transform>(bolt_entity).unwrap();
        let expected_y = animated_y + hh.half_height() + default_bolt_radius().0;
        assert!(
            (tf.translation.y - expected_y).abs() < 1.0,
            "bolt should be pushed above breaker, y={:.1} expected={expected_y:.1}",
            tf.translation.y
        );

        let hits = app.world().resource::<HitBreakers>();
        assert_eq!(
            hits.0.len(),
            1,
            "overlap with downward bolt should send BoltHitBreaker"
        );
    }

    #[test]
    fn upward_bolt_inside_breaker_pushed_out_no_message() {
        let mut app = test_app();
        let hh = default_breaker_height();
        let y_pos = -250.0;
        app.insert_resource(HitBreakers::default()).add_systems(
            FixedUpdate,
            collect_breaker_hits.after(bolt_breaker_collision),
        );

        let animated_y = y_pos + 10.0;
        spawn_breaker_at(&mut app, 0.0, animated_y);

        let bolt_entity = spawn_bolt(&mut app, 0.0, animated_y, 50.0, 400.0);
        tick(&mut app);

        let vel = app.world().get::<BoltVelocity>(bolt_entity).unwrap();
        assert!(
            vel.value.y > 0.0,
            "upward bolt should keep moving up, got vy={:.1}",
            vel.value.y
        );
        assert!(
            (vel.value.x - 50.0).abs() < f32::EPSILON,
            "velocity should be unchanged, got vx={:.1}",
            vel.value.x
        );

        let tf = app.world().get::<Transform>(bolt_entity).unwrap();
        let min_y = animated_y + hh.half_height() + default_bolt_radius().0;
        assert!(
            tf.translation.y >= min_y - 0.01,
            "bolt should be pushed above breaker, y={:.3} min={min_y:.3}",
            tf.translation.y
        );

        let hits = app.world().resource::<HitBreakers>();
        assert!(
            hits.0.is_empty(),
            "upward bolt overlap should NOT send BoltHitBreaker"
        );
    }

    // ---------------------------------------------------------------------------
    // Side-hit direction guard tests
    //
    // BoltConfig::default().radius = 8.0 (Rust default, not RON)
    // BreakerWidth(120.0).half_width() = 60.0
    // expanded_half.x = 60.0 + 8.0 = 68.0  →  left edge at x = -68.0
    //
    // At speed 360.5 and dt=1/64, max_dist ≈ 5.63.
    // Bolt at x = -70.0 is 2.0 outside the left edge.
    // CCD ray hits left face at t = 2.0 / (200/360.5) ≈ 3.6 < 5.63 ✓
    // ---------------------------------------------------------------------------

    /// Bolt moving up-right clips the breaker's left edge via CCD.
    /// The expected behavior (not yet implemented) is that upward side hits are
    /// NOT reflected — the guard should skip the reflection entirely.
    #[test]
    fn upward_bolt_side_hit_is_not_reflected() {
        let mut app = test_app();
        let breaker_y = -250.0;
        app.insert_resource(HitBreakers::default()).add_systems(
            FixedUpdate,
            collect_breaker_hits.after(bolt_breaker_collision),
        );
        spawn_breaker_at(&mut app, 0.0, breaker_y);

        // expanded_half.x = half_w(60) + r(8) = 68. Left edge at x = -68.
        // Bolt at x = -70 is 2.0 units outside. CCD ray (200, 300) hits at
        // t ≈ 3.6 which is within max_dist ≈ 5.63 (speed=360.5, dt=1/64).
        let bolt_entity = spawn_bolt(&mut app, -70.0, breaker_y, 200.0, 300.0);
        tick(&mut app);

        let vel = app.world().get::<BoltVelocity>(bolt_entity).unwrap();
        assert!(
            vel.value.x > 0.0,
            "upward side hit should NOT flip X velocity (guard should skip), got vx={:.1}",
            vel.value.x
        );
        assert!(
            vel.value.y > 0.0,
            "upward side hit should NOT flip Y velocity, got vy={:.1}",
            vel.value.y
        );

        let hits = app.world().resource::<HitBreakers>();
        assert!(
            hits.0.is_empty(),
            "upward side hit should NOT send BoltHitBreaker, got {} messages",
            hits.0.len()
        );
    }

    /// Bolt moving down-right clips the breaker's left edge via CCD.
    /// Downward side hits SHOULD still be reflected (existing behavior preserved).
    #[test]
    fn downward_bolt_side_hit_is_reflected() {
        let mut app = test_app();
        let breaker_y = -250.0;
        app.insert_resource(HitBreakers::default()).add_systems(
            FixedUpdate,
            collect_breaker_hits.after(bolt_breaker_collision),
        );
        spawn_breaker_at(&mut app, 0.0, breaker_y);

        // Same positioning as upward test, but with negative Y velocity.
        // expanded_half.x = 68. Bolt at x = -70 is 2.0 outside left edge.
        let bolt_entity = spawn_bolt(&mut app, -70.0, breaker_y, 200.0, -300.0);
        tick(&mut app);

        let vel = app.world().get::<BoltVelocity>(bolt_entity).unwrap();
        assert!(
            vel.value.x < 0.0,
            "downward side hit SHOULD flip X velocity, got vx={:.1}",
            vel.value.x
        );
    }

    #[test]
    fn multiple_bolts_each_reflect_off_breaker() {
        let mut app = test_app();
        let hh = default_breaker_height();
        let y_pos = -250.0;
        app.insert_resource(HitBreakers::default()).add_systems(
            FixedUpdate,
            collect_breaker_hits.after(bolt_breaker_collision),
        );
        spawn_breaker_at(&mut app, 0.0, y_pos);

        let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;

        let left_bolt = spawn_bolt(&mut app, -30.0, start_y, 0.0, -400.0);
        let right_bolt = spawn_bolt(&mut app, 30.0, start_y, 0.0, -400.0);

        tick(&mut app);

        let velocities: Vec<(Entity, Vec2)> = app
            .world_mut()
            .query::<(Entity, &BoltVelocity)>()
            .iter(app.world())
            .map(|(e, v)| (e, v.value))
            .collect();

        for (entity, vel) in &velocities {
            assert!(
                vel.y > 0.0,
                "bolt {entity:?} should reflect upward, got vy={:.1}",
                vel.y
            );
        }

        let hits = app.world().resource::<HitBreakers>();
        assert_eq!(hits.0.len(), 2, "both bolts should trigger hit messages");

        let left_vel = velocities.iter().find(|(e, _)| *e == left_bolt).unwrap().1;
        let right_vel = velocities.iter().find(|(e, _)| *e == right_bolt).unwrap().1;
        assert!(
            left_vel.x < 0.0,
            "left bolt should angle leftward, got vx={:.1}",
            left_vel.x
        );
        assert!(
            right_vel.x > 0.0,
            "right bolt should angle rightward, got vx={:.1}",
            right_vel.x
        );
    }

    // --- Chip effect reset tests ---

    #[test]
    fn breaker_hit_resets_piercing_remaining() {
        // Bolt with Piercing(3), PiercingRemaining(0). Bolt hits breaker.
        // PiercingRemaining should reset to Piercing.0 = 3.
        let mut app = test_app();
        let hh = default_breaker_height();
        let y_pos = -250.0;
        spawn_breaker_at(&mut app, 0.0, y_pos);

        // Place bolt just above breaker, moving downward toward it
        let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(0.0, -400.0),
                bolt_param_bundle(),
                Piercing(3),
                PiercingRemaining(0),
                Transform::from_xyz(0.0, start_y, 0.0),
            ))
            .id();

        tick(&mut app);

        // Verify breaker hit occurred (velocity.y > 0 after downward approach)
        let vel = app.world().get::<BoltVelocity>(bolt_entity).unwrap();
        assert!(
            vel.value.y > 0.0,
            "bolt should have reflected off breaker, got vy={}",
            vel.value.y
        );

        // PiercingRemaining should be reset to Piercing.0
        let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
        assert_eq!(
            pr.0, 3,
            "breaker hit should reset PiercingRemaining to Piercing.0 (3), got {}",
            pr.0
        );
    }

    #[test]
    fn piercing_remaining_without_piercing_does_not_reset_on_breaker_hit() {
        // Bolt has PiercingRemaining(5) but NO Piercing component.
        // The breaker hit reset guard uses `if let (Some(pr), Some(p))` — both must exist.
        // PiercingRemaining should stay at 5 after breaker hit.
        let mut app = test_app();
        let hh = default_breaker_height();
        let y_pos = -250.0;
        spawn_breaker_at(&mut app, 0.0, y_pos);

        let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(0.0, -400.0),
                bolt_param_bundle(),
                // PiercingRemaining WITHOUT Piercing
                PiercingRemaining(5),
                Transform::from_xyz(0.0, start_y, 0.0),
            ))
            .id();

        tick(&mut app);

        // Verify breaker hit occurred (velocity.y > 0 after downward approach)
        let vel = app.world().get::<BoltVelocity>(bolt_entity).unwrap();
        assert!(
            vel.value.y > 0.0,
            "bolt should have reflected off breaker, got vy={}",
            vel.value.y
        );

        // PiercingRemaining should be unchanged — reset requires BOTH Piercing and PiercingRemaining
        let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
        assert_eq!(
            pr.0, 5,
            "PiercingRemaining without Piercing should not be reset on breaker hit, got {}",
            pr.0
        );
    }

    // --- TiltControlBoost tests ---

    #[test]
    fn tilt_control_boost_widens_effective_max_reflection_angle() {
        // Breaker with TiltControlBoost(15.0 degrees).
        // Bolt hitting far from center (x near the edge).
        // With TiltControlBoost, the effective max_angle is widened.
        // Reflected angle should be wider (larger |vx / vy| ratio) than without boost.
        let mut app = test_app();
        let hh = default_breaker_height();
        let hw = default_breaker_width();
        let y_pos = -250.0;

        // Spawn breaker WITH TiltControlBoost
        app.world_mut().spawn((
            Breaker,
            BreakerTilt::default(),
            BreakerWidth(120.0),
            default_breaker_height(),
            default_max_reflection_angle(),
            default_min_angle(),
            TiltControlBoost(15.0),
            Transform::from_xyz(0.0, y_pos, 0.0),
        ));

        // Hit near the right edge of the breaker — should produce maximum angle
        let hit_x = hw.half_width() - 2.0;
        let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
        let bolt_entity = spawn_bolt(&mut app, hit_x, start_y, 0.0, -400.0);

        tick(&mut app);

        let vel_with_boost = app.world().get::<BoltVelocity>(bolt_entity).unwrap().value;
        // Reflected angle from vertical: atan2(|vx|, vy)
        let angle_with_boost = vel_with_boost.x.abs().atan2(vel_with_boost.y);

        // Now test without boost for comparison — spawn fresh app
        let mut app_no_boost = test_app();
        app_no_boost.world_mut().spawn((
            Breaker,
            BreakerTilt::default(),
            default_breaker_width(),
            default_breaker_height(),
            default_max_reflection_angle(),
            default_min_angle(),
            Transform::from_xyz(0.0, y_pos, 0.0),
        ));
        let bolt_no_boost = spawn_bolt(&mut app_no_boost, hit_x, start_y, 0.0, -400.0);
        tick(&mut app_no_boost);

        let vel_no_boost = app_no_boost
            .world()
            .get::<BoltVelocity>(bolt_no_boost)
            .unwrap()
            .value;
        let angle_no_boost = vel_no_boost.x.abs().atan2(vel_no_boost.y);

        assert!(
            angle_with_boost > angle_no_boost,
            "TiltControlBoost should widen reflection angle: boost={angle_with_boost:.3} rad, no-boost={angle_no_boost:.3} rad"
        );
    }

    // --- WidthBoost tests ---

    #[test]
    fn width_boost_widens_effective_breaker_collision_width() {
        // Breaker with WidthBoost(40.0) — half_w += 20.
        // Base BreakerWidth(120.0) → half_w = 60.0.
        // Boosted: half_w = 60.0 + 20.0 = 80.0.
        // Bolt at x=75.0 (outside base 60 but inside boosted 80) should reflect.
        let mut app = test_app();
        let hh = default_breaker_height();
        let y_pos = -250.0;

        // Spawn breaker with WidthBoost(40.0) — adds 20 to each half
        app.world_mut().spawn((
            Breaker,
            BreakerTilt::default(),
            default_breaker_width(),
            default_breaker_height(),
            default_max_reflection_angle(),
            default_min_angle(),
            WidthBoost(40.0),
            Transform::from_xyz(0.0, y_pos, 0.0),
        ));

        // Bolt at x=75.0 — outside base half_w(60) but inside boosted half_w(80)
        let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
        let bolt_entity = spawn_bolt(&mut app, 75.0, start_y, 0.0, -400.0);

        tick(&mut app);

        let vel = app.world().get::<BoltVelocity>(bolt_entity).unwrap();
        assert!(
            vel.value.y > 0.0,
            "bolt at x=75.0 (inside boosted width) should reflect upward, got vy={}",
            vel.value.y
        );
    }
}
