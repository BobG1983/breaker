//! Breaker speed boost chip effect observer — adds flat speed to breaker.

use bevy::prelude::*;

use super::stack_f32;
use crate::{
    breaker::components::Breaker, chips::components::BreakerSpeedBoost,
    effect::typed_events::SpeedBoostApplied,
};

/// Observer: applies breaker speed boost stacking to all breaker entities.
pub(crate) fn handle_breaker_speed_boost(
    trigger: On<SpeedBoostApplied>,
    mut query: Query<(Entity, Option<&mut BreakerSpeedBoost>), With<Breaker>>,
    mut commands: Commands,
) {
    let event = trigger.event();
    if event.target != crate::effect::definition::Target::Breaker {
        return;
    }
    let per_stack = event.multiplier;
    let max_stacks = event.max_stacks;
    for (entity, mut existing) in &mut query {
        stack_f32(
            entity,
            existing.as_deref_mut().map(|c| &mut c.0),
            per_stack,
            max_stacks,
            &mut commands,
            BreakerSpeedBoost,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_breaker_speed_boost);
        app
    }

    #[test]
    fn inserts_breaker_speed_boost_on_breaker() {
        let mut app = test_app();
        let breaker = app.world_mut().spawn(Breaker).id();

        app.world_mut().commands().trigger(SpeedBoostApplied {
            target: crate::effect::definition::Target::Breaker,
            multiplier: 1.1,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let s = app
            .world()
            .entity(breaker)
            .get::<BreakerSpeedBoost>()
            .unwrap();
        assert!((s.0 - 1.1).abs() < f32::EPSILON);
    }

    #[test]
    fn stacks_breaker_speed_boost() {
        let mut app = test_app();
        let breaker = app
            .world_mut()
            .spawn((Breaker, BreakerSpeedBoost(1.1)))
            .id();

        app.world_mut().commands().trigger(SpeedBoostApplied {
            target: crate::effect::definition::Target::Breaker,
            multiplier: 1.1,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let s = app
            .world()
            .entity(breaker)
            .get::<BreakerSpeedBoost>()
            .unwrap();
        assert!(
            (s.0 - 2.2).abs() < f32::EPSILON,
            "BreakerSpeedBoost should stack from 1.1 to 2.2, got {}",
            s.0
        );
    }

    #[test]
    fn ignores_bolt_target() {
        let mut app = test_app();
        app.world_mut().spawn(Breaker);

        app.world_mut().commands().trigger(SpeedBoostApplied {
            target: crate::effect::definition::Target::Bolt,
            multiplier: 1.1,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        assert!(
            app.world_mut()
                .query::<&BreakerSpeedBoost>()
                .iter(app.world())
                .next()
                .is_none(),
            "handle_breaker_speed_boost should ignore Target::Bolt"
        );
    }
}
