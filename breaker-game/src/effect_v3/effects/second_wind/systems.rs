//! Second-wind systems — despawn on first reflection.

use bevy::prelude::*;

use super::components::SecondWindWall;
use crate::bolt::messages::BoltImpactWall;

/// Despawns a `SecondWindWall` entity on the first `BoltImpactWall` targeting it.
///
/// Reads `BoltImpactWall` messages and despawns any message target that matches
/// `With<SecondWindWall>`. Second messages targeting an already-despawned wall
/// (same frame or subsequent frames) are silent no-ops.
pub(crate) fn despawn_on_first_reflection(
    mut reader: MessageReader<BoltImpactWall>,
    query: Query<Entity, With<SecondWindWall>>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if query.get(msg.wall).is_ok() {
            commands.entity(msg.wall).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;
    use crate::{
        bolt::messages::BoltImpactWall,
        effect_v3::{
            effects::second_wind::{
                components::{SecondWindOwner, SecondWindWall},
                config::SecondWindConfig,
            },
            traits::Fireable,
        },
        shared::test_utils::{TestAppBuilder, tick},
    };

    // ── Helpers ─────────────────────────────────────────────────────────────

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

    fn second_wind_app() -> App {
        TestAppBuilder::new()
            .with_message::<BoltImpactWall>()
            .with_resource::<TestBoltImpactWallMessages>()
            .with_system(
                FixedUpdate,
                (
                    inject_impacts.before(despawn_on_first_reflection),
                    despawn_on_first_reflection,
                ),
            )
            .build()
    }

    fn spawn_second_wind_wall(app: &mut App, owner: Entity) -> Entity {
        app.world_mut()
            .spawn((SecondWindWall, SecondWindOwner(owner)))
            .id()
    }

    // ── Behavior 12: first BoltImpactWall on a SecondWind wall despawns it ──

    #[test]
    fn first_bolt_impact_despawns_second_wind_wall() {
        let mut app = second_wind_app();

        let owner = app.world_mut().spawn_empty().id();
        let bolt = app.world_mut().spawn_empty().id();
        let wall = spawn_second_wind_wall(&mut app, owner);

        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0
            .push(BoltImpactWall { bolt, wall });

        tick(&mut app);

        assert!(
            app.world().get_entity(wall).is_err(),
            "second-wind wall should be despawned on first impact",
        );
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<SecondWindWall>>();
        let remaining = query.iter(app.world()).count();
        assert_eq!(
            remaining, 0,
            "no SecondWindWall entities should remain, got {remaining}",
        );
    }

    #[test]
    fn first_bolt_impact_does_not_despawn_owner_or_bolt() {
        let mut app = second_wind_app();

        let owner = app.world_mut().spawn_empty().id();
        let bolt = app.world_mut().spawn_empty().id();
        let wall = spawn_second_wind_wall(&mut app, owner);

        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0
            .push(BoltImpactWall { bolt, wall });

        tick(&mut app);

        assert!(
            app.world().get_entity(owner).is_ok(),
            "owner entity must not be despawned",
        );
        assert!(
            app.world().get_entity(bolt).is_ok(),
            "bolt entity must not be despawned",
        );
    }

    // ── Behavior 13: second BoltImpactWall on a despawned wall is a no-op ──

    #[test]
    fn second_impact_on_despawned_wall_is_noop() {
        let mut app = second_wind_app();

        let owner = app.world_mut().spawn_empty().id();
        let bolt = app.world_mut().spawn_empty().id();
        let wall = spawn_second_wind_wall(&mut app, owner);

        // First tick: push and despawn.
        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0
            .push(BoltImpactWall { bolt, wall });
        tick(&mut app);

        // Confirm the wall is gone.
        assert!(
            app.world().get_entity(wall).is_err(),
            "wall should be despawned after first tick",
        );

        // Clear the injection queue so the first (stale) message is not re-injected.
        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0
            .clear();

        // Push a second message targeting the now-despawned wall.
        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0
            .push(BoltImpactWall { bolt, wall });

        // Second tick must not panic.
        tick(&mut app);

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<SecondWindWall>>();
        let remaining = query.iter(app.world()).count();
        assert_eq!(
            remaining, 0,
            "no SecondWindWall entities should exist after second tick, got {remaining}",
        );
    }

    #[test]
    fn two_impacts_in_same_tick_do_not_panic() {
        let mut app = second_wind_app();

        let owner = app.world_mut().spawn_empty().id();
        let bolt = app.world_mut().spawn_empty().id();
        let wall = spawn_second_wind_wall(&mut app, owner);

        {
            let mut msgs = app.world_mut().resource_mut::<TestBoltImpactWallMessages>();
            msgs.0.push(BoltImpactWall { bolt, wall });
            msgs.0.push(BoltImpactWall { bolt, wall });
        }

        // Must not panic.
        tick(&mut app);

        assert!(
            app.world().get_entity(wall).is_err(),
            "wall should be despawned on first-of-two impacts",
        );
    }

    // ── Behavior 14: impact targeting a non-second-wind wall is ignored ────

    #[test]
    fn impact_on_non_second_wind_wall_is_ignored() {
        // This test ensures that the despawn system IS correctly registered
        // (so that it runs at all), and that when it runs on a non-second-wind
        // wall it does NOT despawn anything. We verify both halves: first a
        // baseline hit on the second-wind wall (which MUST despawn) confirms
        // the system is wired, then a separate hit on a regular wall confirms
        // it ignores non-marked entities.
        let mut app = second_wind_app();

        let owner = app.world_mut().spawn_empty().id();
        let bolt = app.world_mut().spawn_empty().id();
        let second_wind_baseline = spawn_second_wind_wall(&mut app, owner);

        // Baseline: hitting a second-wind wall must despawn it (ensures the
        // system is actually running). Against today's stub this fails.
        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0
            .push(BoltImpactWall {
                bolt,
                wall: second_wind_baseline,
            });
        tick(&mut app);
        assert!(
            app.world().get_entity(second_wind_baseline).is_err(),
            "baseline: second-wind wall should be despawned",
        );

        // Clear the injection queue.
        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0
            .clear();

        // Now the real assertion: hitting a non-second-wind wall should leave
        // another second-wind wall (and the regular wall) alone.
        let second_wind = spawn_second_wind_wall(&mut app, owner);
        let regular_wall = app.world_mut().spawn_empty().id();

        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0
            .push(BoltImpactWall {
                bolt,
                wall: regular_wall,
            });

        tick(&mut app);

        assert!(
            app.world().get_entity(second_wind).is_ok(),
            "second-wind wall should still be alive",
        );
        assert!(
            app.world().get_entity(regular_wall).is_ok(),
            "regular wall should still be alive (system must not touch non-marked entities)",
        );
    }

    #[test]
    fn mixed_impacts_only_despawn_second_wind_wall() {
        let mut app = second_wind_app();

        let owner = app.world_mut().spawn_empty().id();
        let bolt = app.world_mut().spawn_empty().id();
        let second_wind = spawn_second_wind_wall(&mut app, owner);
        let regular_wall = app.world_mut().spawn_empty().id();

        {
            let mut msgs = app.world_mut().resource_mut::<TestBoltImpactWallMessages>();
            msgs.0.push(BoltImpactWall {
                bolt,
                wall: regular_wall,
            });
            msgs.0.push(BoltImpactWall {
                bolt,
                wall: second_wind,
            });
        }

        tick(&mut app);

        assert!(
            app.world().get_entity(second_wind).is_err(),
            "second-wind wall should be despawned",
        );
        assert!(
            app.world().get_entity(regular_wall).is_ok(),
            "regular wall should remain alive",
        );
    }

    // ── Behavior 15: multi-owner isolation ─────────────────────────────────

    #[test]
    fn multi_owner_isolation_other_wall_survives() {
        let mut app = second_wind_app();

        let owner_a = app.world_mut().spawn_empty().id();
        let owner_b = app.world_mut().spawn_empty().id();
        let bolt = app.world_mut().spawn_empty().id();
        let wall_a = spawn_second_wind_wall(&mut app, owner_a);
        let wall_b = spawn_second_wind_wall(&mut app, owner_b);

        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0
            .push(BoltImpactWall { bolt, wall: wall_a });

        tick(&mut app);

        assert!(
            app.world().get_entity(wall_a).is_err(),
            "wall_a should be despawned",
        );
        assert!(
            app.world().get_entity(wall_b).is_ok(),
            "wall_b should still be alive",
        );
        let b_owner = app.world().get::<SecondWindOwner>(wall_b);
        assert!(
            b_owner.is_some(),
            "wall_b should still have SecondWindOwner"
        );
        assert_eq!(b_owner.unwrap().0, owner_b);

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<SecondWindWall>>();
        let remaining: Vec<Entity> = query.iter(app.world()).collect();
        assert_eq!(
            remaining.len(),
            1,
            "exactly 1 SecondWindWall should remain, got {}",
            remaining.len(),
        );
        assert_eq!(remaining[0], wall_b, "the surviving wall should be wall_b");
    }

    #[test]
    fn multi_owner_double_hit_does_not_affect_other_wall() {
        let mut app = second_wind_app();

        let owner_a = app.world_mut().spawn_empty().id();
        let owner_b = app.world_mut().spawn_empty().id();
        let bolt = app.world_mut().spawn_empty().id();
        let wall_a = spawn_second_wind_wall(&mut app, owner_a);
        let wall_b = spawn_second_wind_wall(&mut app, owner_b);

        {
            let mut msgs = app.world_mut().resource_mut::<TestBoltImpactWallMessages>();
            msgs.0.push(BoltImpactWall { bolt, wall: wall_a });
            msgs.0.push(BoltImpactWall { bolt, wall: wall_a });
        }

        tick(&mut app);

        assert!(
            app.world().get_entity(wall_a).is_err(),
            "wall_a should be despawned",
        );
        assert!(
            app.world().get_entity(wall_b).is_ok(),
            "wall_b should still be alive",
        );
    }

    // ── Behavior 16: register() wires despawn_on_first_reflection ──────────

    #[test]
    fn register_wires_despawn_into_fixed_update() {
        // Positive branch: register() should wire despawn_on_first_reflection into
        // FixedUpdate / EffectV3Systems::Tick. This test builds the app WITHOUT
        // manually adding despawn_on_first_reflection via `.with_system(...)` —
        // the ONLY way the wall gets despawned is if register() wired it itself.
        let mut app = TestAppBuilder::new()
            .with_message::<BoltImpactWall>()
            .with_resource::<TestBoltImpactWallMessages>()
            .with_system(FixedUpdate, inject_impacts)
            .build();

        <SecondWindConfig as Fireable>::register(&mut app);

        let owner = app.world_mut().spawn_empty().id();
        let bolt = app.world_mut().spawn_empty().id();
        let wall = app
            .world_mut()
            .spawn((SecondWindWall, SecondWindOwner(owner)))
            .id();

        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0
            .push(BoltImpactWall { bolt, wall });

        // Tick twice to absorb any system ordering race between inject_impacts
        // and despawn_on_first_reflection — messages in Bevy survive 2 frames.
        tick(&mut app);
        tick(&mut app);

        assert!(
            app.world().get_entity(wall).is_err(),
            "wall should be despawned — register() must wire despawn_on_first_reflection \
             into FixedUpdate",
        );
    }

    #[test]
    fn no_register_leaves_wall_alive() {
        // Negative branch: same wiring EXCEPT register() is NOT called and
        // despawn_on_first_reflection is NOT manually added. The wall must survive.
        let mut app = TestAppBuilder::new()
            .with_message::<BoltImpactWall>()
            .with_resource::<TestBoltImpactWallMessages>()
            .with_system(FixedUpdate, inject_impacts)
            .build();

        // Intentionally do NOT call <SecondWindConfig as Fireable>::register(&mut app).

        let owner = app.world_mut().spawn_empty().id();
        let bolt = app.world_mut().spawn_empty().id();
        let wall = app
            .world_mut()
            .spawn((SecondWindWall, SecondWindOwner(owner)))
            .id();

        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0
            .push(BoltImpactWall { bolt, wall });

        // Tick twice to match the positive branch exactly.
        tick(&mut app);
        tick(&mut app);

        assert!(
            app.world().get_entity(wall).is_ok(),
            "without register() the despawn system should not run — wall must remain alive",
        );
    }
}
