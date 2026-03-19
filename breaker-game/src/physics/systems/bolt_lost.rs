//! Bolt-lost detection — bolt falls below the playfield.

use bevy::prelude::*;
use rand::Rng;

use crate::{
    bolt::{
        components::{Bolt, BoltVelocity},
        filters::ActiveBoltFilter,
        queries::BoltLostQuery,
    },
    breaker::components::Breaker,
    interpolate::components::PhysicsTranslation,
    physics::messages::BoltLost,
    shared::{GameRng, PlayfieldConfig},
};

/// Collected data for a single lost bolt, used as scratch storage between
/// the filter pass and the command-application pass.
#[derive(Clone, Copy)]
pub(crate) struct LostBoltEntry {
    entity: Entity,
    base_speed: f32,
    respawn_offset: f32,
    angle_spread: f32,
    is_extra: bool,
}

/// Detects when the bolt falls below the playfield.
///
/// Baseline bolts (without [`ExtraBolt`]) are respawned above the breaker.
/// Extra bolts (with [`ExtraBolt`]) are despawned permanently.
/// Sends a [`BoltLost`] message in both cases.
pub(crate) fn bolt_lost(
    mut commands: Commands,
    playfield: Res<PlayfieldConfig>,
    mut rng: ResMut<GameRng>,
    bolt_query: Query<BoltLostQuery, ActiveBoltFilter>,
    breaker_query: Query<&Transform, (With<Breaker>, Without<Bolt>)>,
    mut writer: MessageWriter<BoltLost>,
    mut lost_bolts: Local<Vec<LostBoltEntry>>,
) {
    let Ok(breaker_transform) = breaker_query.single() else {
        return;
    };
    let breaker_pos = breaker_transform.translation;

    // Collect lost bolts to avoid mutable borrow conflicts with despawn.
    // Local<Vec> reuses its heap allocation across frames — zero allocs after warmup.
    lost_bolts.clear();
    lost_bolts.extend(
        bolt_query
            .iter()
            .filter(|(_, tf, _, _, radius, ..)| tf.translation.y < playfield.bottom() - radius.0)
            .map(
                |(entity, _, _, base_speed, _, respawn_offset, angle_spread, is_extra)| {
                    LostBoltEntry {
                        entity,
                        base_speed: base_speed.0,
                        respawn_offset: respawn_offset.0,
                        angle_spread: angle_spread.0,
                        is_extra,
                    }
                },
            ),
    );

    for entry in &*lost_bolts {
        writer.write(BoltLost);

        if entry.is_extra {
            commands.entity(entry.entity).despawn();
        } else {
            // Respawn above breaker
            let angle = rng.0.random_range(-entry.angle_spread..=entry.angle_spread);
            // Angle from vertical: sin→X, cos→Y; positive Y is upward.
            let new_velocity =
                Vec2::new(entry.base_speed * angle.sin(), entry.base_speed * angle.cos());
            let new_pos = Vec3::new(breaker_pos.x, breaker_pos.y + entry.respawn_offset, 1.0);
            commands.entity(entry.entity).insert((
                Transform::from_xyz(new_pos.x, new_pos.y, new_pos.z),
                BoltVelocity {
                    value: new_velocity,
                },
                // Snap interpolation to avoid lerping through teleport
                PhysicsTranslation::new(new_pos),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::{
        components::{
            Bolt, BoltBaseSpeed, BoltRadius, BoltRespawnAngleSpread, BoltRespawnOffsetY,
            BoltVelocity, ExtraBolt,
        },
        resources::BoltConfig,
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<PlayfieldConfig>()
            .init_resource::<GameRng>()
            .add_message::<BoltLost>()
            .add_systems(FixedUpdate, bolt_lost);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn bolt_lost_bundle() -> (
        BoltBaseSpeed,
        BoltRadius,
        BoltRespawnOffsetY,
        BoltRespawnAngleSpread,
    ) {
        let config = BoltConfig::default();
        (
            BoltBaseSpeed(config.base_speed),
            BoltRadius(config.radius),
            BoltRespawnOffsetY(config.respawn_offset_y),
            BoltRespawnAngleSpread(config.respawn_angle_spread),
        )
    }

    #[test]
    fn bolt_below_floor_triggers_respawn() {
        let mut app = test_app();
        let playfield = PlayfieldConfig::default();
        app.world_mut()
            .spawn((Breaker, Transform::from_xyz(0.0, -250.0, 0.0)));

        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, -400.0),
            bolt_lost_bundle(),
            Transform::from_xyz(0.0, playfield.bottom() - 100.0, 0.0),
        ));
        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(vel.value.y > 0.0, "bolt should be relaunched upward");
    }

    #[test]
    fn respawn_launches_within_angle_spread() {
        let mut app = test_app();
        let bolt_config = BoltConfig::default();
        let playfield = PlayfieldConfig::default();
        let breaker_x = 42.0;
        app.world_mut()
            .spawn((Breaker, Transform::from_xyz(breaker_x, -250.0, 0.0)));

        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(100.0, -400.0),
            bolt_lost_bundle(),
            Transform::from_xyz(200.0, playfield.bottom() - 100.0, 0.0),
        ));
        tick(&mut app);

        let (vel, transform) = app
            .world_mut()
            .query::<(&BoltVelocity, &Transform)>()
            .iter(app.world())
            .next()
            .unwrap();

        let speed = vel.value.length();
        assert!(
            (speed - bolt_config.base_speed).abs() < 1.0,
            "respawn speed should equal base_speed {:.0}, got {:.1}",
            bolt_config.base_speed,
            speed,
        );

        let angle = vel.value.x.atan2(vel.value.y).abs();
        assert!(
            angle <= bolt_config.respawn_angle_spread + f32::EPSILON,
            "respawn angle {angle:.3} rad should be within spread {:.3} rad",
            bolt_config.respawn_angle_spread,
        );

        assert!(vel.value.y > 0.0, "respawn should launch upward");

        assert!(
            (transform.translation.x - breaker_x).abs() < f32::EPSILON,
            "respawn X should match breaker X {breaker_x:.0}, got {:.1}",
            transform.translation.x,
        );
    }

    #[test]
    fn respawn_with_zero_spread_launches_straight_up() {
        let mut app = test_app();
        let bolt_config = BoltConfig::default();
        let playfield = PlayfieldConfig::default();
        app.world_mut()
            .spawn((Breaker, Transform::from_xyz(0.0, -250.0, 0.0)));

        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(100.0, -400.0),
            (
                BoltBaseSpeed(bolt_config.base_speed),
                BoltRadius(bolt_config.radius),
                BoltRespawnOffsetY(bolt_config.respawn_offset_y),
                BoltRespawnAngleSpread(0.0),
            ),
            Transform::from_xyz(0.0, playfield.bottom() - 100.0, 0.0),
        ));
        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();

        assert!(
            vel.value.x.abs() < f32::EPSILON,
            "zero spread should launch straight up, got vx={:.3}",
            vel.value.x,
        );
    }

    #[test]
    fn respawn_y_uses_respawn_offset() {
        let mut app = test_app();
        let bolt_config = BoltConfig::default();
        let playfield = PlayfieldConfig::default();
        let breaker_y = -250.0;
        app.world_mut()
            .spawn((Breaker, Transform::from_xyz(0.0, breaker_y, 0.0)));

        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, -400.0),
            bolt_lost_bundle(),
            Transform::from_xyz(0.0, playfield.bottom() - 100.0, 0.0),
        ));
        tick(&mut app);

        let transform = app
            .world_mut()
            .query_filtered::<&Transform, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap();

        let expected_y = breaker_y + bolt_config.respawn_offset_y;
        assert!(
            (transform.translation.y - expected_y).abs() < f32::EPSILON,
            "respawn Y should be breaker_y + respawn_offset_y ({expected_y}), got {}",
            transform.translation.y,
        );
    }

    #[test]
    fn extra_bolt_below_floor_is_despawned() {
        let mut app = test_app();
        let playfield = PlayfieldConfig::default();
        app.world_mut()
            .spawn((Breaker, Transform::from_xyz(0.0, -250.0, 0.0)));

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                ExtraBolt,
                BoltVelocity::new(0.0, -400.0),
                bolt_lost_bundle(),
                Transform::from_xyz(0.0, playfield.bottom() - 100.0, 0.0),
            ))
            .id();
        tick(&mut app);

        assert!(
            app.world().get_entity(entity).is_err(),
            "extra bolt should be despawned when lost"
        );
    }

    #[derive(Resource, Default)]
    struct BoltLostCount(u32);

    fn count_bolt_lost(mut reader: MessageReader<BoltLost>, mut count: ResMut<BoltLostCount>) {
        for _msg in reader.read() {
            count.0 += 1;
        }
    }

    #[test]
    fn extra_bolt_sends_bolt_lost_on_despawn() {
        let mut app = test_app();
        let playfield = PlayfieldConfig::default();
        app.world_mut()
            .spawn((Breaker, Transform::from_xyz(0.0, -250.0, 0.0)));

        app.init_resource::<BoltLostCount>();
        app.add_systems(FixedUpdate, count_bolt_lost.after(bolt_lost));

        app.world_mut().spawn((
            Bolt,
            ExtraBolt,
            BoltVelocity::new(0.0, -400.0),
            bolt_lost_bundle(),
            Transform::from_xyz(0.0, playfield.bottom() - 100.0, 0.0),
        ));
        tick(&mut app);

        let count = app.world().resource::<BoltLostCount>();
        assert_eq!(count.0, 1, "BoltLost message should be sent for extra bolt");
    }

    #[test]
    fn baseline_bolt_still_respawns_with_extra_present() {
        let mut app = test_app();
        let playfield = PlayfieldConfig::default();
        app.world_mut()
            .spawn((Breaker, Transform::from_xyz(0.0, -250.0, 0.0)));

        // Baseline bolt (no ExtraBolt)
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, -400.0),
            bolt_lost_bundle(),
            Transform::from_xyz(0.0, playfield.bottom() - 100.0, 0.0),
        ));
        // Extra bolt
        app.world_mut().spawn((
            Bolt,
            ExtraBolt,
            BoltVelocity::new(0.0, -400.0),
            bolt_lost_bundle(),
            Transform::from_xyz(50.0, playfield.bottom() - 100.0, 0.0),
        ));
        tick(&mut app);

        // Baseline bolt should still exist (respawned)
        let bolt_count = app
            .world_mut()
            .query_filtered::<Entity, With<Bolt>>()
            .iter(app.world())
            .count();
        assert_eq!(bolt_count, 1, "only baseline bolt should remain");

        // Verify it's the baseline (no ExtraBolt)
        let extra_count = app
            .world_mut()
            .query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()
            .iter(app.world())
            .count();
        assert_eq!(extra_count, 0, "extra bolt should be gone");
    }

    #[test]
    fn bolt_above_floor_not_lost() {
        let mut app = test_app();
        app.world_mut()
            .spawn((Breaker, Transform::from_xyz(0.0, -250.0, 0.0)));

        let original_y = 100.0;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(100.0, -200.0),
            bolt_lost_bundle(),
            Transform::from_xyz(0.0, original_y, 0.0),
        ));
        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(vel.value.y < 0.0, "bolt above floor should keep going down");
    }
}
