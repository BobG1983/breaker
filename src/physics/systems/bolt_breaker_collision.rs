//! Bolt-breaker collision detection and reflection via CCD.

use bevy::prelude::*;

use crate::{
    bolt::{
        BoltConfig,
        components::{Bolt, BoltVelocity},
        filters::ActiveBoltFilter,
    },
    breaker::{
        BreakerConfig,
        components::{Breaker, BreakerTilt},
    },
    physics::{
        ccd::{CCD_EPSILON, ray_vs_aabb},
        messages::BoltHitBreaker,
        resources::PhysicsConfig,
    },
};

/// Query filter for breaker data.
type BreakerQueryFilter = (With<Breaker>, Without<Bolt>);

/// Detects bolt-breaker collisions via swept CCD and overwrites bolt direction.
///
/// Includes overlap resolution: if the breaker has moved into the bolt (e.g.,
/// bump pop), the bolt is pushed above the breaker and reflected if moving
/// downward. CCD alone cannot detect this case since it only sweeps bolt
/// movement.
pub fn bolt_breaker_collision(
    time: Res<Time<Fixed>>,
    bolt_config: Res<BoltConfig>,
    breaker_config: Res<BreakerConfig>,
    physics_config: Res<PhysicsConfig>,
    mut bolt_query: Query<(Entity, &mut Transform, &mut BoltVelocity), ActiveBoltFilter>,
    breaker_query: Query<(&Transform, &BreakerTilt), BreakerQueryFilter>,
    mut writer: MessageWriter<BoltHitBreaker>,
) {
    let Ok((breaker_transform, breaker_tilt)) = breaker_query.single() else {
        return;
    };

    let breaker_pos = breaker_transform.translation.truncate();
    let dt = time.delta_secs();
    let expanded_half = Vec2::new(
        breaker_config.half_width + bolt_config.radius,
        breaker_config.half_height + bolt_config.radius,
    );
    let above_y = breaker_pos.y + breaker_config.half_height + bolt_config.radius;

    // Overwrites bolt velocity based on hit position on the breaker surface.
    // Direction is entirely overwritten (no incoming angle carryover).
    let reflect_top_hit = |hit_x: f32, bolt_velocity: &mut BoltVelocity| {
        let hit_fraction = ((hit_x - breaker_pos.x) / breaker_config.half_width).clamp(-1.0, 1.0);
        let base_angle = hit_fraction * physics_config.max_reflection_angle;
        let total_angle = base_angle + breaker_tilt.angle;
        let clamped_angle = total_angle.clamp(
            -physics_config.max_reflection_angle,
            physics_config.max_reflection_angle,
        );
        let new_speed = bolt_velocity.speed().max(bolt_config.base_speed);
        bolt_velocity.value = Vec2::new(
            new_speed * clamped_angle.sin(),
            new_speed * clamped_angle.cos(),
        );
        bolt_velocity.enforce_min_angle(bolt_config.min_angle_from_horizontal);
    };

    for (bolt_entity, mut bolt_transform, mut bolt_velocity) in &mut bolt_query {
        let bolt_pos = bolt_transform.translation.truncate();

        // Overlap resolution: breaker may have moved into the bolt (e.g., bump pop).
        // CCD can't detect this since it only sweeps bolt movement.
        let inside = bolt_pos.x > breaker_pos.x - expanded_half.x
            && bolt_pos.x < breaker_pos.x + expanded_half.x
            && bolt_pos.y > breaker_pos.y - expanded_half.y
            && bolt_pos.y < breaker_pos.y + expanded_half.y;

        if inside {
            bolt_transform.translation.y = above_y;

            if bolt_velocity.value.y <= 0.0 {
                reflect_top_hit(bolt_pos.x, &mut bolt_velocity);
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

            reflect_top_hit(impact_pos.x, &mut bolt_velocity);

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
        bolt::components::{Bolt, BoltVelocity},
        breaker::components::{Breaker, BreakerTilt},
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BoltConfig>();
        app.init_resource::<BreakerConfig>();
        app.init_resource::<PhysicsConfig>();
        app.add_message::<BoltHitBreaker>();
        app.add_systems(Update, bolt_breaker_collision);
        app
    }

    fn spawn_breaker_at(app: &mut App, x: f32, y: f32) {
        app.world_mut().spawn((
            Breaker,
            BreakerTilt::default(),
            Transform::from_xyz(x, y, 0.0),
        ));
    }

    /// Advances `Time<Fixed>` by one default timestep, then runs one update.
    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(timestep);
        app.update();
    }

    #[test]
    fn bolt_reflects_upward_on_center_hit() {
        let mut app = test_app();
        let config = BreakerConfig::default();
        let bolt_config = BoltConfig::default();
        spawn_breaker_at(&mut app, 0.0, config.y_position);

        // Place bolt above breaker, moving downward — close enough to hit within one timestep
        let start_y = config.y_position + config.half_height + bolt_config.radius + 3.0;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, -400.0),
            Transform::from_xyz(0.0, start_y, 0.0),
        ));
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
        let bc = BreakerConfig::default();
        let bolt_config = BoltConfig::default();
        spawn_breaker_at(&mut app, 0.0, bc.y_position);

        // Hit left edge of breaker
        let hit_x = -bc.half_width + 5.0;
        let start_y = bc.y_position + bc.half_height + bolt_config.radius + 3.0;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, -400.0),
            Transform::from_xyz(hit_x, start_y, 0.0),
        ));
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
        let bc = BreakerConfig::default();
        let bolt_config = BoltConfig::default();
        spawn_breaker_at(&mut app, 0.0, bc.y_position);

        // Hit right edge of breaker
        let hit_x = bc.half_width - 5.0;
        let start_y = bc.y_position + bc.half_height + bolt_config.radius + 3.0;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, -400.0),
            Transform::from_xyz(hit_x, start_y, 0.0),
        ));
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
        let bc = BreakerConfig::default();
        let bolt_config = BoltConfig::default();

        // Breaker tilted right
        app.world_mut().spawn((
            Breaker,
            BreakerTilt {
                angle: 0.3,
                settle_start_angle: 0.0,
            },
            Transform::from_xyz(0.0, bc.y_position, 0.0),
        ));

        // Center hit
        let start_y = bc.y_position + bc.half_height + bolt_config.radius + 3.0;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, -400.0),
            Transform::from_xyz(0.0, start_y, 0.0),
        ));
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
        let bc = BreakerConfig::default();
        spawn_breaker_at(&mut app, 0.0, bc.y_position);

        // Bolt is far above breaker
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, -400.0),
            Transform::from_xyz(0.0, 200.0, 0.0),
        ));
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
        let bc = BreakerConfig::default();
        let bolt_config = BoltConfig::default();
        spawn_breaker_at(&mut app, 0.0, bc.y_position);

        // Bolt moving upward through breaker — should not double-bounce
        let start_y = bc.y_position + bc.half_height + bolt_config.radius + 3.0;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 400.0),
            Transform::from_xyz(0.0, start_y, 0.0),
        ));
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

    /// Collects `BoltHitBreaker` messages into a resource for multi-bolt test assertions.
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
        let bc = BreakerConfig::default();
        let bolt_config = BoltConfig::default();
        app.insert_resource(HitBreakers::default());
        app.add_systems(Update, collect_breaker_hits.after(bolt_breaker_collision));

        // Breaker popped up by 10 units (simulating bump visual)
        let animated_y = bc.y_position + 10.0;
        spawn_breaker_at(&mut app, 0.0, animated_y);

        // Bolt inside the breaker's expanded AABB, moving downward.
        // It's at the original breaker position — now inside the shifted breaker.
        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(0.0, -400.0),
                Transform::from_xyz(0.0, bc.y_position, 0.0),
            ))
            .id();
        tick(&mut app);

        let vel = app.world().get::<BoltVelocity>(bolt_entity).unwrap();
        assert!(
            vel.value.y > 0.0,
            "overlap should reflect bolt upward, got vy={:.1}",
            vel.value.y
        );

        let tf = app.world().get::<Transform>(bolt_entity).unwrap();
        let expected_y = animated_y + bc.half_height + bolt_config.radius;
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
        let bc = BreakerConfig::default();
        let bolt_config = BoltConfig::default();
        app.insert_resource(HitBreakers::default());
        app.add_systems(Update, collect_breaker_hits.after(bolt_breaker_collision));

        // Breaker popped up into the bolt
        let animated_y = bc.y_position + 10.0;
        spawn_breaker_at(&mut app, 0.0, animated_y);

        // Bolt inside AABB but moving upward (already reflected)
        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(50.0, 400.0),
                Transform::from_xyz(0.0, animated_y, 0.0),
            ))
            .id();
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
        let min_y = animated_y + bc.half_height + bolt_config.radius;
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
        let bc = BreakerConfig::default();
        let bolt_config = BoltConfig::default();
        app.insert_resource(HitBreakers::default());
        app.add_systems(Update, collect_breaker_hits.after(bolt_breaker_collision));
        spawn_breaker_at(&mut app, 0.0, bc.y_position);

        let start_y = bc.y_position + bc.half_height + bolt_config.radius + 3.0;

        // Left bolt — hit left side of breaker
        let left_bolt = app
            .world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(0.0, -400.0),
                Transform::from_xyz(-30.0, start_y, 0.0),
            ))
            .id();

        // Right bolt — hit right side of breaker
        let right_bolt = app
            .world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(0.0, -400.0),
                Transform::from_xyz(30.0, start_y, 0.0),
            ))
            .id();

        tick(&mut app);

        // Both should be reflected upward
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

        // Two messages sent
        let hits = app.world().resource::<HitBreakers>();
        assert_eq!(hits.0.len(), 2, "both bolts should trigger hit messages");

        // Left bolt angles left, right bolt angles right
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
