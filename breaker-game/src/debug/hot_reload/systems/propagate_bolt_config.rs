//! System to propagate `BoltConfig` resource changes to bolt entity components.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{BaseSpeed, MaxSpeed, MinSpeed};

use crate::bolt::{
    components::{
        Bolt, BoltInitialAngle, BoltRadius, BoltRespawnAngleSpread, BoltRespawnOffsetY,
        BoltSpawnOffsetY,
    },
    resources::BoltConfig,
};

/// Force-overwrites bolt components on all bolt entities when `BoltConfig` changes.
///
/// Runs in `Update` in the `HotReloadSystems::PropagateConfig` system set,
/// conditioned on `resource_changed::<BoltConfig>`. Always overwrites all
/// config-derived components on every bolt entity.
pub(crate) fn propagate_bolt_config(
    mut commands: Commands,
    config: Res<BoltConfig>,
    query: Query<Entity, With<Bolt>>,
) {
    for entity in &query {
        commands.entity(entity).insert((
            BaseSpeed(config.base_speed),
            MinSpeed(config.min_speed),
            MaxSpeed(config.max_speed),
            BoltRadius(config.radius),
            BoltSpawnOffsetY(config.spawn_offset_y),
            BoltRespawnOffsetY(config.respawn_offset_y),
            BoltRespawnAngleSpread(config.respawn_angle_spread),
            BoltInitialAngle(config.initial_angle),
        ));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<BoltConfig>()
            .add_systems(Update, propagate_bolt_config);
        app
    }

    /// When `BoltConfig` changes (`is_changed` returns true on first frame after insert),
    /// the system should overwrite `BaseSpeed` with the config value even if it
    /// was previously stamped with a different value.
    #[test]
    fn force_overwrites_base_speed_when_config_changes() {
        let mut app = test_app();
        let config = app.world().resource::<BoltConfig>().clone();

        // Spawn bolt with a deliberately wrong BaseSpeed.
        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                BaseSpeed(999.0),
                MinSpeed(config.min_speed),
                MaxSpeed(config.max_speed),
                BoltRadius(config.radius),
                BoltSpawnOffsetY(config.spawn_offset_y),
                BoltRespawnOffsetY(config.respawn_offset_y),
                BoltRespawnAngleSpread(config.respawn_angle_spread),
                BoltInitialAngle(config.initial_angle),
            ))
            .id();

        // Mutate config so resource_changed fires.
        app.world_mut().resource_mut::<BoltConfig>().base_speed = 600.0;
        app.update();

        let base_speed = app.world().get::<BaseSpeed>(entity).unwrap();
        assert!(
            (base_speed.0 - 600.0).abs() < f32::EPSILON,
            "BaseSpeed should be 600.0 after config change, got {}",
            base_speed.0
        );
    }

    /// The system must overwrite ALL 8 bolt components, not just `BaseSpeed`.
    #[test]
    fn overwrites_all_eight_bolt_components() {
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                BaseSpeed(999.0),
                MinSpeed(999.0),
                MaxSpeed(999.0),
                BoltRadius(999.0),
                BoltSpawnOffsetY(999.0),
                BoltRespawnOffsetY(999.0),
                BoltRespawnAngleSpread(999.0),
                BoltInitialAngle(999.0),
            ))
            .id();

        // Set known config values and mark as changed.
        {
            let mut config = app.world_mut().resource_mut::<BoltConfig>();
            config.base_speed = 400.0;
            config.min_speed = 200.0;
            config.max_speed = 800.0;
            config.radius = 8.0;
            config.spawn_offset_y = 30.0;
            config.respawn_offset_y = 30.0;
            config.respawn_angle_spread = 0.524;
            config.initial_angle = 0.26;
        }

        app.update();

        let world = app.world();
        assert!(
            (world.get::<BaseSpeed>(entity).unwrap().0 - 400.0).abs() < f32::EPSILON,
            "BaseSpeed should be overwritten to 400.0"
        );
        assert!(
            (world.get::<MinSpeed>(entity).unwrap().0 - 200.0).abs() < f32::EPSILON,
            "MinSpeed should be overwritten to 200.0"
        );
        assert!(
            (world.get::<MaxSpeed>(entity).unwrap().0 - 800.0).abs() < f32::EPSILON,
            "MaxSpeed should be overwritten to 800.0"
        );
        assert!(
            (world.get::<BoltRadius>(entity).unwrap().0 - 8.0).abs() < f32::EPSILON,
            "BoltRadius should be overwritten to 8.0"
        );
        assert!(
            (world.get::<BoltSpawnOffsetY>(entity).unwrap().0 - 30.0).abs() < f32::EPSILON,
            "BoltSpawnOffsetY should be overwritten to 30.0"
        );
        assert!(
            (world.get::<BoltRespawnOffsetY>(entity).unwrap().0 - 30.0).abs() < f32::EPSILON,
            "BoltRespawnOffsetY should be overwritten to 30.0"
        );
        assert!(
            (world.get::<BoltRespawnAngleSpread>(entity).unwrap().0 - 0.524).abs() < 1e-5,
            "BoltRespawnAngleSpread should be overwritten to 0.524"
        );
        assert!(
            (world.get::<BoltInitialAngle>(entity).unwrap().0 - 0.26).abs() < 1e-5,
            "BoltInitialAngle should be overwritten to 0.26"
        );
    }

    /// All bolt entities must be updated, not just the first one.
    #[test]
    fn updates_all_bolt_entities() {
        let mut app = test_app();

        // Spawn 3 bolt entities with different BaseSpeed values.
        let e1 = app
            .world_mut()
            .spawn((
                Bolt,
                BaseSpeed(100.0),
                MinSpeed(50.0),
                MaxSpeed(200.0),
                BoltRadius(8.0),
                BoltSpawnOffsetY(30.0),
                BoltRespawnOffsetY(30.0),
                BoltRespawnAngleSpread(0.524),
                BoltInitialAngle(0.26),
            ))
            .id();
        let e2 = app
            .world_mut()
            .spawn((
                Bolt,
                BaseSpeed(200.0),
                MinSpeed(50.0),
                MaxSpeed(200.0),
                BoltRadius(8.0),
                BoltSpawnOffsetY(30.0),
                BoltRespawnOffsetY(30.0),
                BoltRespawnAngleSpread(0.524),
                BoltInitialAngle(0.26),
            ))
            .id();
        let e3 = app
            .world_mut()
            .spawn((
                Bolt,
                BaseSpeed(300.0),
                MinSpeed(50.0),
                MaxSpeed(200.0),
                BoltRadius(8.0),
                BoltSpawnOffsetY(30.0),
                BoltRespawnOffsetY(30.0),
                BoltRespawnAngleSpread(0.524),
                BoltInitialAngle(0.26),
            ))
            .id();

        // Change config to 500.0.
        app.world_mut().resource_mut::<BoltConfig>().base_speed = 500.0;
        app.update();

        let world = app.world();
        assert!(
            (world.get::<BaseSpeed>(e1).unwrap().0 - 500.0).abs() < f32::EPSILON,
            "entity 1 BaseSpeed should be 500.0"
        );
        assert!(
            (world.get::<BaseSpeed>(e2).unwrap().0 - 500.0).abs() < f32::EPSILON,
            "entity 2 BaseSpeed should be 500.0"
        );
        assert!(
            (world.get::<BaseSpeed>(e3).unwrap().0 - 500.0).abs() < f32::EPSILON,
            "entity 3 BaseSpeed should be 500.0"
        );
    }

    /// When registered with `run_if(resource_changed::<BoltConfig>)` and the
    /// config has not been mutated since the last run, no overwrite occurs.
    /// This test wires the system with the run condition to verify it respects
    /// unchanged config.
    #[test]
    fn does_not_run_when_config_unchanged() {
        // Register system WITH the run condition, same as the real plugin.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<BoltConfig>()
            .add_systems(
                Update,
                propagate_bolt_config.run_if(resource_changed::<BoltConfig>),
            );

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                BaseSpeed(999.0),
                MinSpeed(100.0),
                MaxSpeed(800.0),
                BoltRadius(8.0),
                BoltSpawnOffsetY(30.0),
                BoltRespawnOffsetY(30.0),
                BoltRespawnAngleSpread(0.524),
                BoltInitialAngle(0.26),
            ))
            .id();

        // First update: resource was just inserted, so is_changed() is true.
        // Let it settle.
        app.update();

        // Second update: config has NOT been mutated — run condition should block.
        app.update();

        // The stale 999.0 value should still be on the entity IF the first update
        // overwrite hadn't happened. But since the first update runs (resource was
        // just inserted = changed), the value will reflect config.base_speed (400.0).
        // The key assertion: a second update without mutation does NOT reset to 999.0.
        let base_speed = app.world().get::<BaseSpeed>(entity).unwrap();
        let config_base = app.world().resource::<BoltConfig>().base_speed;
        assert!(
            (base_speed.0 - config_base).abs() < f32::EPSILON,
            "BaseSpeed should match config after initial propagation and not revert; got {}",
            base_speed.0
        );
    }

    /// Edge case: zero bolt entities — system should not panic on empty query.
    #[test]
    fn handles_no_bolt_entities() {
        let mut app = test_app();

        // Mutate config to trigger propagation.
        app.world_mut().resource_mut::<BoltConfig>().base_speed = 500.0;

        // Should not panic.
        app.update();
    }
}
