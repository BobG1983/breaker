//! System to spawn the breaker entity.

use bevy::prelude::*;

use crate::{
    breaker::{
        components::{
            Breaker, BreakerState, BreakerStateTimer, BreakerTilt, BreakerVelocity, BumpState,
        },
        resources::BreakerConfig,
    },
    shared::{CleanupOnNodeExit, CleanupOnRunEnd},
};

/// Spawns the breaker entity with all required components.
///
/// Runs once when entering [`GameState::Playing`].
pub fn spawn_breaker(
    mut commands: Commands,
    config: Res<BreakerConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Breaker,
        BreakerVelocity::default(),
        BreakerState::default(),
        BreakerTilt::default(),
        BumpState::default(),
        BreakerStateTimer::default(),
        Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(config.color()))),
        Transform {
            translation: Vec3::new(0.0, config.y_position, 0.0),
            scale: Vec3::new(config.half_width * 2.0, config.half_height * 2.0, 1.0),
            ..default()
        },
        CleanupOnNodeExit,
        CleanupOnRunEnd,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::breaker::components::Breaker;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BreakerConfig>();
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();
        app.add_systems(Startup, spawn_breaker);
        app
    }

    #[test]
    fn spawn_breaker_creates_entity() {
        let mut app = test_app();
        app.update();

        let count = app
            .world_mut()
            .query::<&Breaker>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn spawned_breaker_has_required_components() {
        let mut app = test_app();
        app.update();

        let mut query = app.world_mut().query::<(
            &Breaker,
            &BreakerVelocity,
            &BreakerState,
            &BreakerTilt,
            &BumpState,
            &Mesh2d,
            &MeshMaterial2d<ColorMaterial>,
            &Transform,
            &CleanupOnNodeExit,
            &CleanupOnRunEnd,
        )>();
        assert_eq!(query.iter(app.world()).count(), 1);
    }

    #[test]
    fn spawned_breaker_at_correct_position() {
        let mut app = test_app();
        app.update();

        let config = BreakerConfig::default();
        let transform = app
            .world_mut()
            .query::<&Transform>()
            .iter(app.world())
            .next()
            .expect("breaker should exist");
        assert!((transform.translation.y - config.y_position).abs() < f32::EPSILON);
        assert!((transform.translation.x - 0.0).abs() < f32::EPSILON);
    }
}
