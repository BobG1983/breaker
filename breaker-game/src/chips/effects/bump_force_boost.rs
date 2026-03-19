//! Bump force boost chip effect observer — adds flat bump force to breaker.

use bevy::prelude::*;

use super::stack_f32;
use crate::{
    breaker::components::Breaker,
    chips::{
        components::BumpForceBoost,
        definition::{AugmentEffect, ChipEffect},
        messages::ChipEffectApplied,
    },
};

/// Observer: applies bump force boost stacking to all breaker entities.
pub(crate) fn handle_bump_force_boost(
    trigger: On<ChipEffectApplied>,
    mut query: Query<(Entity, Option<&mut BumpForceBoost>), With<Breaker>>,
    mut commands: Commands,
) {
    let ChipEffect::Augment(AugmentEffect::BumpForce(per_stack)) = trigger.event().effect else {
        return;
    };
    let max_stacks = trigger.event().max_stacks;
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

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Augment(AugmentEffect::BumpForce(10.0)),
            max_stacks: 3,
        });
        app.world_mut().flush();

        let b = app.world().entity(breaker).get::<BumpForceBoost>().unwrap();
        assert!((b.0 - 10.0).abs() < f32::EPSILON);
    }
}
