//! System to launch a serving bolt when the player presses the bump button.

use bevy::prelude::*;

use crate::{
    bolt::{
        components::{Bolt, BoltServing, BoltVelocity},
        resources::BoltConfig,
    },
    input::resources::{GameAction, InputActions},
};

type LaunchBoltFilter = (With<Bolt>, With<BoltServing>);

/// Launches the bolt when the player activates bump.
///
/// Removes [`BoltServing`] and sets the launch velocity. Only affects
/// bolts that are currently serving.
pub fn launch_bolt(
    actions: Res<InputActions>,
    config: Res<BoltConfig>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut BoltVelocity), LaunchBoltFilter>,
) {
    if !actions.active(GameAction::Bump) {
        return;
    }

    for (entity, mut velocity) in &mut query {
        velocity.value = config.initial_velocity();
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
        app.init_resource::<InputActions>();
        app.add_systems(Update, launch_bolt);
        app
    }

    #[test]
    fn bump_launches_serving_bolt() {
        let mut app = test_app();

        app.world_mut()
            .spawn((Bolt, BoltServing, BoltVelocity::new(0.0, 0.0)));

        app.world_mut()
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::Bump);
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
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::Bump);
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
