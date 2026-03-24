//! Width boost chip effect observer — adds flat width to breaker.

use bevy::prelude::*;

use super::stack_f32;
use crate::{
    breaker::components::Breaker,
    chips::{
        components::WidthBoost,
        definition::{AugmentEffect, ChipEffect, ChipEffectApplied},
    },
};

/// Observer: applies width boost stacking to all breaker entities.
pub(crate) fn handle_width_boost(
    trigger: On<ChipEffectApplied>,
    mut query: Query<(Entity, Option<&mut WidthBoost>), With<Breaker>>,
    mut commands: Commands,
) {
    let ChipEffect::Augment(AugmentEffect::WidthBoost(per_stack)) = trigger.event().effect.clone()
    else {
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
            WidthBoost,
        );
    }
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

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Augment(AugmentEffect::WidthBoost(20.0)),
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

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Augment(AugmentEffect::WidthBoost(20.0)),
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let w = app.world().entity(breaker).get::<WidthBoost>().unwrap();
        assert!((w.0 - 40.0).abs() < f32::EPSILON);
    }
}
