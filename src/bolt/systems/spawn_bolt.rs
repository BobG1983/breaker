//! System to spawn the bolt entity.

use bevy::prelude::*;

use crate::bolt::components::{Bolt, BoltVelocity};
use crate::bolt::resources::BoltConfig;
use crate::breaker::components::Breaker;
use crate::shared::CleanupOnNodeExit;

/// Spawns the bolt entity above the breaker.
///
/// Runs once when entering [`GameState::Playing`].
pub fn spawn_bolt(
    mut commands: Commands,
    config: Res<BoltConfig>,
    breaker_query: Query<&Transform, With<Breaker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok(breaker_transform) = breaker_query.single() else {
        return;
    };
    let breaker_pos = breaker_transform.translation;

    let spawn_pos = Vec3::new(breaker_pos.x, breaker_pos.y + config.spawn_offset_y, 0.0);

    // Launch upward with a slight rightward angle
    let vx = config.base_speed * config.initial_angle.sin();
    let vy = config.base_speed * config.initial_angle.cos();

    commands.spawn((
        Bolt,
        BoltVelocity::new(vx, vy),
        Mesh2d(meshes.add(Circle::new(1.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(config.color()))),
        Transform {
            translation: spawn_pos,
            scale: Vec3::new(config.radius, config.radius, 1.0),
            ..default()
        },
        CleanupOnNodeExit,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::components::Bolt;
    use crate::breaker::BreakerConfig;
    use crate::breaker::components::Breaker;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BoltConfig>();
        app.init_resource::<BreakerConfig>();
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();
        app
    }

    #[test]
    fn spawn_bolt_creates_entity() {
        let mut app = test_app();
        // Spawn a breaker first so the bolt can reference its position
        app.world_mut()
            .spawn((Breaker, Transform::from_xyz(0.0, -250.0, 0.0)));
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let count = app.world_mut().query::<&Bolt>().iter(app.world()).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn bolt_spawns_above_breaker() {
        let mut app = test_app();
        let breaker_y = -250.0;
        app.world_mut()
            .spawn((Breaker, Transform::from_xyz(0.0, breaker_y, 0.0)));
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let bolt_transform = app
            .world_mut()
            .query_filtered::<&Transform, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should exist");
        assert!(bolt_transform.translation.y > breaker_y);
    }

    #[test]
    fn bolt_has_upward_velocity() {
        let mut app = test_app();
        app.world_mut()
            .spawn((Breaker, Transform::from_xyz(0.0, -250.0, 0.0)));
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let velocity = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .expect("bolt should have velocity");
        assert!(velocity.value.y > 0.0, "bolt should launch upward");
    }
}
