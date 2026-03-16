//! Per-trigger bridge systems — translate messages into consequence events.

use bevy::prelude::*;

use super::{
    active::ActiveBehaviors,
    consequences::{life_lost::LoseLifeRequested, time_penalty::TimePenaltyRequested},
    definition::{Consequence, Trigger},
};
use crate::{
    breaker::messages::{BumpGrade, BumpPerformed},
    physics::messages::BoltLost,
};

/// Reads `BoltLost` messages and fires consequence events for that trigger.
///
/// Drains all messages (not just the first) so that extras from future multi-bolt
/// archetypes don't leak into subsequent frames. Consequences fire once per
/// bridge invocation regardless of how many bolts were lost in the same frame.
pub fn bridge_bolt_lost(
    mut reader: MessageReader<BoltLost>,
    bindings: Res<ActiveBehaviors>,
    mut commands: Commands,
) {
    if reader.read().count() == 0 {
        return;
    }
    fire_consequences(&bindings, Trigger::BoltLost, &mut commands);
}

/// Reads `BumpPerformed` messages and fires consequence events for the
/// corresponding bump trigger.
pub fn bridge_bump(
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

/// Dispatches consequences for the given trigger to ECS commands.
///
/// Iterates the active behavior bindings for `trigger` and translates each
/// [`Consequence`] variant into the appropriate ECS command or observer trigger.
///
/// `BoltSpeedBoost` is intentionally a no-op here — it is applied once at
/// init time by `consequences::bolt_speed_boost` when the archetype loads.
/// Adding a runtime arm here for it would double-apply the multiplier.
fn fire_consequences(bindings: &ActiveBehaviors, trigger: Trigger, commands: &mut Commands) {
    for consequence in bindings.consequences_for(trigger) {
        match consequence {
            Consequence::LoseLife => {
                commands.trigger(LoseLifeRequested);
            }
            Consequence::BoltSpeedBoost(_) => {
                // Init-time only — handled by consequences::bolt_speed_boost
            }
            Consequence::TimePenalty(seconds) => {
                commands.trigger(TimePenaltyRequested { seconds: *seconds });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        use crate::{
            breaker::behaviors::consequences::life_lost::LivesCount, run::messages::RunLost,
        };

        fn test_app() -> App {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            app.add_message::<BoltLost>();
            app.add_message::<RunLost>();
            app.insert_resource(ActiveBehaviors(vec![(
                Trigger::BoltLost,
                Consequence::LoseLife,
            )]));
            app.insert_resource(SendBoltLost(false));
            app.add_observer(crate::breaker::behaviors::consequences::life_lost::handle_life_lost);
            app.add_systems(FixedUpdate, (send_bolt_lost, bridge_bolt_lost).chain());
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
            app.add_plugins(MinimalPlugins);
            app.add_message::<BoltLost>();
            app.add_message::<ApplyTimePenalty>();
            app.insert_resource(ActiveBehaviors(vec![(
                Trigger::BoltLost,
                Consequence::TimePenalty(5.0),
            )]));
            app.insert_resource(SendBoltLost(false));
            app.init_resource::<CapturedPenalties>();
            app.add_observer(
                crate::breaker::behaviors::consequences::time_penalty::handle_time_penalty,
            );
            app.add_systems(
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
            use crate::breaker::behaviors::consequences::life_lost::LivesCount;

            let mut app = test_app();
            let entity = app.world_mut().spawn(LivesCount(3)).id();
            app.world_mut().resource_mut::<SendBoltLost>().0 = true;
            tick(&mut app);

            let lives = app.world().get::<LivesCount>(entity).unwrap();
            assert_eq!(lives.0, 3, "TimePenalty should not affect lives");
        }
    }

    mod bump {
        use super::*;
        use crate::{
            breaker::behaviors::consequences::life_lost::LivesCount, run::messages::RunLost,
        };

        fn test_app() -> App {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            app.add_message::<BumpPerformed>();
            app.add_message::<RunLost>();
            // BumpWhiff triggers LoseLife (for testing bridge_bump dispatch)
            app.insert_resource(ActiveBehaviors(vec![
                (Trigger::PerfectBump, Consequence::BoltSpeedBoost(1.5)),
                (Trigger::EarlyBump, Consequence::LoseLife),
            ]));
            app.insert_resource(SendBump(None));
            app.add_observer(crate::breaker::behaviors::consequences::life_lost::handle_life_lost);
            app.add_systems(FixedUpdate, (send_bump, bridge_bump).chain());
            app
        }

        #[test]
        fn early_bump_triggers_lose_life() {
            let mut app = test_app();
            let entity = app.world_mut().spawn(LivesCount(3)).id();
            app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
                grade: BumpGrade::Early,
                multiplier: 0.8,
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
}
