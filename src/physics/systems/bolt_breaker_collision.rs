//! Bolt-breaker collision detection and reflection via CCD.

use bevy::prelude::*;

use crate::{
    bolt::{components::BoltVelocity, filters::ActiveBoltFilter},
    breaker::components::{
        BreakerHeight, BreakerTilt, BreakerWidth, MaxReflectionAngle, MinAngleFromHorizontal,
    },
    physics::{
        ccd::{CCD_EPSILON, ray_vs_aabb},
        filters::BreakerCollisionFilter,
        messages::BoltHitBreaker,
        queries::BoltPhysicsQuery,
    },
};

/// Detects bolt-breaker collisions via swept CCD and overwrites bolt direction.
///
/// Includes overlap resolution: if the breaker has moved into the bolt (e.g.,
/// bump pop), the bolt is pushed above the breaker and reflected if moving
/// downward. CCD alone cannot detect this case since it only sweeps bolt
/// movement.
pub fn bolt_breaker_collision(
    time: Res<Time<Fixed>>,
    mut bolt_query: Query<BoltPhysicsQuery, ActiveBoltFilter>,
    breaker_query: Query<
        (
            &Transform,
            &BreakerTilt,
            &BreakerWidth,
            &BreakerHeight,
            &MaxReflectionAngle,
            &MinAngleFromHorizontal,
        ),
        BreakerCollisionFilter,
    >,
    mut writer: MessageWriter<BoltHitBreaker>,
) {
    let Ok((breaker_transform, breaker_tilt, breaker_w, breaker_h, max_angle, min_angle)) =
        breaker_query.single()
    else {
        return;
    };

    let breaker_pos = breaker_transform.translation.truncate();
    let half_w = breaker_w.half_width();
    let half_h = breaker_h.half_height();
    let dt = time.delta_secs();

    for (bolt_entity, mut bolt_transform, mut bolt_velocity, base_speed, bolt_radius) in
        &mut bolt_query
    {
        let bolt_pos = bolt_transform.translation.truncate();
        let r = bolt_radius.0;
        let expanded_half = Vec2::new(half_w + r, half_h + r);
        let above_y = breaker_pos.y + half_h + r;

        // Overwrites bolt velocity based on hit position on the breaker surface.
        let reflect_top_hit = |hit_x: f32,
                               bolt_velocity: &mut BoltVelocity,
                               b_speed: f32,
                               m_angle: f32,
                               m_min: f32| {
            let hit_fraction = ((hit_x - breaker_pos.x) / half_w).clamp(-1.0, 1.0);
            let base_angle = hit_fraction * m_angle;
            let total_angle = base_angle + breaker_tilt.angle;
            let clamped_angle = total_angle.clamp(-m_angle, m_angle);
            let new_speed = bolt_velocity.speed().max(b_speed);
            bolt_velocity.value = Vec2::new(
                new_speed * clamped_angle.sin(),
                new_speed * clamped_angle.cos(),
            );
            bolt_velocity.enforce_min_angle(m_min);
        };

        // Overlap resolution: breaker may have moved into the bolt (e.g., bump pop).
        // CCD can't detect this since it only sweeps bolt movement.
        let inside = bolt_pos.x > breaker_pos.x - expanded_half.x
            && bolt_pos.x < breaker_pos.x + expanded_half.x
            && bolt_pos.y > breaker_pos.y - expanded_half.y
            && bolt_pos.y < breaker_pos.y + expanded_half.y;

        if inside {
            bolt_transform.translation.y = above_y;

            if bolt_velocity.value.y <= 0.0 {
                reflect_top_hit(
                    bolt_pos.x,
                    &mut bolt_velocity,
                    base_speed.0,
                    max_angle.0,
                    min_angle.0,
                );
                writer.write(BoltHitBreaker { bolt: bolt_entity });
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
            // Top/bottom hit — only reflect if bolt is moving downward
            if bolt_velocity.value.y > 0.0 {
                continue;
            }

            // Move to impact point
            let advance = (hit.distance - CCD_EPSILON).max(0.0);
            let impact_pos = bolt_pos + direction * advance;

            reflect_top_hit(
                impact_pos.x,
                &mut bolt_velocity,
                base_speed.0,
                max_angle.0,
                min_angle.0,
            );

            // Push bolt above breaker to prevent re-collision
            bolt_transform.translation.x = impact_pos.x;
            bolt_transform.translation.y = above_y;
        }

        writer.write(BoltHitBreaker { bolt: bolt_entity });
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
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<BoltHitBreaker>();
        app.add_systems(FixedUpdate, bolt_breaker_collision);
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
        MaxReflectionAngle(BreakerConfig::default().max_reflection_angle)
    }

    fn default_min_angle() -> MinAngleFromHorizontal {
        MinAngleFromHorizontal(BreakerConfig::default().min_angle_from_horizontal)
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
                settle_start_angle: 0.0,
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
        app.insert_resource(HitBreakers::default());
        app.add_systems(
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
        app.insert_resource(HitBreakers::default());
        app.add_systems(
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
            tf.translation.y >= min_y - 1.0,
            "bolt should be pushed above breaker, y={:.1} min={min_y:.1}",
            tf.translation.y
        );

        let hits = app.world().resource::<HitBreakers>();
        assert!(
            hits.0.is_empty(),
            "upward bolt overlap should NOT send BoltHitBreaker"
        );
    }

    #[test]
    fn multiple_bolts_each_reflect_off_breaker() {
        let mut app = test_app();
        let hh = default_breaker_height();
        let y_pos = -250.0;
        app.insert_resource(HitBreakers::default());
        app.add_systems(
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
}
