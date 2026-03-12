//! Bolt-lost detection — bolt falls below the playfield.

use bevy::prelude::*;

use crate::{
    bolt::{
        components::{Bolt, BoltBaseSpeed, BoltRadius, BoltRespawnOffsetY, BoltVelocity},
        filters::ActiveBoltFilter,
    },
    breaker::components::Breaker,
    physics::messages::BoltLost,
    shared::PlayfieldConfig,
};

/// Detects when the bolt falls below the playfield and respawns it.
///
/// Sends a [`BoltLost`] message. In Phase 2, the breaker plugin will
/// apply penalties per breaker type.
pub fn bolt_lost(
    playfield: Res<PlayfieldConfig>,
    mut bolt_query: Query<
        (
            &mut Transform,
            &mut BoltVelocity,
            &BoltBaseSpeed,
            &BoltRadius,
            &BoltRespawnOffsetY,
        ),
        ActiveBoltFilter,
    >,
    breaker_query: Query<&Transform, (With<Breaker>, Without<Bolt>)>,
    mut writer: MessageWriter<BoltLost>,
) {
    let Ok(breaker_transform) = breaker_query.single() else {
        return;
    };
    let breaker_pos = breaker_transform.translation;

    for (mut bolt_transform, mut bolt_velocity, base_speed, radius, respawn_offset) in
        &mut bolt_query
    {
        if bolt_transform.translation.y < playfield.bottom() - radius.0 {
            writer.write(BoltLost);

            // Respawn above breaker
            bolt_transform.translation.x = breaker_pos.x;
            bolt_transform.translation.y = breaker_pos.y + respawn_offset.0;

            // Relaunch straight up at base speed — losing position is penalty enough
            bolt_velocity.value = Vec2::new(0.0, base_speed.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::{
        components::{Bolt, BoltBaseSpeed, BoltRadius, BoltRespawnOffsetY, BoltVelocity},
        resources::BoltConfig,
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<PlayfieldConfig>();
        app.add_message::<BoltLost>();
        app.add_systems(Update, bolt_lost);
        app
    }

    fn bolt_lost_bundle() -> (BoltBaseSpeed, BoltRadius, BoltRespawnOffsetY) {
        let config = BoltConfig::default();
        (
            BoltBaseSpeed(config.base_speed),
            BoltRadius(config.radius),
            BoltRespawnOffsetY(config.respawn_offset_y),
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
        app.update();

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(vel.value.y > 0.0, "bolt should be relaunched upward");
    }

    #[test]
    fn respawn_launches_straight_up() {
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
        app.update();

        let (vel, transform) = app
            .world_mut()
            .query::<(&BoltVelocity, &Transform)>()
            .iter(app.world())
            .next()
            .unwrap();

        assert!(
            vel.value.x.abs() < f32::EPSILON,
            "respawn should launch straight up (vx=0), got vx={:.3}",
            vel.value.x,
        );
        assert!(
            (vel.value.y - bolt_config.base_speed).abs() < 1.0,
            "respawn vy should equal base_speed {:.0}, got {:.1}",
            bolt_config.base_speed,
            vel.value.y,
        );
        assert!(
            (transform.translation.x - breaker_x).abs() < f32::EPSILON,
            "respawn X should match breaker X {breaker_x:.0}, got {:.1}",
            transform.translation.x,
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
        app.update();

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
        app.update();

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(vel.value.y < 0.0, "bolt above floor should keep going down");
    }
}
