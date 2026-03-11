//! Bolt-breaker collision detection and reflection.

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
    physics::{messages::BoltHitBreaker, resources::PhysicsConfig},
};

/// Query filter for breaker data.
type BreakerQueryFilter = (With<Breaker>, Without<Bolt>);

/// Detects bolt-breaker collisions and overwrites bolt direction.
///
/// The reflection model:
/// - Direction is entirely overwritten (no incoming angle carryover)
/// - Angle determined by hit position on breaker (left = left angle, right = right angle)
/// - Breaker tilt modifies the effective surface angle
/// - Minimum angle from horizontal is enforced
pub fn bolt_breaker_collision(
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

    let breaker_pos = breaker_transform.translation;
    let breaker_half_width = breaker_config.half_width;
    let breaker_half_height = breaker_config.half_height;

    for (bolt_entity, mut bolt_transform, mut bolt_velocity) in &mut bolt_query {
        let bolt_pos = bolt_transform.translation;

        // AABB overlap test
        let overlap_x =
            (bolt_pos.x - breaker_pos.x).abs() < breaker_half_width + bolt_config.radius;
        let overlap_y =
            (bolt_pos.y - breaker_pos.y).abs() < breaker_half_height + bolt_config.radius;

        if !overlap_x || !overlap_y {
            continue;
        }

        // Determine side hit vs top hit based on overlap penetration depth
        let overlap_depth_x =
            breaker_half_width + bolt_config.radius - (bolt_pos.x - breaker_pos.x).abs();
        let overlap_depth_y =
            breaker_half_height + bolt_config.radius - (bolt_pos.y - breaker_pos.y).abs();

        if overlap_depth_x < overlap_depth_y {
            // Side hit — reflect X only, preserve Y velocity
            bolt_velocity.value.x = -bolt_velocity.value.x;
            let sign = (bolt_pos.x - breaker_pos.x).signum();
            bolt_transform.translation.x =
                sign.mul_add(breaker_half_width + bolt_config.radius, breaker_pos.x);
        } else {
            // Top hit — only reflect if bolt is moving downward (prevents double-bouncing)
            if bolt_velocity.value.y > 0.0 {
                continue;
            }

            // Calculate hit position as fraction of breaker width (-1.0 to 1.0)
            let hit_fraction = ((bolt_pos.x - breaker_pos.x) / breaker_half_width).clamp(-1.0, 1.0);

            // Base angle from hit position (center = straight up, edges = angled)
            let base_angle = hit_fraction * physics_config.max_reflection_angle;

            // Add breaker tilt influence
            let total_angle = base_angle + breaker_tilt.angle;

            // Clamp to max reflection angle
            let clamped_angle = total_angle.clamp(
                -physics_config.max_reflection_angle,
                physics_config.max_reflection_angle,
            );

            // Overwrite bolt direction entirely
            let speed = bolt_velocity.speed().max(bolt_config.base_speed);
            let new_vel_x = speed * clamped_angle.sin();
            let new_vel_y = speed * clamped_angle.cos(); // Always positive (upward)

            bolt_velocity.value = Vec2::new(new_vel_x, new_vel_y);

            // Enforce minimum angle from horizontal
            bolt_velocity.enforce_min_angle(bolt_config.min_angle_from_horizontal);

            // Push bolt above breaker to prevent re-collision
            bolt_transform.translation.y = breaker_pos.y + breaker_half_height + bolt_config.radius;
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
            BreakerTilt { angle: 0.0 },
            Transform::from_xyz(x, y, 0.0),
        ));
    }

    #[test]
    fn bolt_reflects_upward_on_center_hit() {
        let mut app = test_app();
        let config = BreakerConfig::default();
        spawn_breaker_at(&mut app, 0.0, config.y_position);

        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, -400.0),
            Transform::from_xyz(0.0, config.y_position + 5.0, 0.0),
        ));
        app.update();

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
        spawn_breaker_at(&mut app, 0.0, bc.y_position);

        // Hit left edge of breaker
        let hit_x = -bc.half_width + 5.0;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, -400.0),
            Transform::from_xyz(hit_x, bc.y_position + 5.0, 0.0),
        ));
        app.update();

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
        spawn_breaker_at(&mut app, 0.0, bc.y_position);

        // Hit right edge of breaker
        let hit_x = bc.half_width - 5.0;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, -400.0),
            Transform::from_xyz(hit_x, bc.y_position + 5.0, 0.0),
        ));
        app.update();

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

        // Breaker tilted right
        app.world_mut().spawn((
            Breaker,
            BreakerTilt { angle: 0.3 },
            Transform::from_xyz(0.0, bc.y_position, 0.0),
        ));

        // Center hit
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, -400.0),
            Transform::from_xyz(0.0, bc.y_position + 5.0, 0.0),
        ));
        app.update();

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
    fn side_hit_reflects_x_not_teleport_above() {
        let mut app = test_app();
        let bc = BreakerConfig::default();
        let bolt_config = BoltConfig::default();
        spawn_breaker_at(&mut app, 0.0, bc.y_position);

        // Bolt at breaker's right edge, same Y as breaker center, moving left+slightly down
        let bolt_x = bc.half_width - 2.0;
        let bolt_y = bc.y_position;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(-200.0, -50.0),
            Transform::from_xyz(bolt_x, bolt_y, 0.0),
        ));
        app.update();

        let (vel, transform) = app
            .world_mut()
            .query::<(&BoltVelocity, &Transform)>()
            .iter(app.world())
            .next()
            .unwrap();

        // Side hit: vx should be reflected (now positive)
        assert!(
            vel.value.x > 0.0,
            "side hit should reflect vx to positive, got {}",
            vel.value.x
        );
        // Y should stay near original, NOT teleported above breaker
        let max_y = bc.y_position + bc.half_height + bolt_config.radius + 1.0;
        assert!(
            transform.translation.y < max_y,
            "bolt Y {:.1} should stay near original {bolt_y:.1}, not teleport above breaker",
            transform.translation.y
        );
        // vy should remain negative (not overwritten upward)
        assert!(
            vel.value.y < 0.0,
            "side hit should preserve negative vy, got {}",
            vel.value.y
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
    fn multiple_bolts_each_reflect_off_breaker() {
        let mut app = test_app();
        let bc = BreakerConfig::default();
        app.insert_resource(HitBreakers::default());
        app.add_systems(Update, collect_breaker_hits.after(bolt_breaker_collision));
        spawn_breaker_at(&mut app, 0.0, bc.y_position);

        // Left bolt — hit left side of breaker
        let left_bolt = app
            .world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(0.0, -400.0),
                Transform::from_xyz(-30.0, bc.y_position + 5.0, 0.0),
            ))
            .id();

        // Right bolt — hit right side of breaker
        let right_bolt = app
            .world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(0.0, -400.0),
                Transform::from_xyz(30.0, bc.y_position + 5.0, 0.0),
            ))
            .id();

        app.update();

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
        app.update();

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
        spawn_breaker_at(&mut app, 0.0, bc.y_position);

        // Bolt moving upward through breaker — should not double-bounce
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 400.0),
            Transform::from_xyz(0.0, bc.y_position + 5.0, 0.0),
        ));
        app.update();

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
}
