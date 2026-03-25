//! Bump force boost chip effect observer — adds flat bump force to breaker.

use bevy::prelude::*;

use super::stack_f32;
use crate::{
    breaker::components::Breaker, chips::components::BumpForceBoost,
    effect::typed_events::BumpForceApplied,
};

/// Observer: applies bump force boost stacking to all breaker entities.
pub(crate) fn handle_bump_force_boost(
    trigger: On<BumpForceApplied>,
    mut query: Query<(Entity, Option<&mut BumpForceBoost>), With<Breaker>>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let per_stack = event.per_stack;
    let max_stacks = event.max_stacks;
    for (entity, mut existing) in &mut query {
        stack_f32(
            entity,
            existing.as_deref_mut().map(|c| &mut c.0),
            per_stack,
            max_stacks,
            &mut commands,
            BumpForceBoost,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_bump_force_boost);
        app
    }

    #[test]
    fn inserts_bump_force_boost_on_breaker() {
        let mut app = test_app();
        let breaker = app.world_mut().spawn(Breaker).id();

        app.world_mut().commands().trigger(BumpForceApplied {
            per_stack: 10.0,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let b = app.world().entity(breaker).get::<BumpForceBoost>().unwrap();
        assert!((b.0 - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn stacks_bump_force_boost() {
        let mut app = test_app();
        let breaker = app.world_mut().spawn((Breaker, BumpForceBoost(10.0))).id();

        app.world_mut().commands().trigger(BumpForceApplied {
            per_stack: 10.0,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let b = app.world().entity(breaker).get::<BumpForceBoost>().unwrap();
        assert!(
            (b.0 - 20.0).abs() < f32::EPSILON,
            "BumpForceBoost should stack from 10.0 to 20.0, got {}",
            b.0
        );
    }

    #[test]
    fn respects_max_stacks_bump_force_boost() {
        let mut app = test_app();
        let breaker = app.world_mut().spawn((Breaker, BumpForceBoost(30.0))).id();

        app.world_mut().commands().trigger(BumpForceApplied {
            per_stack: 10.0,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let b = app.world().entity(breaker).get::<BumpForceBoost>().unwrap();
        assert!(
            (b.0 - 30.0).abs() < f32::EPSILON,
            "BumpForceBoost should not exceed max_stacks cap, got {}",
            b.0
        );
    }
}
