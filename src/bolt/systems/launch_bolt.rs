//! System to launch a serving bolt when the player presses the bump button.

use bevy::prelude::*;

use crate::bolt::components::{Bolt, BoltServing, BoltVelocity};
use crate::bolt::resources::BoltConfig;

type LaunchBoltFilter = (With<Bolt>, With<BoltServing>);

/// Launches the bolt when the player presses the bump button (Up / W).
///
/// Removes [`BoltServing`] and sets the launch velocity. Only affects
/// bolts that are currently serving.
pub fn launch_bolt(
    keyboard: Res<ButtonInput<KeyCode>>,
    config: Res<BoltConfig>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut BoltVelocity), LaunchBoltFilter>,
) {
    if !keyboard.just_pressed(KeyCode::ArrowUp) && !keyboard.just_pressed(KeyCode::KeyW) {
        return;
    }

    for (entity, mut velocity) in &mut query {
        let vx = config.base_speed * config.initial_angle.sin();
        let vy = config.base_speed * config.initial_angle.cos();
        velocity.value = Vec2::new(vx, vy);
        commands.entity(entity).remove::<BoltServing>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BoltConfig>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, launch_bolt);
        app
    }

    #[test]
    fn pressing_up_launches_serving_bolt() {
        let mut app = test_app();

        app.world_mut()
            .spawn((Bolt, BoltServing, BoltVelocity::new(0.0, 0.0)));

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowUp);
        app.update();

        // BoltServing should be removed
        let serving_count = app
            .world_mut()
            .query_filtered::<Entity, (With<Bolt>, With<BoltServing>)>()
            .iter(app.world())
            .count();
        assert_eq!(
            serving_count, 0,
            "BoltServing should be removed after launch"
        );

        // Velocity should be non-zero and upward
        let velocity = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .expect("bolt should have velocity");
        assert!(velocity.value.y > 0.0, "bolt should launch upward");
        assert!(velocity.speed() > 0.0, "bolt should have non-zero speed");
    }

    #[test]
    fn pressing_w_also_launches() {
        let mut app = test_app();

        app.world_mut()
            .spawn((Bolt, BoltServing, BoltVelocity::new(0.0, 0.0)));

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);
        app.update();

        let serving_count = app
            .world_mut()
            .query_filtered::<Entity, (With<Bolt>, With<BoltServing>)>()
            .iter(app.world())
            .count();
        assert_eq!(
            serving_count, 0,
            "BoltServing should be removed after W press"
        );
    }

    #[test]
    fn no_input_keeps_serving() {
        let mut app = test_app();

        app.world_mut()
            .spawn((Bolt, BoltServing, BoltVelocity::new(0.0, 0.0)));

        app.update();

        let serving_count = app
            .world_mut()
            .query_filtered::<Entity, (With<Bolt>, With<BoltServing>)>()
            .iter(app.world())
            .count();
        assert_eq!(
            serving_count, 1,
            "bolt should still be serving without input"
        );

        let velocity = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .expect("bolt should have velocity");
        assert!(
            velocity.speed() < f32::EPSILON,
            "serving bolt should have zero velocity"
        );
    }

    #[test]
    fn non_serving_bolt_unaffected() {
        let mut app = test_app();

        // Bolt without BoltServing
        app.world_mut()
            .spawn((Bolt, BoltVelocity::new(100.0, 300.0)));

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowUp);
        app.update();

        let velocity = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .expect("bolt should have velocity");
        assert!(
            (velocity.value.x - 100.0).abs() < f32::EPSILON,
            "non-serving bolt velocity should be unchanged"
        );
    }
}
