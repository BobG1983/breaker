//! Per-trigger bridge systems — translate messages into consequence events.

use bevy::prelude::*;

use super::{
    active::ActiveBehaviors,
    definition::{Consequence, ConsequenceFired, Trigger},
};
use crate::{
    breaker::messages::{BumpGrade, BumpPerformed, BumpWhiffed},
    physics::messages::BoltLost,
};

/// Reads `BoltLost` messages and fires consequence events for that trigger.
///
/// Drains all messages (not just the first) so that extras from future multi-bolt
/// archetypes don't leak into subsequent frames. Consequences fire once per
/// bridge invocation regardless of how many bolts were lost in the same frame.
pub(crate) fn bridge_bolt_lost(
    mut reader: MessageReader<BoltLost>,
    bindings: Res<ActiveBehaviors>,
    mut commands: Commands,
) {
    if reader.read().count() == 0 {
        return;
    }
    fire_consequences(&bindings, Trigger::BoltLost, &mut commands);
}

/// Reads `BumpWhiffed` messages and fires consequence events for that trigger.
///
/// Drains all messages (not just the first) so that extras don't leak into
/// subsequent frames. Consequences fire once per bridge invocation.
pub(crate) fn bridge_bump_whiff(
    mut reader: MessageReader<BumpWhiffed>,
    bindings: Res<ActiveBehaviors>,
    mut commands: Commands,
) {
    if reader.read().count() == 0 {
        return;
    }
    fire_consequences(&bindings, Trigger::BumpWhiff, &mut commands);
}

/// Reads `BumpPerformed` messages and fires consequence events for the
/// corresponding bump trigger.
pub(crate) fn bridge_bump(
    mut reader: MessageReader<BumpPerformed>,
    bindings: Res<ActiveBehaviors>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let trigger = match msg.grade {
            BumpGrade::Perfect => Trigger::PerfectBump,
            BumpGrade::Early => Trigger::EarlyBump,
            BumpGrade::Late => Trigger::LateBump,
        };
        fire_consequences(&bindings, trigger, &mut commands);
    }
}

