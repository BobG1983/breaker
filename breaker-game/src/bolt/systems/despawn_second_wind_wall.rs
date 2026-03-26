//! Despawns the [`SecondWindWall`] after a bolt hits it.
//!
//! Reads [`BoltHitWall`] messages and checks whether the wall entity has the
//! [`SecondWindWall`] marker. If so, despawns it — the wall has served its
//! single-bounce purpose.

use bevy::prelude::*;

use crate::{
    bolt::messages::BoltHitWall,
    effect::effects::second_wind::SecondWindWall,
};

/// Despawns the [`SecondWindWall`] entity when a bolt hits it.
///
/// Reads [`BoltHitWall`] messages each tick. If the wall entity in any message
/// has the [`SecondWindWall`] marker, it is despawned — the wall has served its
/// single-bounce purpose.
pub(crate) fn despawn_second_wind_wall(
    mut reader: MessageReader<BoltHitWall>,
    sw_query: Query<Entity, With<SecondWindWall>>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        // Only despawn if the hit wall IS the SecondWindWall.
        if sw_query.get(msg.wall).is_ok() {
            commands.entity(msg.wall).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::effects::second_wind::SecondWindWall;
    use crate::wall::components::Wall;

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // --- Message enqueue helper ---

    #[derive(Resource)]
    struct SendBoltHitWall(Option<BoltHitWall>);

    fn enqueue_bolt_hit_wall(
        msg_res: Res<SendBoltHitWall>,
        mut writer: MessageWriter<BoltHitWall>,
    ) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    fn test_app_with_message() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitWall>()
            .insert_resource(SendBoltHitWall(None))
            .add_systems(
                FixedUpdate,
                enqueue_bolt_hit_wall.before(despawn_second_wind_wall),
            )
            .add_systems(FixedUpdate, despawn_second_wind_wall);
        app
    }

    // =========================================================================
    // Part B: SecondWind wall despawn after bolt hit
    // =========================================================================

    #[test]
    fn second_wind_wall_despawns_after_bolt_hits_it() {
        // Given: SecondWindWall entity exists. BoltHitWall message targets it.
        // When: despawn_second_wind_wall runs
        // Then: SecondWindWall entity is despawned
        let mut app = test_app_with_message();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let sw_wall_entity = app
            .world_mut()
            .spawn((Wall, SecondWindWall))
            .id();

        app.insert_resource(SendBoltHitWall(Some(BoltHitWall {
            bolt: bolt_entity,
            wall: sw_wall_entity,
        })));

        tick(&mut app);

        // The SecondWindWall entity should be despawned
        let count = app
            .world_mut()
            .query_filtered::<Entity, With<SecondWindWall>>()
            .iter(app.world())
            .count();
        assert_eq!(
            count, 0,
            "SecondWindWall should be despawned after a bolt hits it"
        );
    }

    #[test]
    fn regular_wall_hit_does_not_despawn_second_wind_wall() {
        // Given: Regular Wall entity (no SecondWindWall). SecondWindWall exists elsewhere.
        //        BoltHitWall targets the regular wall.
        // When: despawn_second_wind_wall runs
        // Then: SecondWindWall is NOT despawned
        let mut app = test_app_with_message();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let regular_wall = app.world_mut().spawn(Wall).id();
        let _sw_wall = app
            .world_mut()
            .spawn((Wall, SecondWindWall))
            .id();

        app.insert_resource(SendBoltHitWall(Some(BoltHitWall {
            bolt: bolt_entity,
            wall: regular_wall,
        })));

        tick(&mut app);

        // The SecondWindWall entity should still exist
        let count = app
            .world_mut()
            .query_filtered::<Entity, With<SecondWindWall>>()
            .iter(app.world())
            .count();
        assert_eq!(
            count, 1,
            "SecondWindWall should NOT be despawned when a regular wall is hit"
        );
    }

    #[test]
    fn no_bolt_hit_wall_messages_leaves_second_wind_wall_alive() {
        // Given: SecondWindWall entity exists. No BoltHitWall messages.
        // When: despawn_second_wind_wall runs
        // Then: SecondWindWall remains alive
        let mut app = test_app_with_message();

        let _sw_wall = app
            .world_mut()
            .spawn((Wall, SecondWindWall))
            .id();

        // No message enqueued (SendBoltHitWall is None)
        tick(&mut app);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<SecondWindWall>>()
            .iter(app.world())
            .count();
        assert_eq!(
            count, 1,
            "SecondWindWall should remain alive when no BoltHitWall messages exist"
        );
    }
}
