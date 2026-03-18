//! System to spawn the breaker entity.

use bevy::prelude::*;
use tracing::debug;

use crate::{
    breaker::{
        components::{
            Breaker, BreakerState, BreakerStateTimer, BreakerTilt, BreakerVelocity, BumpState,
        },
        queries::BreakerResetQuery,
        resources::BreakerConfig,
    },
    interpolate::components::{InterpolateTransform, PhysicsTranslation},
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

    let spawn_pos = Vec3::new(0.0, config.y_position, 0.0);
    let entity = commands.spawn((
        Breaker,
        BreakerVelocity::default(),
        BreakerState::default(),
        BreakerTilt::default(),
        BumpState::default(),
        BreakerStateTimer::default(),
        InterpolateTransform,
        PhysicsTranslation::new(spawn_pos),
        Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(config.color()))),
        Transform {
            translation: spawn_pos,
            scale: Vec3::new(config.width, config.height, 1.0),
            ..default()
        },
        CleanupOnRunEnd,
    ));
    debug!("breaker spawned entity={:?}", entity.id());
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
    // Robust if PlayfieldConfig is ever offset from world origin
    let center_x = f32::midpoint(playfield.left(), playfield.right());
    for (mut transform, mut state, mut velocity, mut tilt, mut timer, mut bump, base_y, physics) in
        &mut query
    {
        transform.translation.x = center_x;
        transform.translation.y = base_y.0;
        *state = BreakerState::Idle;
        velocity.x = 0.0;
        tilt.angle = 0.0;
        tilt.ease_start = 0.0;
        tilt.ease_target = 0.0;
        timer.remaining = 0.0;
        bump.active = false;
        bump.timer = 0.0;
        bump.post_hit_timer = 0.0;
        bump.cooldown = 0.0;
        // Snap interpolation to avoid lerping through teleport
        if let Some(mut pt) = physics {
            let pos = Vec3::new(center_x, base_y.0, transform.translation.z);
            *pt = PhysicsTranslation::new(pos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::breaker::components::{Breaker, BreakerBaseY};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<BreakerConfig>()
            .init_resource::<PlayfieldConfig>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .add_systems(Startup, spawn_breaker);
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
        app.add_plugins(MinimalPlugins)
            .init_resource::<BreakerConfig>()
            .init_resource::<PlayfieldConfig>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>();

        // Spawn breaker with modified state (including active bump window)
        let config = BreakerConfig::default();
        app.world_mut().spawn((
            Breaker,
            BreakerVelocity { x: 300.0 },
            BreakerState::Dashing,
            BreakerTilt {
                angle: 0.5,
                ease_start: 0.5,
                ease_target: 0.0,
            },
            BreakerStateTimer { remaining: 0.1 },
            BreakerBaseY(config.y_position),
            BumpState {
                active: true,
                timer: 0.1,
                post_hit_timer: 0.05,
                cooldown: 0.2,
            },
            Transform::from_xyz(100.0, config.y_position + 50.0, 0.0),
            CleanupOnRunEnd,
        ));

        app.add_systems(Update, reset_breaker);
        app.update();

        let (transform, state, velocity, tilt, timer, bump) = app
            .world_mut()
            .query::<(
                &Transform,
                &BreakerState,
                &BreakerVelocity,
                &BreakerTilt,
                &BreakerStateTimer,
                &BumpState,
            )>()
            .iter(app.world())
            .next()
            .expect("breaker should exist");

        assert_eq!(*state, BreakerState::Idle);
        assert!(velocity.x.abs() < f32::EPSILON);
        assert!(tilt.angle.abs() < f32::EPSILON);
        assert!(tilt.ease_start.abs() < f32::EPSILON);
        assert!(timer.remaining.abs() < f32::EPSILON);
        assert!(!bump.active, "bump should be inactive after reset");
        assert!(
            bump.timer.abs() < f32::EPSILON,
            "bump timer should be cleared"
        );
        assert!(
            bump.post_hit_timer.abs() < f32::EPSILON,
            "post_hit_timer should be cleared"
        );
        assert!(
            bump.cooldown.abs() < f32::EPSILON,
            "cooldown should be cleared"
        );
        assert!((transform.translation.x).abs() < f32::EPSILON);
        assert!((transform.translation.y - config.y_position).abs() < f32::EPSILON);
    }
}
