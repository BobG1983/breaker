//! Tilt control boost chip effect observer — adds sensitivity to breaker.

use bevy::prelude::*;

use super::stack_f32;
use crate::{
    breaker::components::Breaker,
    chips::{
        components::TiltControlBoost,
        definition::{AugmentEffect, ChipEffect},
        messages::ChipEffectApplied,
    },
};

/// Observer: applies tilt control boost stacking to all breaker entities.
pub(crate) fn handle_tilt_control_boost(
    trigger: On<ChipEffectApplied>,
    mut query: Query<(Entity, Option<&mut TiltControlBoost>), With<Breaker>>,
    mut commands: Commands,
) {
    let ChipEffect::Augment(AugmentEffect::TiltControl(per_stack)) = trigger.event().effect else {
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
            TiltControlBoost,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_tilt_control_boost);
        app
    }

    #[test]
    fn inserts_tilt_control_boost_on_breaker() {
        let mut app = test_app();
        let breaker = app.world_mut().spawn(Breaker).id();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Augment(AugmentEffect::TiltControl(5.0)),
            max_stacks: 3,
        });
        app.world_mut().flush();

        let t = app
            .world()
            .entity(breaker)
            .get::<TiltControlBoost>()
            .unwrap();
        assert!((t.0 - 5.0).abs() < f32::EPSILON);
    }
}