/// Dispatches consequences for the given trigger via [`ConsequenceFired`].
///
/// Each handler self-selects via pattern matching on the [`Consequence`] variant.
/// Adding a new consequence never touches this function.
///
/// `BoltSpeedBoost` is skipped — it is applied once at init time by
/// `consequences::bolt_speed_boost` when the archetype loads.
fn fire_consequences(bindings: &ActiveBehaviors, trigger: Trigger, commands: &mut Commands) {
    for consequence in bindings.consequences_for(trigger) {
        if matches!(consequence, Consequence::BoltSpeedBoost(_)) {
            continue; // Init-time only — no runtime handler
        }
        commands.trigger(ConsequenceFired(consequence.clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::breaker::messages::BumpWhiffed;

    #[derive(Resource)]
    struct SendBoltLost(bool);

    fn send_bolt_lost(flag: Res<SendBoltLost>, mut writer: MessageWriter<BoltLost>) {
        if flag.0 {
            writer.write(BoltLost);
        }
    }

    #[derive(Resource)]
    struct SendBump(Option<BumpPerformed>);

    fn send_bump(msg: Res<SendBump>, mut writer: MessageWriter<BumpPerformed>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    mod bolt_lost {
        use super::*;
        use crate::{behaviors::consequences::life_lost::LivesCount, run::messages::RunLost};

        fn test_app() -> App {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins)
                .add_message::<BoltLost>()
                .add_message::<RunLost>()
                .insert_resource(ActiveBehaviors(vec![(
                    Trigger::BoltLost,
                    Consequence::LoseLife,
                )]))
                .insert_resource(SendBoltLost(false))
                .add_observer(crate::behaviors::consequences::life_lost::handle_life_lost)
                .add_systems(FixedUpdate, (send_bolt_lost, bridge_bolt_lost).chain());
            app
        }

        #[test]
        fn bolt_lost_triggers_lose_life() {
            let mut app = test_app();
            let entity = app.world_mut().spawn(LivesCount(3)).id();
            app.world_mut().resource_mut::<SendBoltLost>().0 = true;
            tick(&mut app);

            let lives = app.world().get::<LivesCount>(entity).unwrap();
            assert_eq!(lives.0, 2);
        }

        #[test]
        fn no_bolt_lost_no_consequence() {
            let mut app = test_app();
            let entity = app.world_mut().spawn(LivesCount(3)).id();
            tick(&mut app);

            let lives = app.world().get::<LivesCount>(entity).unwrap();
            assert_eq!(lives.0, 3);
        }
    }

    mod bolt_lost_time_penalty {
        use super::*;
        use crate::run::node::messages::ApplyTimePenalty;

        #[derive(Resource, Default)]
        struct CapturedPenalties(Vec<f32>);

        fn capture_penalties(
            mut reader: MessageReader<ApplyTimePenalty>,
            mut captured: ResMut<CapturedPenalties>,
        ) {
            for msg in reader.read() {
                captured.0.push(msg.seconds);
            }
        }

        fn test_app() -> App {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins)
                .add_message::<BoltLost>()
                .add_message::<ApplyTimePenalty>()
                .insert_resource(ActiveBehaviors(vec![(
                    Trigger::BoltLost,
                    Consequence::TimePenalty(5.0),
                )]))
                .insert_resource(SendBoltLost(false))
                .init_resource::<CapturedPenalties>()
                .add_observer(crate::behaviors::consequences::time_penalty::handle_time_penalty)
                .add_systems(
                    FixedUpdate,
                    (send_bolt_lost, bridge_bolt_lost, capture_penalties).chain(),
                );
            app
        }

        #[test]
        fn bolt_lost_triggers_time_penalty() {
            let mut app = test_app();
            app.world_mut().resource_mut::<SendBoltLost>().0 = true;
            tick(&mut app);

            let captured = app.world().resource::<CapturedPenalties>();
            assert_eq!(captured.0.len(), 1);
            assert!((captured.0[0] - 5.0).abs() < f32::EPSILON);
        }

        #[test]
        fn time_penalty_does_not_lose_life() {
            use crate::behaviors::consequences::life_lost::LivesCount;

            let mut app = test_app();
            let entity = app.world_mut().spawn(LivesCount(3)).id();
            app.world_mut().resource_mut::<SendBoltLost>().0 = true;
            tick(&mut app);

            let lives = app.world().get::<LivesCount>(entity).unwrap();
            assert_eq!(lives.0, 3, "TimePenalty should not affect lives");
        }
    }

    mod bump_spawn_bolt {
        use super::*;
        use crate::bolt::messages::SpawnAdditionalBolt;

        #[derive(Resource, Default)]
        struct CapturedSpawnBolt(u32);

        fn capture_spawn(
            mut reader: MessageReader<SpawnAdditionalBolt>,
            mut captured: ResMut<CapturedSpawnBolt>,
        ) {
            for _msg in reader.read() {
                captured.0 += 1;
            }
        }

        fn test_app() -> App {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins)
                .add_message::<BumpPerformed>()
                .add_message::<SpawnAdditionalBolt>()
                .insert_resource(ActiveBehaviors(vec![(
                    Trigger::PerfectBump,
                    Consequence::SpawnBolt,
                )]))
                .insert_resource(SendBump(None))
                .init_resource::<CapturedSpawnBolt>()
                .add_observer(crate::behaviors::consequences::spawn_bolt::handle_spawn_bolt)
                .add_systems(FixedUpdate, (send_bump, bridge_bump, capture_spawn).chain());
            app
        }

        #[test]
        fn perfect_bump_triggers_spawn_bolt() {
            let mut app = test_app();
            app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
                grade: BumpGrade::Perfect,
                multiplier: 1.5,
            });
            tick(&mut app);

            let captured = app.world().resource::<CapturedSpawnBolt>();
            assert_eq!(captured.0, 1, "perfect bump should trigger SpawnBolt");
        }

        #[test]
        fn early_bump_does_not_trigger_spawn_bolt() {
            let mut app = test_app();
            app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
                grade: BumpGrade::Early,
                multiplier: 1.1,
            });
            tick(&mut app);

            let captured = app.world().resource::<CapturedSpawnBolt>();
            assert_eq!(captured.0, 0, "early bump should not trigger SpawnBolt");
        }
    }

    mod bump {
        use super::*;
        use crate::{behaviors::consequences::life_lost::LivesCount, run::messages::RunLost};

        fn test_app() -> App {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins)
                .add_message::<BumpPerformed>()
                .add_message::<RunLost>();
            // BumpWhiff triggers LoseLife (for testing bridge_bump dispatch)
            app.insert_resource(ActiveBehaviors(vec![
                (Trigger::PerfectBump, Consequence::BoltSpeedBoost(1.5)),
                (Trigger::EarlyBump, Consequence::LoseLife),
            ]))
            .insert_resource(SendBump(None))
            .add_observer(crate::behaviors::consequences::life_lost::handle_life_lost)
            .add_systems(FixedUpdate, (send_bump, bridge_bump).chain());
            app
        }

        #[test]
        fn early_bump_triggers_lose_life() {
            let mut app = test_app();
            let entity = app.world_mut().spawn(LivesCount(3)).id();
            app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
                grade: BumpGrade::Early,
                multiplier: 1.1,
            });
            tick(&mut app);

            let lives = app.world().get::<LivesCount>(entity).unwrap();
            assert_eq!(lives.0, 2);
        }

        #[test]
        fn perfect_bump_does_not_lose_life() {
            let mut app = test_app();
            let entity = app.world_mut().spawn(LivesCount(3)).id();
            app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
                grade: BumpGrade::Perfect,
                multiplier: 1.5,
            });
            tick(&mut app);

            // BoltSpeedBoost is init-time only, should not fire LoseLife
            let lives = app.world().get::<LivesCount>(entity).unwrap();
            assert_eq!(lives.0, 3);
        }
    }

    mod bump_whiff {
        use super::*;
        use crate::{behaviors::consequences::life_lost::LivesCount, run::messages::RunLost};

        #[derive(Resource)]
        struct SendBumpWhiff(bool);

        fn send_bump_whiff(flag: Res<SendBumpWhiff>, mut writer: MessageWriter<BumpWhiffed>) {
            if flag.0 {
                writer.write(BumpWhiffed);
            }
        }

        fn test_app() -> App {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins)
                .add_message::<BumpWhiffed>()
                .add_message::<RunLost>()
                .insert_resource(ActiveBehaviors(vec![(
                    Trigger::BumpWhiff,
                    Consequence::LoseLife,
                )]))
                .insert_resource(SendBumpWhiff(false))
                .add_observer(crate::behaviors::consequences::life_lost::handle_life_lost)
                .add_systems(FixedUpdate, (send_bump_whiff, bridge_bump_whiff).chain());
            app
        }

        #[test]
        fn bump_whiff_triggers_lose_life() {
            let mut app = test_app();
            let entity = app.world_mut().spawn(LivesCount(3)).id();
            app.world_mut().resource_mut::<SendBumpWhiff>().0 = true;
            tick(&mut app);

            let lives = app.world().get::<LivesCount>(entity).unwrap();
            assert_eq!(lives.0, 2);
        }

        #[test]
        fn no_bump_whiff_no_consequence() {
            let mut app = test_app();
            let entity = app.world_mut().spawn(LivesCount(3)).id();
            tick(&mut app);

            let lives = app.world().get::<LivesCount>(entity).unwrap();
            assert_eq!(lives.0, 3);
        }

        #[test]
        fn bump_whiff_does_not_fire_for_bolt_lost_trigger() {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins)
                .add_message::<BumpWhiffed>()
                .add_message::<RunLost>()
                .insert_resource(ActiveBehaviors(vec![(
                    Trigger::BoltLost,
                    Consequence::LoseLife,
                )]))
                .insert_resource(SendBumpWhiff(false))
                .add_observer(crate::behaviors::consequences::life_lost::handle_life_lost)
                .add_systems(FixedUpdate, (send_bump_whiff, bridge_bump_whiff).chain());

            let entity = app.world_mut().spawn(LivesCount(3)).id();
            app.world_mut().resource_mut::<SendBumpWhiff>().0 = true;
            tick(&mut app);

            let lives = app.world().get::<LivesCount>(entity).unwrap();
            assert_eq!(
                lives.0, 3,
                "BumpWhiff trigger should not fire BoltLost consequence"
            );
        }
    }

    mod bump_whiff_time_penalty {
        use super::*;
        use crate::run::node::messages::ApplyTimePenalty;

        #[derive(Resource)]
        struct SendBumpWhiff(bool);

        fn send_bump_whiff(flag: Res<SendBumpWhiff>, mut writer: MessageWriter<BumpWhiffed>) {
            if flag.0 {
                writer.write(BumpWhiffed);
            }
        }

        #[derive(Resource, Default)]
        struct CapturedPenalties(Vec<f32>);

        fn capture_penalties(
            mut reader: MessageReader<ApplyTimePenalty>,
            mut captured: ResMut<CapturedPenalties>,
        ) {
            for msg in reader.read() {
                captured.0.push(msg.seconds);
            }
        }

        fn test_app() -> App {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins)
                .add_message::<BumpWhiffed>()
                .add_message::<ApplyTimePenalty>()
                .insert_resource(ActiveBehaviors(vec![(
                    Trigger::BumpWhiff,
                    Consequence::TimePenalty(3.0),
                )]))
                .insert_resource(SendBumpWhiff(false))
                .init_resource::<CapturedPenalties>()
                .add_observer(crate::behaviors::consequences::time_penalty::handle_time_penalty)
                .add_systems(
                    FixedUpdate,
                    (send_bump_whiff, bridge_bump_whiff, capture_penalties).chain(),
                );
            app
        }

        #[test]
        fn bump_whiff_triggers_time_penalty() {
            let mut app = test_app();
            app.world_mut().resource_mut::<SendBumpWhiff>().0 = true;
            tick(&mut app);

            let captured = app.world().resource::<CapturedPenalties>();
            assert_eq!(captured.0.len(), 1);
            assert!(
                (captured.0[0] - 3.0).abs() < f32::EPSILON,
                "expected 3.0 second penalty, got {}",
                captured.0[0]
            );
        }
    }
}
