//! System to initialize bolt entity components from config.

use bevy::prelude::*;

use crate::bolt::{
    components::{
        Bolt, BoltBaseSpeed, BoltInitialAngle, BoltMaxSpeed, BoltMinSpeed, BoltRadius,
        BoltRespawnAngleSpread, BoltRespawnOffsetY, BoltSpawnOffsetY,
    },
    resources::BoltConfig,
};

/// Materializes config values as components on the bolt entity.
///
/// Runs `OnEnter(GameState::Playing)` after `spawn_bolt`. Uses
/// `Without<BoltBaseSpeed>` to skip already-initialized bolts.
pub fn init_bolt_params(
    mut commands: Commands,
    bolt_config: Res<BoltConfig>,
    query: Query<Entity, (With<Bolt>, Without<BoltBaseSpeed>)>,
) {
    for entity in &query {
        commands.entity(entity).insert((
            BoltBaseSpeed(bolt_config.base_speed),
            BoltMinSpeed(bolt_config.min_speed),
            BoltMaxSpeed(bolt_config.max_speed),
            BoltRadius(bolt_config.radius),
            BoltSpawnOffsetY(bolt_config.spawn_offset_y),
            BoltRespawnOffsetY(bolt_config.respawn_offset_y),
            BoltRespawnAngleSpread(bolt_config.respawn_angle_spread),
            BoltInitialAngle(bolt_config.initial_angle),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::components::{
        BoltInitialAngle, BoltRadius, BoltRespawnAngleSpread, BoltRespawnOffsetY, BoltSpawnOffsetY,
        BoltVelocity,
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BoltConfig>();
        app.add_systems(Update, init_bolt_params);
        app
    }

    #[test]
    fn init_inserts_all_components() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((Bolt, BoltVelocity::new(0.0, 400.0)))
            .id();

        app.update();

        let world = app.world();
        assert!(world.get::<BoltBaseSpeed>(entity).is_some());
        assert!(world.get::<BoltMinSpeed>(entity).is_some());
        assert!(world.get::<BoltMaxSpeed>(entity).is_some());
        assert!(world.get::<BoltRadius>(entity).is_some());
        assert!(world.get::<BoltSpawnOffsetY>(entity).is_some());
        assert!(world.get::<BoltRespawnOffsetY>(entity).is_some());
        assert!(world.get::<BoltRespawnAngleSpread>(entity).is_some());
        assert!(world.get::<BoltInitialAngle>(entity).is_some());
    }

    #[test]
    fn init_values_match_config() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((Bolt, BoltVelocity::new(0.0, 400.0)))
            .id();

        app.update();

        let bolt_config = app.world().resource::<BoltConfig>();
        let world = app.world();
        assert!(
            (world.get::<BoltBaseSpeed>(entity).unwrap().0 - bolt_config.base_speed).abs()
                < f32::EPSILON
        );
        assert!(
            (world.get::<BoltRadius>(entity).unwrap().0 - bolt_config.radius).abs() < f32::EPSILON
        );
        assert!(
            (world.get::<BoltSpawnOffsetY>(entity).unwrap().0 - bolt_config.spawn_offset_y).abs()
                < f32::EPSILON
        );
        assert!(
            (world.get::<BoltRespawnOffsetY>(entity).unwrap().0 - bolt_config.respawn_offset_y)
                .abs()
                < f32::EPSILON
        );
        assert!(
            (world.get::<BoltRespawnAngleSpread>(entity).unwrap().0
                - bolt_config.respawn_angle_spread)
                .abs()
                < f32::EPSILON
        );
        assert!(
            (world.get::<BoltInitialAngle>(entity).unwrap().0 - bolt_config.initial_angle).abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn skips_already_initialized() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((Bolt, BoltVelocity::new(0.0, 400.0), BoltBaseSpeed(999.0)))
            .id();

        app.update();

        let base_speed = app.world().get::<BoltBaseSpeed>(entity).unwrap();
        assert!((base_speed.0 - 999.0).abs() < f32::EPSILON);
    }
}
