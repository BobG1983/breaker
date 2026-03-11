//! Wall and ceiling collision for the bolt.

use bevy::prelude::*;

use crate::bolt::BoltConfig;
use crate::bolt::components::BoltVelocity;
use crate::bolt::filters::ActiveBoltFilter;
use crate::shared::PlayfieldConfig;

/// Reflects the bolt off walls and ceiling.
///
/// Left/right walls reflect the X component, ceiling reflects Y.
/// The floor is intentionally not a boundary — bolt-lost is handled separately.
pub fn wall_collision(
    config: Res<BoltConfig>,
    playfield: Res<PlayfieldConfig>,
    mut query: Query<(&mut Transform, &mut BoltVelocity), ActiveBoltFilter>,
) {
    let left_bound = playfield.left() + config.radius;
    let right_bound = playfield.right() - config.radius;
    let top_bound = playfield.top() - config.radius;

    for (mut transform, mut velocity) in &mut query {
        let pos = &mut transform.translation;

        // Left wall
        if pos.x <= left_bound && velocity.value.x < 0.0 {
            velocity.value.x = -velocity.value.x;
            pos.x = left_bound;
        }

        // Right wall
        if pos.x >= right_bound && velocity.value.x > 0.0 {
            velocity.value.x = -velocity.value.x;
            pos.x = right_bound;
        }

        // Ceiling
        if pos.y >= top_bound && velocity.value.y > 0.0 {
            velocity.value.y = -velocity.value.y;
            pos.y = top_bound;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::components::{Bolt, BoltVelocity};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BoltConfig>();
        app.init_resource::<PlayfieldConfig>();
        app.add_systems(Update, wall_collision);
        app
    }

    #[test]
    fn bolt_reflects_off_left_wall() {
        let mut app = test_app();
        let config = BoltConfig::default();
        let playfield = PlayfieldConfig::default();
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(-300.0, 100.0),
            Transform::from_xyz(playfield.left() + config.radius - 1.0, 0.0, 0.0),
        ));
        app.update();

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(vel.value.x > 0.0, "x velocity should be reflected");
    }

    #[test]
    fn bolt_reflects_off_right_wall() {
        let mut app = test_app();
        let config = BoltConfig::default();
        let playfield = PlayfieldConfig::default();
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(300.0, 100.0),
            Transform::from_xyz(playfield.right() - config.radius + 1.0, 0.0, 0.0),
        ));
        app.update();

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(vel.value.x < 0.0, "x velocity should be reflected");
    }

    #[test]
    fn bolt_reflects_off_ceiling() {
        let mut app = test_app();
        let config = BoltConfig::default();
        let playfield = PlayfieldConfig::default();
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(100.0, 300.0),
            Transform::from_xyz(0.0, playfield.top() - config.radius + 1.0, 0.0),
        ));
        app.update();

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(vel.value.y < 0.0, "y velocity should be reflected");
    }

    #[test]
    fn bolt_does_not_reflect_off_floor() {
        let mut app = test_app();
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(100.0, -300.0),
            Transform::from_xyz(0.0, -500.0, 0.0),
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
            "y velocity should NOT be reflected at floor"
        );
    }
}
