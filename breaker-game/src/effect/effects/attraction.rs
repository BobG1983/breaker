//! Attraction chip effect observer — pulls nearby cells toward the bolt.

use bevy::prelude::*;

use super::stack_f32;
use crate::{
    bolt::components::Bolt, chips::components::AttractionForce,
    effect::typed_events::AttractionApplied,
};

/// Observer: applies attraction force stacking to all bolt entities.
pub(crate) fn handle_attraction(
    trigger: On<AttractionApplied>,
    mut query: Query<(Entity, Option<&mut AttractionForce>), With<Bolt>>,
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
            AttractionForce,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_attraction);
        app
    }

    #[test]
    fn inserts_attraction_force_on_bolt() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn(Bolt).id();
        // Non-bolt entity should NOT receive the component.
        let non_bolt = app.world_mut().spawn_empty().id();

        app.world_mut().commands().trigger(AttractionApplied {
            attraction_type: crate::effect::definition::AttractionType::Cell,
            per_stack: 8.0,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let a = app
            .world()
            .entity(bolt)
            .get::<AttractionForce>()
            .expect("bolt should have AttractionForce after Attraction effect");
        assert!((a.0 - 8.0).abs() < f32::EPSILON);

        assert!(
            app.world()
                .entity(non_bolt)
                .get::<AttractionForce>()
                .is_none(),
            "non-bolt entity should NOT receive AttractionForce"
        );
    }

    #[test]
    fn stacks_attraction_force() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn((Bolt, AttractionForce(8.0))).id();

        app.world_mut().commands().trigger(AttractionApplied {
            attraction_type: crate::effect::definition::AttractionType::Cell,
            per_stack: 8.0,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let a = app.world().entity(bolt).get::<AttractionForce>().unwrap();
        assert!(
            (a.0 - 16.0).abs() < f32::EPSILON,
            "AttractionForce should stack from 8.0 to 16.0, got {}",
            a.0
        );
    }

    #[test]
    fn respects_max_stacks_attraction_force() {
        let mut app = test_app();
        // 3 stacks at 8.0 per stack = 24.0, which is at the cap of max_stacks: 3.
        let bolt = app.world_mut().spawn((Bolt, AttractionForce(24.0))).id();

        app.world_mut().commands().trigger(AttractionApplied {
            attraction_type: crate::effect::definition::AttractionType::Cell,
            per_stack: 8.0,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let a = app.world().entity(bolt).get::<AttractionForce>().unwrap();
        assert!(
            (a.0 - 24.0).abs() < f32::EPSILON,
            "AttractionForce should not exceed max_stacks cap, got {}",
            a.0
        );
    }
}
