//! Bolt-lost detection — bolt falls below the playfield.

use bevy::prelude::*;

use crate::bolt::BoltConfig;
use crate::bolt::components::{Bolt, BoltVelocity};
use crate::breaker::components::Breaker;
use crate::physics::messages::BoltLost;
use crate::shared::PlayfieldConfig;

/// Detects when the bolt falls below the playfield and respawns it.
///
/// Sends a [`BoltLost`] message. In Phase 2, the breaker plugin will
/// apply penalties per breaker type.
pub fn bolt_lost(
    bolt_config: Res<BoltConfig>,
    playfield: Res<PlayfieldConfig>,
    mut bolt_query: Query<(&mut Transform, &mut BoltVelocity), With<Bolt>>,
    breaker_query: Query<&Transform, (With<Breaker>, Without<Bolt>)>,
    mut writer: MessageWriter<BoltLost>,
) {
    let Ok(breaker_transform) = breaker_query.single() else {
        return;
    };
    let breaker_pos = breaker_transform.translation;

    for (mut bolt_transform, mut bolt_velocity) in &mut bolt_query {
        if bolt_transform.translation.y < playfield.bottom() - bolt_config.radius {
            writer.write(BoltLost);

            // Respawn above breaker
            bolt_transform.translation.x = breaker_pos.x;
            bolt_transform.translation.y = breaker_pos.y + bolt_config.respawn_offset_y;

            // Relaunch straight up at min speed — losing the bolt should sting
            bolt_velocity.value = Vec2::new(0.0, bolt_config.min_speed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::components::{Bolt, BoltVelocity};
    use crate::breaker::components::Breaker;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BoltConfig>();
        app.init_resource::<PlayfieldConfig>();
        app.add_message::<BoltLost>();
        app.add_systems(Update, bolt_lost);
        app
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
            (vel.value.y - bolt_config.min_speed).abs() < 1.0,
            "respawn vy should equal min_speed {:.0}, got {:.1}",
            bolt_config.min_speed,
            vel.value.y,
        );
        assert!(
            (transform.translation.x - breaker_x).abs() < f32::EPSILON,
            "respawn X should match breaker X {breaker_x:.0}, got {:.1}",
            transform.translation.x,
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
