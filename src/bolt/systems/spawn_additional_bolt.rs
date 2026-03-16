//! System to spawn additional bolt entities from archetype consequences.

use bevy::prelude::*;
use rand::Rng;

use crate::{
    bolt::{
        components::{
            Bolt, BoltBaseSpeed, BoltInitialAngle, BoltMaxSpeed, BoltMinSpeed, BoltRadius,
            BoltRespawnAngleSpread, BoltRespawnOffsetY, BoltSpawnOffsetY, BoltVelocity, ExtraBolt,
        },
        messages::SpawnAdditionalBolt,
        resources::BoltConfig,
    },
    breaker::components::Breaker,
    shared::{CleanupOnNodeExit, GameRng},
};

/// Reads [`SpawnAdditionalBolt`] messages and spawns new bolt entities.
///
/// Each bolt spawns above the breaker with a randomized upward velocity
/// at base speed. The bolt is marked [`ExtraBolt`] so it despawns on loss
/// rather than respawning.
pub fn spawn_additional_bolt(
    mut commands: Commands,
    mut reader: MessageReader<SpawnAdditionalBolt>,
    bolt_config: Res<BoltConfig>,
    mut rng: ResMut<GameRng>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    breaker_query: Query<&Transform, With<Breaker>>,
) {
    let Ok(breaker_tf) = breaker_query.single() else {
        return;
    };
    let breaker_pos = breaker_tf.translation;

    for _msg in reader.read() {
        let angle = rng
            .0
            .random_range(-bolt_config.respawn_angle_spread..=bolt_config.respawn_angle_spread);
        let velocity = BoltVelocity::new(
            bolt_config.base_speed * angle.sin(),
            bolt_config.base_speed * angle.cos(),
        );

        let spawn_pos = Vec3::new(
            breaker_pos.x,
            breaker_pos.y + bolt_config.spawn_offset_y,
            1.0,
        );

        commands.spawn((
            Bolt,
            ExtraBolt,
            velocity,
            BoltBaseSpeed(bolt_config.base_speed),
            BoltMinSpeed(bolt_config.min_speed),
            BoltMaxSpeed(bolt_config.max_speed),
            BoltRadius(bolt_config.radius),
            BoltSpawnOffsetY(bolt_config.spawn_offset_y),
            BoltRespawnOffsetY(bolt_config.respawn_offset_y),
            BoltRespawnAngleSpread(bolt_config.respawn_angle_spread),
            BoltInitialAngle(bolt_config.initial_angle),
            Mesh2d(meshes.add(Circle::new(1.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(bolt_config.color()))),
            Transform {
                translation: spawn_pos,
                scale: Vec3::new(bolt_config.radius, bolt_config.radius, 1.0),
                ..default()
            },
            CleanupOnNodeExit,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct SendSpawn(u32);

    fn send_spawn(flag: Res<SendSpawn>, mut writer: MessageWriter<SpawnAdditionalBolt>) {
        for _ in 0..flag.0 {
            writer.write(SpawnAdditionalBolt);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BoltConfig>();
        app.init_resource::<GameRng>();
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();
        app.add_message::<SpawnAdditionalBolt>();
        app.insert_resource(SendSpawn(0));
        app.add_systems(FixedUpdate, (send_spawn, spawn_additional_bolt).chain());
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn spawn_breaker(app: &mut App) {
        app.world_mut()
            .spawn((Breaker, Transform::from_xyz(0.0, -250.0, 0.0)));
    }

    #[test]
    fn creates_new_bolt_entity() {
        let mut app = test_app();
        spawn_breaker(&mut app);
        // Pre-existing baseline bolt
        app.world_mut().spawn((Bolt, BoltVelocity::new(0.0, 400.0)));
        app.world_mut().resource_mut::<SendSpawn>().0 = 1;
        tick(&mut app);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<Bolt>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 2, "should have baseline + 1 additional bolt");
    }

    #[test]
    fn new_bolt_has_extra_bolt_marker() {
        let mut app = test_app();
        spawn_breaker(&mut app);
        app.world_mut().resource_mut::<SendSpawn>().0 = 1;
        tick(&mut app);

        let extra_count = app
            .world_mut()
            .query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()
            .iter(app.world())
            .count();
        assert_eq!(extra_count, 1);
    }

    #[test]
    fn new_bolt_has_all_components() {
        let mut app = test_app();
        spawn_breaker(&mut app);
        app.world_mut().resource_mut::<SendSpawn>().0 = 1;
        tick(&mut app);

        let entity = app
            .world_mut()
            .query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()
            .iter(app.world())
            .next()
            .expect("extra bolt should exist");

        let world = app.world();
        assert!(world.get::<BoltVelocity>(entity).is_some());
        assert!(world.get::<BoltBaseSpeed>(entity).is_some());
        assert!(world.get::<BoltMinSpeed>(entity).is_some());
        assert!(world.get::<BoltMaxSpeed>(entity).is_some());
        assert!(world.get::<BoltRadius>(entity).is_some());
        assert!(world.get::<BoltRespawnOffsetY>(entity).is_some());
        assert!(world.get::<BoltRespawnAngleSpread>(entity).is_some());
        assert!(world.get::<CleanupOnNodeExit>(entity).is_some());
    }

    #[test]
    fn new_bolt_launches_upward() {
        let mut app = test_app();
        spawn_breaker(&mut app);
        app.world_mut().resource_mut::<SendSpawn>().0 = 1;
        tick(&mut app);

        let vel = app
            .world_mut()
            .query_filtered::<&BoltVelocity, With<ExtraBolt>>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(vel.value.y > 0.0, "additional bolt should launch upward");
    }

    #[test]
    fn new_bolt_at_base_speed() {
        let mut app = test_app();
        spawn_breaker(&mut app);
        app.world_mut().resource_mut::<SendSpawn>().0 = 1;
        tick(&mut app);

        let config = app.world().resource::<BoltConfig>();
        let base_speed = config.base_speed;

        let vel = app
            .world_mut()
            .query_filtered::<&BoltVelocity, With<ExtraBolt>>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            (vel.speed() - base_speed).abs() < 1.0,
            "speed {:.1} should equal base_speed {base_speed:.1}",
            vel.speed()
        );
    }

    #[test]
    fn new_bolt_above_breaker() {
        let mut app = test_app();
        let breaker_y = -250.0;
        app.world_mut()
            .spawn((Breaker, Transform::from_xyz(0.0, breaker_y, 0.0)));
        app.world_mut().resource_mut::<SendSpawn>().0 = 1;
        tick(&mut app);

        let tf = app
            .world_mut()
            .query_filtered::<&Transform, With<ExtraBolt>>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            tf.translation.y > breaker_y,
            "bolt Y {:.1} should be above breaker Y {breaker_y:.1}",
            tf.translation.y
        );
    }

    #[test]
    fn new_bolt_has_cleanup_marker() {
        let mut app = test_app();
        spawn_breaker(&mut app);
        app.world_mut().resource_mut::<SendSpawn>().0 = 1;
        tick(&mut app);

        let entity = app
            .world_mut()
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(app.world().get::<CleanupOnNodeExit>(entity).is_some());
    }

    #[test]
    fn no_message_no_spawn() {
        let mut app = test_app();
        spawn_breaker(&mut app);
        tick(&mut app);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<Bolt>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "no bolt should spawn without message");
    }

    #[test]
    fn multiple_messages_spawn_multiple() {
        let mut app = test_app();
        spawn_breaker(&mut app);
        app.world_mut().resource_mut::<SendSpawn>().0 = 2;
        tick(&mut app);

        let count = app
            .world_mut()
            .query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()
            .iter(app.world())
            .count();
        assert_eq!(count, 2, "2 messages should spawn 2 extra bolts");
    }
}
