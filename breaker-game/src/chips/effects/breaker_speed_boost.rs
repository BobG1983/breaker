//! Breaker speed boost chip effect observer — adds flat speed to breaker.

use bevy::prelude::*;

use super::stack_f32;
use crate::{
    breaker::components::Breaker,
    chips::{
        components::BreakerSpeedBoost,
        definition::{AugmentEffect, ChipEffect, ChipEffectApplied},
    },
};

/// Observer: applies breaker speed boost stacking to all breaker entities.
pub(crate) fn handle_breaker_speed_boost(
    trigger: On<ChipEffectApplied>,
    mut query: Query<(Entity, Option<&mut BreakerSpeedBoost>), With<Breaker>>,
    mut commands: Commands,
) {
    let ChipEffect::Augment(AugmentEffect::SpeedBoost(per_stack)) = trigger.event().effect.clone()
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

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Augment(AugmentEffect::SpeedBoost(30.0)),
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let s = app
            .world()
            .entity(breaker)
            .get::<BreakerSpeedBoost>()
            .unwrap();
        assert!((s.0 - 30.0).abs() < f32::EPSILON);
    }

    #[test]
    fn stacks_breaker_speed_boost() {
        let mut app = test_app();
        let breaker = app
            .world_mut()
            .spawn((Breaker, BreakerSpeedBoost(30.0)))
            .id();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Augment(AugmentEffect::SpeedBoost(30.0)),
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
            (s.0 - 60.0).abs() < f32::EPSILON,
            "BreakerSpeedBoost should stack from 30.0 to 60.0, got {}",
            s.0
        );
    }

    #[test]
    fn respects_max_stacks_breaker_speed_boost() {
        let mut app = test_app();
        // 3 stacks of 30.0 = 90.0 (at cap)
        let breaker = app
            .world_mut()
            .spawn((Breaker, BreakerSpeedBoost(90.0)))
            .id();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Augment(AugmentEffect::SpeedBoost(30.0)),
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
            (s.0 - 90.0).abs() < f32::EPSILON,
            "BreakerSpeedBoost should not exceed max_stacks cap, got {}",
            s.0
        );
    }
}
