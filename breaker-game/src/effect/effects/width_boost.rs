//! Width boost chip effect observer — adds flat width to breaker.

use bevy::prelude::*;

use super::stack_f32;
use crate::{
    breaker::components::Breaker, chips::components::WidthBoost,
    effect::typed_events::SizeBoostApplied,
};

/// Observer: applies width boost stacking to all breaker entities.
pub(crate) fn handle_width_boost(
    trigger: On<SizeBoostApplied>,
    mut query: Query<(Entity, Option<&mut WidthBoost>), With<Breaker>>,
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
            WidthBoost,
        );
    }
}

/// Registers all observers and systems for the width boost effect.
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_width_boost);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_width_boost);
        app
    }

    #[test]
    fn inserts_width_boost_on_breaker() {
        let mut app = test_app();
        let breaker = app.world_mut().spawn(Breaker).id();

        app.world_mut().commands().trigger(SizeBoostApplied {
            per_stack: 20.0,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let w = app.world().entity(breaker).get::<WidthBoost>().unwrap();
        assert!((w.0 - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn stacks_width_boost() {
        let mut app = test_app();
        let breaker = app.world_mut().spawn((Breaker, WidthBoost(20.0))).id();

        app.world_mut().commands().trigger(SizeBoostApplied {
            per_stack: 20.0,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let w = app.world().entity(breaker).get::<WidthBoost>().unwrap();
        assert!((w.0 - 40.0).abs() < f32::EPSILON);
    }
}
