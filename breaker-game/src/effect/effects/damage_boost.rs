//! Damage boost chip effect observer — multiplies bolt damage.

use bevy::prelude::*;

use super::stack_f32;
use crate::{
    bolt::components::Bolt, chips::components::DamageBoost,
    effect::typed_events::DamageBoostApplied,
};

/// Observer: applies damage boost stacking to all bolt entities.
pub(crate) fn handle_damage_boost(
    trigger: On<DamageBoostApplied>,
    mut query: Query<(Entity, Option<&mut DamageBoost>), With<Bolt>>,
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
            DamageBoost,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_damage_boost);
        app
    }

    #[test]
    fn inserts_damage_boost_on_bolt() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn(Bolt).id();

        app.world_mut().commands().trigger(DamageBoostApplied {
            per_stack: 1.5,
            max_stacks: 2,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let d = app.world().entity(bolt).get::<DamageBoost>().unwrap();
        assert!((d.0 - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn stacks_damage_boost() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn((Bolt, DamageBoost(1.5))).id();

        app.world_mut().commands().trigger(DamageBoostApplied {
            per_stack: 1.5,
            max_stacks: 2,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let d = app.world().entity(bolt).get::<DamageBoost>().unwrap();
        assert!((d.0 - 3.0).abs() < f32::EPSILON);
    }
}
