//! Bolt size boost chip effect observer — increases bolt radius.

use bevy::prelude::*;

use super::stack_f32;
use crate::{
    bolt::components::Bolt,
    chips::{
        components::BoltSizeBoost,
        definition::{ChipEffectApplied, Target, TriggerChain},
    },
};

/// Observer: applies bolt size boost stacking to all bolt entities.
pub(crate) fn handle_bolt_size_boost(
    trigger: On<ChipEffectApplied>,
    mut query: Query<(Entity, Option<&mut BoltSizeBoost>), With<Bolt>>,
    mut commands: Commands,
) {
    let &TriggerChain::SizeBoost(Target::Bolt, per_stack) = &trigger.event().effect else {
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
            BoltSizeBoost,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_bolt_size_boost);
        app
    }

    #[test]
    fn inserts_bolt_size_boost_on_bolt() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn(Bolt).id();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: TriggerChain::SizeBoost(Target::Bolt, 0.5),
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let s = app.world().entity(bolt).get::<BoltSizeBoost>().unwrap();
        assert!((s.0 - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn stacks_bolt_size_boost() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn((Bolt, BoltSizeBoost(0.5))).id();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: TriggerChain::SizeBoost(Target::Bolt, 0.5),
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let s = app.world().entity(bolt).get::<BoltSizeBoost>().unwrap();
        assert!(
            (s.0 - 1.0).abs() < f32::EPSILON,
            "BoltSizeBoost should stack from 0.5 to 1.0, got {}",
            s.0
        );
    }

    #[test]
    fn respects_max_stacks_bolt_size_boost() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn((Bolt, BoltSizeBoost(1.5))).id();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: TriggerChain::SizeBoost(Target::Bolt, 0.5),
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let s = app.world().entity(bolt).get::<BoltSizeBoost>().unwrap();
        assert!(
            (s.0 - 1.5).abs() < f32::EPSILON,
            "BoltSizeBoost should not exceed max_stacks cap, got {}",
            s.0
        );
    }

    #[test]
    fn ignores_breaker_target() {
        let mut app = test_app();
        app.world_mut().spawn(Bolt);

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: TriggerChain::SizeBoost(Target::Breaker, 20.0),
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        assert!(
            app.world_mut()
                .query::<&BoltSizeBoost>()
                .iter(app.world())
                .next()
                .is_none(),
            "handle_bolt_size_boost should ignore Target::Breaker"
        );
    }
}
