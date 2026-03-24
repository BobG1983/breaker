//! System to launch a serving bolt when the player presses the bump button.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::{components::*, filters::ServingFilter},
    input::resources::{GameAction, InputActions},
};

/// Launches the bolt when the player activates bump.
///
/// Removes [`BoltServing`] and sets the launch velocity. Only affects
/// bolts that are currently serving.
pub fn launch_bolt(
    actions: Res<InputActions>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Velocity2D, &BoltBaseSpeed, &BoltInitialAngle), ServingFilter>,
) {
    if !actions.active(GameAction::Bump) {
        return;
    }

    for (entity, mut velocity, base_speed, initial_angle) in &mut query {
        velocity.0 = Vec2::new(
            base_speed.0 * initial_angle.0.sin(),
            base_speed.0 * initial_angle.0.cos(),
        );
        commands.entity(entity).remove::<BoltServing>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::resources::BoltConfig;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<InputActions>()
            .add_systems(FixedUpdate, launch_bolt);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn bolt_launch_bundle() -> (BoltBaseSpeed, BoltInitialAngle) {
        let config = BoltConfig::default();
        (
            BoltBaseSpeed(config.base_speed),
            BoltInitialAngle(config.initial_angle),
        )
    }

    #[test]
    fn bump_launches_serving_bolt() {
        let mut app = test_app();

        app.world_mut().spawn((
            Bolt,
            BoltServing,
            Velocity2D(Vec2::new(0.0, 0.0)),
            bolt_launch_bundle(),
        ));

        app.world_mut()
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::Bump);
        tick(&mut app);

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
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .expect("bolt should have velocity");
        assert!(velocity.0.y > 0.0, "bolt should launch upward");
        assert!(velocity.speed() > 0.0, "bolt should have non-zero speed");
    }

    #[test]
    fn no_input_keeps_serving() {
        let mut app = test_app();

        app.world_mut().spawn((
            Bolt,
            BoltServing,
            Velocity2D(Vec2::new(0.0, 0.0)),
            bolt_launch_bundle(),
        ));

        tick(&mut app);

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
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .expect("bolt should have velocity");
        assert!(
            velocity.speed() < f32::EPSILON,
            "serving bolt should have zero velocity"
        );
    }

    #[test]
    fn launch_velocity_matches_base_speed_and_angle() {
        let mut app = test_app();
        let config = BoltConfig::default();

        app.world_mut().spawn((
            Bolt,
            BoltServing,
            Velocity2D(Vec2::new(0.0, 0.0)),
            bolt_launch_bundle(),
        ));

        app.world_mut()
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::Bump);
        tick(&mut app);

        let velocity = app
            .world_mut()
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .expect("bolt should have velocity");

        let expect_x = config.base_speed * config.initial_angle.sin();
        let expect_y = config.base_speed * config.initial_angle.cos();
        assert!(
            (velocity.0.x - expect_x).abs() < 1e-4,
            "vx should be base_speed * sin(angle), got {} expected {expect_x}",
            velocity.0.x
        );
        assert!(
            (velocity.0.y - expect_y).abs() < 1e-4,
            "vy should be base_speed * cos(angle), got {} expected {expect_y}",
            velocity.0.y
        );
    }

    #[test]
    fn non_serving_bolt_unaffected() {
        let mut app = test_app();

        // Bolt without BoltServing
        app.world_mut()
            .spawn((Bolt, Velocity2D(Vec2::new(100.0, 300.0))));

        app.world_mut()
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::Bump);
        tick(&mut app);

        let velocity = app
            .world_mut()
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .expect("bolt should have velocity");
        assert!(
            (velocity.0.x - 100.0).abs() < f32::EPSILON,
            "non-serving bolt velocity should be unchanged"
        );
    }
}
