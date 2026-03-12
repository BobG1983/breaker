//! System to spawn the breaker entity.

use bevy::prelude::*;

use crate::{
    breaker::{
        components::{
            Breaker, BreakerState, BreakerStateTimer, BreakerTilt, BreakerVelocity, BumpState,
        },
        queries::BreakerResetQuery,
        resources::BreakerConfig,
    },
    shared::{CleanupOnRunEnd, PlayfieldConfig},
};

/// Spawns the breaker entity with all required components.
///
/// Runs when entering [`GameState::Playing`]. If a breaker already exists
/// (persisted from a previous node), this is a no-op.
pub fn spawn_breaker(
    mut commands: Commands,
    config: Res<BreakerConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    existing: Query<Entity, With<Breaker>>,
) {
    if existing.iter().next().is_some() {
        return;
    }

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
            scale: Vec3::new(config.width, config.height, 1.0),
            ..default()
        },
        CleanupOnRunEnd,
    ));
}

/// Resets breaker state at the start of each node.
///
/// Runs when entering [`GameState::Playing`]. Returns breaker to center,
/// clears velocity/tilt/state. On the first node, `spawn_breaker` handles
/// initialization — this system is a no-op if no breaker exists yet.
pub fn reset_breaker(
    playfield: Res<PlayfieldConfig>,
    mut query: Query<BreakerResetQuery, With<Breaker>>,
) {
    let _ = &playfield; // future: use playfield for centering
    for (mut transform, mut state, mut velocity, mut tilt, mut timer, base_y) in &mut query {
        transform.translation.x = 0.0;
        transform.translation.y = base_y.0;
        *state = BreakerState::Idle;
        velocity.x = 0.0;
        tilt.angle = 0.0;
        tilt.settle_start_angle = 0.0;
        timer.remaining = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::breaker::components::{Breaker, BreakerBaseY};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BreakerConfig>();
        app.init_resource::<PlayfieldConfig>();
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

    #[test]
    fn no_double_spawn() {
        let mut app = test_app();
        app.update();

        // Run spawn_breaker again (simulating a second node entry)
        app.add_systems(Update, spawn_breaker);
        app.update();

        let count = app
            .world_mut()
            .query::<&Breaker>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "should not double-spawn breaker");
    }

    #[test]
    fn reset_breaker_restores_state() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BreakerConfig>();
        app.init_resource::<PlayfieldConfig>();
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();

        // Spawn breaker with modified state
        let config = BreakerConfig::default();
        app.world_mut().spawn((
            Breaker,
            BreakerVelocity { x: 300.0 },
            BreakerState::Dashing,
            BreakerTilt {
                angle: 0.5,
                settle_start_angle: 0.5,
            },
            BreakerStateTimer { remaining: 0.1 },
            BreakerBaseY(config.y_position),
            BumpState::default(),
            Transform::from_xyz(100.0, config.y_position + 50.0, 0.0),
            CleanupOnRunEnd,
        ));

        app.add_systems(Update, reset_breaker);
        app.update();

        let (transform, state, velocity, tilt, timer) = app
            .world_mut()
            .query::<(
                &Transform,
                &BreakerState,
                &BreakerVelocity,
                &BreakerTilt,
                &BreakerStateTimer,
            )>()
            .iter(app.world())
            .next()
            .expect("breaker should exist");

        assert_eq!(*state, BreakerState::Idle);
        assert!(velocity.x.abs() < f32::EPSILON);
        assert!(tilt.angle.abs() < f32::EPSILON);
        assert!(tilt.settle_start_angle.abs() < f32::EPSILON);
        assert!(timer.remaining.abs() < f32::EPSILON);
        assert!((transform.translation.x).abs() < f32::EPSILON);
        assert!((transform.translation.y - config.y_position).abs() < f32::EPSILON);
    }
}
