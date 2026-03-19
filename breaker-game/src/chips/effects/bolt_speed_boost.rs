//! Bolt speed boost chip effect observer — adds flat speed to bolt.

use bevy::prelude::*;

use super::stack_f32;
use crate::{
    bolt::components::Bolt,
    chips::{
        components::BoltSpeedBoost,
        definition::{AmpEffect, ChipEffect},
        definition::ChipEffectApplied,
    },
};

/// Observer: applies bolt speed boost stacking to all bolt entities.
pub(crate) fn handle_bolt_speed_boost(
    trigger: On<ChipEffectApplied>,
    mut query: Query<(Entity, Option<&mut BoltSpeedBoost>), With<Bolt>>,
    mut commands: Commands,
) {
    let ChipEffect::Amp(AmpEffect::SpeedBoost(per_stack)) = trigger.event().effect else {
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
            BoltSpeedBoost,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_bolt_speed_boost);
        app
    }

    #[test]
    fn inserts_bolt_speed_boost_on_bolt() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn(Bolt).id();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: ChipEffect::Amp(AmpEffect::SpeedBoost(50.0)),
            max_stacks: 3,
        });
        app.world_mut().flush();

        let s = app.world().entity(bolt).get::<BoltSpeedBoost>().unwrap();
        assert!((s.0 - 50.0).abs() < f32::EPSILON);
    }
}
