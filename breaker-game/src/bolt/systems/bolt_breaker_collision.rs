//! Bolt-breaker collision detection and reflection via CCD.

use bevy::prelude::*;

use crate::{
    bolt::{
        components::BoltVelocity, filters::ActiveFilter, messages::BoltHitBreaker,
        queries::CollisionQueryBolt,
    },
    breaker::{filters::CollisionFilterBreaker, queries::CollisionQueryBreaker},
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
    ) in &mut bolt_query
    {
        let bolt_pos = bolt_position.0;
        let bolt_scale = bolt_entity_scale.map_or(1.0, |s| s.0);
        let r = bolt_radius.0 * bolt_scale;
        let expanded_half = Vec2::new(half_w + r, half_h + r);
        let above_y = breaker_pos.y + half_h + r;

        // Overlap resolution: breaker may have moved into the bolt (e.g., bump pop).
        // CCD can't detect this since it only sweeps bolt movement.
        let inside = bolt_pos.x > breaker_pos.x - expanded_half.x
            && bolt_pos.x < breaker_pos.x + expanded_half.x
            && bolt_pos.y > breaker_pos.y - expanded_half.y
            && bolt_pos.y < breaker_pos.y + expanded_half.y;

        if inside {
            bolt_position.0.y = above_y;
            if bolt_velocity.value.y <= 0.0 {
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

        let (direction, max_dist) = (bolt_velocity.value / speed, speed * dt);

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

            let advance = (hit.distance - CCD_EPSILON).max(0.0);
            let new_pos = bolt_pos + direction * advance;
            bolt_position.0 = new_pos;
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

            bolt_position.0.x = impact_pos.x;
            bolt_position.0.y = above_y;
        }

        writer.write(BoltHitBreaker { bolt: bolt_entity });
        if let (Some(pr), Some(p)) = (&mut piercing_remaining, piercing) {
            pr.0 = p.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_spatial2d::components::{Position2D, Spatial2D};

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
        shared::{EntityScale, GameDrawLayer},
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

    /// Breaker entities use `Position2D` as canonical position.
    fn spawn_breaker_at(app: &mut App, x: f32, y: f32) {
        app.world_mut().spawn((
            Breaker,
            BreakerTilt::default(),
            default_breaker_width(),
            default_breaker_height(),
            default_max_reflection_angle(),
            default_min_angle(),
            Position2D(Vec2::new(x, y)),
            Spatial2D,
            GameDrawLayer::Breaker,
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

    /// Bolt entities now use `Position2D` as canonical position.
    fn spawn_bolt(app: &mut App, x: f32, y: f32, vx: f32, vy: f32) -> Entity {
        app.world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(vx, vy),
                bolt_param_bundle(),
                Position2D(Vec2::new(x, y)),
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
            Position2D(Vec2::new(0.0, y_pos)),
            Spatial2D,
            GameDrawLayer::Breaker,
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
    struct HitBreakers(u32);

    fn collect_breaker_hits(
        mut reader: MessageReader<BoltHitBreaker>,
        mut hits: ResMut<HitBreakers>,
    ) {
        for _msg in reader.read() {
            hits.0 += 1;
        }
    }

    #[test]
    fn overlap_resolved_writes_position2d_y() {
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

        let pos = app.world().get::<Position2D>(bolt_entity).unwrap();
        let expected_y = animated_y + hh.half_height() + default_bolt_radius().0;
        assert!(
            (pos.0.y - expected_y).abs() < 1.0,
            "bolt Position2D.0.y should be pushed above breaker, y={:.1} expected={expected_y:.1}",
            pos.0.y
        );

        let hits = app.world().resource::<HitBreakers>();
        assert_eq!(
            hits.0, 1,
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

        let pos = app.world().get::<Position2D>(bolt_entity).unwrap();
        let min_y = animated_y + hh.half_height() + default_bolt_radius().0;
        assert!(
            pos.0.y >= min_y - 0.01,
            "bolt Position2D.0.y should be pushed above breaker, y={:.3} min={min_y:.3}",
            pos.0.y
        );

        let hits = app.world().resource::<HitBreakers>();
        assert!(
            hits.0 == 0,
            "upward bolt overlap should NOT send BoltHitBreaker"
        );
    }

    #[test]
    fn upward_bolt_side_hit_is_not_reflected() {
        let mut app = test_app();
        let breaker_y = -250.0;
        app.insert_resource(HitBreakers::default()).add_systems(
            FixedUpdate,
            collect_breaker_hits.after(bolt_breaker_collision),
        );
        spawn_breaker_at(&mut app, 0.0, breaker_y);

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
            hits.0 == 0,
            "upward side hit should NOT send BoltHitBreaker, got {} messages",
            hits.0
        );
    }

    #[test]
    fn downward_bolt_side_hit_is_reflected() {
        let mut app = test_app();
        let breaker_y = -250.0;
        app.insert_resource(HitBreakers::default()).add_systems(
            FixedUpdate,
            collect_breaker_hits.after(bolt_breaker_collision),
        );
        spawn_breaker_at(&mut app, 0.0, breaker_y);

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
        assert_eq!(hits.0, 2, "both bolts should trigger hit messages");

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

    // --- Bolt entity in message tests ---

    #[derive(Resource, Default)]
    struct CapturedHitBolts(Vec<Entity>);

    fn collect_breaker_hit_bolts(
        mut reader: MessageReader<BoltHitBreaker>,
        mut captured: ResMut<CapturedHitBolts>,
    ) {
        for msg in reader.read() {
            captured.0.push(msg.bolt);
        }
    }

    #[test]
    fn bolt_entity_in_hit_message() {
        let mut app = test_app();
        let hh = default_breaker_height();
        let y_pos = -250.0;
        app.insert_resource(CapturedHitBolts::default())
            .add_systems(
                FixedUpdate,
                collect_breaker_hit_bolts.after(bolt_breaker_collision),
            );

        spawn_breaker_at(&mut app, 0.0, y_pos);

        let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
        let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);
        tick(&mut app);

        let captured = app.world().resource::<CapturedHitBolts>();
        assert_eq!(
            captured.0.len(),
            1,
            "should capture exactly one BoltHitBreaker message"
        );
        assert_eq!(
            captured.0[0], bolt_entity,
            "BoltHitBreaker.bolt should carry the actual bolt entity that hit the breaker"
        );
    }

    // --- Chip effect reset tests ---

    #[test]
    fn breaker_hit_resets_piercing_remaining() {
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
                Piercing(3),
                PiercingRemaining(0),
                Position2D(Vec2::new(0.0, start_y)),
            ))
            .id();

        tick(&mut app);

        let vel = app.world().get::<BoltVelocity>(bolt_entity).unwrap();
        assert!(
            vel.value.y > 0.0,
            "bolt should have reflected off breaker, got vy={}",
            vel.value.y
        );

        let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
        assert_eq!(
            pr.0, 3,
            "breaker hit should reset PiercingRemaining to Piercing.0 (3), got {}",
            pr.0
        );
    }

    #[test]
    fn piercing_remaining_without_piercing_does_not_reset_on_breaker_hit() {
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
                PiercingRemaining(5),
                Position2D(Vec2::new(0.0, start_y)),
            ))
            .id();

        tick(&mut app);

        let vel = app.world().get::<BoltVelocity>(bolt_entity).unwrap();
        assert!(
            vel.value.y > 0.0,
            "bolt should have reflected off breaker, got vy={}",
            vel.value.y
        );

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
        let mut app = test_app();
        let hh = default_breaker_height();
        let hw = default_breaker_width();
        let y_pos = -250.0;

        app.world_mut().spawn((
            Breaker,
            BreakerTilt::default(),
            BreakerWidth(120.0),
            default_breaker_height(),
            default_max_reflection_angle(),
            default_min_angle(),
            TiltControlBoost(15.0),
            Position2D(Vec2::new(0.0, y_pos)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));

        let hit_x = hw.half_width() - 2.0;
        let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
        let bolt_entity = spawn_bolt(&mut app, hit_x, start_y, 0.0, -400.0);

        tick(&mut app);

        let vel_with_boost = app.world().get::<BoltVelocity>(bolt_entity).unwrap().value;
        let angle_with_boost = vel_with_boost.x.abs().atan2(vel_with_boost.y);

        // Now test without boost for comparison
        let mut app_no_boost = test_app();
        app_no_boost.world_mut().spawn((
            Breaker,
            BreakerTilt::default(),
            default_breaker_width(),
            default_breaker_height(),
            default_max_reflection_angle(),
            default_min_angle(),
            Position2D(Vec2::new(0.0, y_pos)),
            Spatial2D,
            GameDrawLayer::Breaker,
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
        let mut app = test_app();
        let hh = default_breaker_height();
        let y_pos = -250.0;

        app.world_mut().spawn((
            Breaker,
            BreakerTilt::default(),
            default_breaker_width(),
            default_breaker_height(),
            default_max_reflection_angle(),
            default_min_angle(),
            WidthBoost(40.0),
            Position2D(Vec2::new(0.0, y_pos)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));

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

    // --- EntityScale collision tests ---

    fn spawn_scaled_breaker_at(app: &mut App, x: f32, y: f32, entity_scale: f32) {
        app.world_mut().spawn((
            Breaker,
            BreakerTilt::default(),
            default_breaker_width(),
            default_breaker_height(),
            default_max_reflection_angle(),
            default_min_angle(),
            EntityScale(entity_scale),
            Position2D(Vec2::new(x, y)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));
    }

    fn spawn_scaled_bolt(
        app: &mut App,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        entity_scale: f32,
    ) -> Entity {
        app.world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(vx, vy),
                bolt_param_bundle(),
                EntityScale(entity_scale),
                Position2D(Vec2::new(x, y)),
            ))
            .id()
    }

    #[test]
    fn scaled_breaker_has_smaller_collision_hitbox() {
        let mut app = test_app();
        let y_pos = -250.0;
        spawn_scaled_breaker_at(&mut app, 0.0, y_pos, 0.7);

        let bolt_entity = spawn_bolt(&mut app, 0.0, -234.0, 0.0, -1.0);
        tick(&mut app);

        let vel = app.world().get::<BoltVelocity>(bolt_entity).unwrap();
        assert!(
            vel.value.y < 0.0,
            "bolt at y=-234 should NOT be inside scaled breaker (scaled expanded top=-235), \
             got vy={:.1} (if positive, overlap resolution fired with unscaled dimensions)",
            vel.value.y
        );
    }

    #[test]
    fn bolt_outside_scaled_breaker_width_misses() {
        let mut app = test_app();
        let y_pos = -250.0;
        spawn_scaled_breaker_at(&mut app, 0.0, y_pos, 0.7);

        let scaled_half_h = 10.0 * 0.7;
        let bolt_r = default_bolt_radius().0;
        let start_y = y_pos + scaled_half_h + bolt_r + 3.0;
        let bolt_entity = spawn_bolt(&mut app, 55.0, start_y, 0.0, -400.0);
        tick(&mut app);

        let vel = app.world().get::<BoltVelocity>(bolt_entity).unwrap();
        assert!(
            vel.value.y < 0.0,
            "bolt at x=55 should miss scaled breaker (expanded half_w=50), got vy={:.1}",
            vel.value.y
        );
    }

    #[test]
    fn width_boost_stacks_with_entity_scale() {
        let mut app = test_app();
        let hh = default_breaker_height();
        let y_pos = -250.0;

        app.world_mut().spawn((
            Breaker,
            BreakerTilt::default(),
            default_breaker_width(),
            default_breaker_height(),
            default_max_reflection_angle(),
            default_min_angle(),
            WidthBoost(40.0),
            EntityScale(0.7),
            Position2D(Vec2::new(0.0, y_pos)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));

        let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
        let bolt_entity = spawn_bolt(&mut app, 70.0, start_y, 0.0, -400.0);
        tick(&mut app);

        let vel = app.world().get::<BoltVelocity>(bolt_entity).unwrap();
        assert!(
            vel.value.y < 0.0,
            "bolt at x=70 should miss scaled breaker (expanded half_w=64), got vy={:.1}",
            vel.value.y
        );
    }

    #[test]
    fn entity_scale_1_0_is_backward_compatible_with_breaker_collision() {
        let mut app = test_app();
        let hh = default_breaker_height();
        let y_pos = -250.0;
        spawn_scaled_breaker_at(&mut app, 0.0, y_pos, 1.0);

        let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
        spawn_scaled_bolt(&mut app, 0.0, start_y, 0.0, -400.0, 1.0);
        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.value.y > 0.0,
            "EntityScale(1.0) should produce identical behavior to no scale, got vy={:.1}",
            vel.value.y
        );
    }

    /// Breaker position is read from `Position2D` (not `Transform`).
    /// This test verifies that the system reads breaker from `Position2D.0`.
    #[test]
    fn breaker_position_read_from_position2d() {
        let mut app = test_app();
        let hh = default_breaker_height();
        let breaker_x = 50.0;
        let breaker_y = -250.0;
        // Breaker at non-zero X via Position2D
        spawn_breaker_at(&mut app, breaker_x, breaker_y);

        let start_y = breaker_y + hh.half_height() + default_bolt_radius().0 + 3.0;
        // Bolt at breaker_x so it hits the center
        spawn_bolt(&mut app, breaker_x, start_y, 0.0, -400.0);
        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.value.y > 0.0,
            "bolt should reflect off breaker at Position2D (50, -250), got vy={:.1}",
            vel.value.y
        );
    }
}
