//! Bolt-lost detection — bolt falls below the playfield.

use bevy::prelude::*;
use rand::Rng;
use rantzsoft_spatial2d::components::{Position2D, PreviousPosition, Velocity2D};

use crate::{
    bolt::{filters::ActiveFilter, messages::BoltLost, queries::LostQuery},
    breaker::filters::CollisionFilterBreaker,
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
    bolt_query: Query<LostQuery, ActiveFilter>,
    breaker_query: Query<&Position2D, CollisionFilterBreaker>,
    mut writer: MessageWriter<BoltLost>,
    mut lost_bolts: Local<Vec<LostBoltEntry>>,
) {
    let Ok(breaker_position) = breaker_query.single() else {
        return;
    };
    let breaker_pos = breaker_position.0;

    // Collect lost bolts to avoid mutable borrow conflicts with despawn.
    // `Local<Vec>` reuses its heap allocation across frames — zero allocs after warmup.
    lost_bolts.clear();
    lost_bolts.extend(
        bolt_query
            .iter()
            .filter(|(_, pos, _, _, radius, _, _, _, entity_scale)| {
                let r = radius.0 * entity_scale.map_or(1.0, |s| s.0);
                pos.0.y < playfield.bottom() - r
            })
            .map(
                |(
                    entity,
                    _,
                    _,
                    base_speed,
                    _,
                    respawn_offset,
                    angle_spread,
                    is_extra,
                    _entity_scale,
                )| {
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
            // Angle from vertical: sin->X, cos->Y; positive Y is upward.
            let new_velocity = Vec2::new(
                entry.base_speed * angle.sin(),
                entry.base_speed * angle.cos(),
            );
            let new_pos = Vec2::new(breaker_pos.x, breaker_pos.y + entry.respawn_offset);
            commands.entity(entry.entity).insert((
                Position2D(new_pos),
                PreviousPosition(new_pos),
                Velocity2D(new_velocity),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_spatial2d::components::{Position2D, PreviousPosition, Spatial2D, Velocity2D};

    use super::*;
    use crate::{
        bolt::{
            components::{
                Bolt, BoltBaseSpeed, BoltRadius, BoltRespawnAngleSpread, BoltRespawnOffsetY,
                ExtraBolt,
            },
            resources::BoltConfig,
        },
        breaker::components::Breaker,
        shared::{EntityScale, GameDrawLayer},
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
    fn bolt_below_floor_detected_via_position2d() {
        let mut app = test_app();
        let playfield = PlayfieldConfig::default();
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(0.0, -250.0)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));

        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, -400.0)),
            bolt_lost_bundle(),
            Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
        ));
        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(vel.0.y > 0.0, "bolt should be relaunched upward");
    }

    #[test]
    fn respawn_inserts_position2d_at_breaker_x() {
        let mut app = test_app();
        let bolt_config = BoltConfig::default();
        let playfield = PlayfieldConfig::default();
        let breaker_x = 42.0;
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(breaker_x, -250.0)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));

        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(100.0, -400.0)),
            bolt_lost_bundle(),
            Position2D(Vec2::new(200.0, playfield.bottom() - 100.0)),
        ));
        tick(&mut app);

        let (vel, pos) = app
            .world_mut()
            .query::<(&Velocity2D, &Position2D)>()
            .iter(app.world())
            .next()
            .unwrap();

        let speed = vel.0.length();
        assert!(
            (speed - bolt_config.base_speed).abs() < 1.0,
            "respawn speed should equal base_speed {:.0}, got {:.1}",
            bolt_config.base_speed,
            speed,
        );

        let angle = vel.0.x.atan2(vel.0.y).abs();
        assert!(
            angle <= bolt_config.respawn_angle_spread + f32::EPSILON,
            "respawn angle {angle:.3} rad should be within spread {:.3} rad",
            bolt_config.respawn_angle_spread,
        );

        assert!(vel.0.y > 0.0, "respawn should launch upward");

        assert!(
            (pos.0.x - breaker_x).abs() < f32::EPSILON,
            "respawn Position2D.0.x should match breaker X {breaker_x:.0}, got {:.1}",
            pos.0.x,
        );
    }

    #[test]
    fn respawn_with_zero_spread_launches_straight_up() {
        let mut app = test_app();
        let bolt_config = BoltConfig::default();
        let playfield = PlayfieldConfig::default();
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(0.0, -250.0)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));

        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(100.0, -400.0)),
            (
                BoltBaseSpeed(bolt_config.base_speed),
                BoltRadius(bolt_config.radius),
                BoltRespawnOffsetY(bolt_config.respawn_offset_y),
                BoltRespawnAngleSpread(0.0),
            ),
            Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
        ));
        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .unwrap();

        assert!(
            vel.0.x.abs() < f32::EPSILON,
            "zero spread should launch straight up, got vx={:.3}",
            vel.0.x,
        );
    }

    #[test]
    fn respawn_position2d_y_uses_respawn_offset() {
        let mut app = test_app();
        let bolt_config = BoltConfig::default();
        let playfield = PlayfieldConfig::default();
        let breaker_y = -250.0;
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(0.0, breaker_y)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));

        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, -400.0)),
            bolt_lost_bundle(),
            Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
        ));
        tick(&mut app);

        let pos = app
            .world_mut()
            .query_filtered::<&Position2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap();

        let expected_y = breaker_y + bolt_config.respawn_offset_y;
        assert!(
            (pos.0.y - expected_y).abs() < f32::EPSILON,
            "respawn Position2D.0.y should be breaker_y + respawn_offset_y ({expected_y}), got {}",
            pos.0.y,
        );
    }

    #[test]
    fn respawn_inserts_previous_position_matching_position2d() {
        let mut app = test_app();
        let bolt_config = BoltConfig::default();
        let playfield = PlayfieldConfig::default();
        let breaker_x = 42.0;
        let breaker_y = -250.0;
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(breaker_x, breaker_y)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));

        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, -400.0)),
            bolt_lost_bundle(),
            Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
        ));
        tick(&mut app);

        let (pos, prev_pos) = app
            .world_mut()
            .query_filtered::<(&Position2D, &PreviousPosition), With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap();

        let expected = Vec2::new(breaker_x, breaker_y + bolt_config.respawn_offset_y);
        assert!(
            (pos.0 - expected).length() < f32::EPSILON,
            "respawn Position2D should be ({expected:?}), got {:?}",
            pos.0,
        );
        assert!(
            (prev_pos.0 - expected).length() < f32::EPSILON,
            "respawn PreviousPosition should match Position2D ({expected:?}), got {:?}",
            prev_pos.0,
        );
    }

    #[test]
    fn extra_bolt_below_floor_is_despawned() {
        let mut app = test_app();
        let playfield = PlayfieldConfig::default();
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(0.0, -250.0)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                ExtraBolt,
                Velocity2D(Vec2::new(0.0, -400.0)),
                bolt_lost_bundle(),
                Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
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
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(0.0, -250.0)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));

        app.init_resource::<BoltLostCount>();
        app.add_systems(FixedUpdate, count_bolt_lost.after(bolt_lost));

        app.world_mut().spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, -400.0)),
            bolt_lost_bundle(),
            Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
        ));
        tick(&mut app);

        let count = app.world().resource::<BoltLostCount>();
        assert_eq!(count.0, 1, "BoltLost message should be sent for extra bolt");
    }

    #[test]
    fn baseline_bolt_still_respawns_with_extra_present() {
        let mut app = test_app();
        let playfield = PlayfieldConfig::default();
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(0.0, -250.0)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));

        // Baseline bolt (no ExtraBolt)
        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, -400.0)),
            bolt_lost_bundle(),
            Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
        ));
        // Extra bolt
        app.world_mut().spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, -400.0)),
            bolt_lost_bundle(),
            Position2D(Vec2::new(50.0, playfield.bottom() - 100.0)),
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
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(0.0, -250.0)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));

        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(100.0, -200.0)),
            bolt_lost_bundle(),
            Position2D(Vec2::new(0.0, 100.0)),
        ));
        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(vel.0.y < 0.0, "bolt above floor should keep going down");
    }

    // --- EntityScale lost detection tests ---

    #[test]
    fn scaled_bolt_uses_effective_radius_for_lost_detection() {
        let mut app = test_app();
        let playfield = PlayfieldConfig::default();
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(0.0, -250.0)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));

        let bolt_y = playfield.bottom() - 4.0 - 1.0; // -305.0
        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, -400.0)),
            bolt_lost_bundle(),
            EntityScale(0.5),
            Position2D(Vec2::new(0.0, bolt_y)),
        ));
        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.0.y > 0.0,
            "scaled bolt below effective threshold should be respawned (vy > 0), got vy={:.1}",
            vel.0.y
        );
    }

    #[test]
    fn bolt_without_entity_scale_in_lost_detection_is_backward_compatible() {
        let mut app = test_app();
        let playfield = PlayfieldConfig::default();
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(0.0, -250.0)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));

        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, -400.0)),
            bolt_lost_bundle(),
            // No EntityScale
            Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
        ));
        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.0.y > 0.0,
            "bolt without EntityScale should be respawned normally, got vy={:.1}",
            vel.0.y
        );
    }
}
