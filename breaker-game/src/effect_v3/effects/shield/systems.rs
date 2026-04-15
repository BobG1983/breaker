//! Shield systems — tick duration countdown and reflection cost.

use bevy::prelude::*;

use super::components::{ShieldDuration, ShieldReflectionCost, ShieldWall};
use crate::prelude::*;

/// Decrements shield duration each frame and despawns expired shields.
pub fn tick_shield_duration(
    mut query: Query<(Entity, &mut ShieldDuration), With<ShieldWall>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    for (entity, mut duration) in &mut query {
        duration.0 -= dt;
        if duration.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Subtracts reflection cost from shield duration for each bolt impact.
///
/// For each `BoltImpactWall` message targeting a shield entity, subtracts
/// `ShieldReflectionCost` from `ShieldDuration`. Does NOT despawn — the
/// `tick_shield_duration` system handles despawn on the next frame.
pub(crate) fn apply_shield_reflection_cost(
    mut reader: MessageReader<BoltImpactWall>,
    mut query: Query<(&mut ShieldDuration, &ShieldReflectionCost), With<ShieldWall>>,
) {
    for msg in reader.read() {
        if let Ok((mut duration, cost)) = query.get_mut(msg.wall) {
            duration.0 -= cost.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;
    use crate::effect_v3::effects::shield::components::{
        ShieldDuration, ShieldReflectionCost, ShieldWall,
    };

    // -- Helpers ----------------------------------------------------------

    /// Resource to inject `BoltImpactWall` messages into the test app.
    #[derive(Resource, Default)]
    struct TestBoltImpactWallMessages(Vec<BoltImpactWall>);

    /// System that writes `BoltImpactWall` messages from the test resource.
    fn inject_impacts(
        messages: Res<TestBoltImpactWallMessages>,
        mut writer: MessageWriter<BoltImpactWall>,
    ) {
        for msg in &messages.0 {
            writer.write(msg.clone());
        }
    }

    fn shield_cost_app() -> App {
        TestAppBuilder::new()
            .with_message::<BoltImpactWall>()
            .with_resource::<TestBoltImpactWallMessages>()
            .with_system(
                FixedUpdate,
                (
                    inject_impacts.before(apply_shield_reflection_cost),
                    apply_shield_reflection_cost,
                ),
            )
            .build()
    }

    fn spawn_shield(app: &mut App, duration: f32, cost: f32) -> Entity {
        app.world_mut()
            .spawn((
                ShieldWall,
                ShieldDuration(duration),
                ShieldReflectionCost(cost),
            ))
            .id()
    }

    // ── C13-1: Bolt impact subtracts reflection cost from duration ──

    #[test]
    fn bolt_impact_subtracts_reflection_cost() {
        let mut app = shield_cost_app();

        let shield = spawn_shield(&mut app, 5.0, 0.5);
        let bolt = app.world_mut().spawn_empty().id();

        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0
            .push(BoltImpactWall { bolt, wall: shield });

        tick(&mut app);

        let duration = app.world().get::<ShieldDuration>(shield).unwrap();
        assert!(
            (duration.0 - 4.5).abs() < f32::EPSILON,
            "duration should be 4.5 (5.0 - 0.5), got {}",
            duration.0,
        );
    }

    #[test]
    fn zero_reflection_cost_is_noop() {
        let mut app = shield_cost_app();

        let shield = spawn_shield(&mut app, 5.0, 0.0);
        let bolt = app.world_mut().spawn_empty().id();

        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0
            .push(BoltImpactWall { bolt, wall: shield });

        tick(&mut app);

        let duration = app.world().get::<ShieldDuration>(shield).unwrap();
        assert!(
            (duration.0 - 5.0).abs() < f32::EPSILON,
            "duration should remain 5.0 with zero cost, got {}",
            duration.0,
        );
    }

    // ── C13-2: Multiple impacts each subtract cost ──

    #[test]
    fn three_impacts_subtract_three_times_cost() {
        let mut app = shield_cost_app();

        let shield = spawn_shield(&mut app, 5.0, 1.0);
        let bolt = app.world_mut().spawn_empty().id();

        {
            let mut msgs = app.world_mut().resource_mut::<TestBoltImpactWallMessages>();
            for _ in 0..3 {
                msgs.0.push(BoltImpactWall { bolt, wall: shield });
            }
        }

        tick(&mut app);

        let duration = app.world().get::<ShieldDuration>(shield).unwrap();
        assert!(
            (duration.0 - 2.0).abs() < f32::EPSILON,
            "duration should be 2.0 (5.0 - 3*1.0), got {}",
            duration.0,
        );
    }

    #[test]
    fn impacts_can_drive_duration_below_zero() {
        let mut app = shield_cost_app();

        let shield = spawn_shield(&mut app, 1.0, 1.0);
        let bolt = app.world_mut().spawn_empty().id();

        {
            let mut msgs = app.world_mut().resource_mut::<TestBoltImpactWallMessages>();
            for _ in 0..3 {
                msgs.0.push(BoltImpactWall { bolt, wall: shield });
            }
        }

        tick(&mut app);

        let duration = app.world().get::<ShieldDuration>(shield).unwrap();
        assert!(
            (duration.0 - (-2.0)).abs() < f32::EPSILON,
            "duration should be -2.0 (1.0 - 3*1.0), got {}",
            duration.0,
        );
    }

    // ── C13-3: Impact on non-shield wall is ignored ──

    #[test]
    fn impact_on_non_shield_wall_is_ignored() {
        let mut app = shield_cost_app();

        let shield = spawn_shield(&mut app, 5.0, 0.5);
        let regular_wall = app.world_mut().spawn_empty().id();
        let bolt = app.world_mut().spawn_empty().id();

        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0
            .push(BoltImpactWall {
                bolt,
                wall: regular_wall,
            });

        tick(&mut app);

        let duration = app.world().get::<ShieldDuration>(shield).unwrap();
        assert!(
            (duration.0 - 5.0).abs() < f32::EPSILON,
            "shield duration should remain 5.0 when non-shield wall is hit, got {}",
            duration.0,
        );
    }

    // ── C13-4: No messages — duration unchanged ──

    #[test]
    fn no_impacts_leaves_duration_unchanged() {
        let mut app = shield_cost_app();

        let shield = spawn_shield(&mut app, 5.0, 0.5);

        tick(&mut app);

        let duration = app.world().get::<ShieldDuration>(shield).unwrap();
        assert!(
            (duration.0 - 5.0).abs() < f32::EPSILON,
            "duration should remain 5.0 with no impacts, got {}",
            duration.0,
        );
    }

    #[test]
    fn impact_on_nonexistent_shield_does_not_panic() {
        let mut app = shield_cost_app();

        let regular_wall = app.world_mut().spawn_empty().id();
        let bolt = app.world_mut().spawn_empty().id();

        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0
            .push(BoltImpactWall {
                bolt,
                wall: regular_wall,
            });

        // Should not panic — no shield entities exist
        tick(&mut app);
    }

    // ── C13-5: Multiple shields only subtract from the hit one ──

    #[test]
    fn only_hit_shield_loses_duration() {
        let mut app = shield_cost_app();

        let shield_a = spawn_shield(&mut app, 5.0, 0.5);
        let shield_b = spawn_shield(&mut app, 3.0, 1.0);
        let bolt = app.world_mut().spawn_empty().id();

        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0
            .push(BoltImpactWall {
                bolt,
                wall: shield_a,
            });

        tick(&mut app);

        let duration_a = app.world().get::<ShieldDuration>(shield_a).unwrap();
        assert!(
            (duration_a.0 - 4.5).abs() < f32::EPSILON,
            "shield A duration should be 4.5, got {}",
            duration_a.0,
        );

        let duration_b = app.world().get::<ShieldDuration>(shield_b).unwrap();
        assert!(
            (duration_b.0 - 3.0).abs() < f32::EPSILON,
            "shield B duration should remain 3.0, got {}",
            duration_b.0,
        );
    }

    // ── C13-6: Reflection cost drives duration to exactly zero ──

    #[test]
    fn reflection_cost_drives_duration_to_exactly_zero() {
        let mut app = shield_cost_app();

        let shield = spawn_shield(&mut app, 0.5, 0.5);
        let bolt = app.world_mut().spawn_empty().id();

        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0
            .push(BoltImpactWall { bolt, wall: shield });

        tick(&mut app);

        let duration = app.world().get::<ShieldDuration>(shield).unwrap();
        assert!(
            duration.0.abs() < f32::EPSILON,
            "duration should be exactly 0.0, got {}",
            duration.0,
        );
    }
}
