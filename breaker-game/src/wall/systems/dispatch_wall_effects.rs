//! Dispatches wall-defined effects to wall entities when spawned.
//!
//! Walls don't currently have RON definitions with effects, so this system
//! is a no-op. When wall definitions gain optional effects, this system will
//! look up each wall's definition, resolve targets, and push to `BoundEffects`.

use bevy::prelude::*;

use crate::wall::components::Wall;

/// Dispatches effects to wall entities.
///
/// No wall definitions currently define effects, so this is a no-op.
pub(crate) const fn dispatch_wall_effects(_commands: Commands, _walls: Query<Entity, With<Wall>>) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        effect::core::{BoundEffects, EffectKind, EffectNode, StagedEffects, Trigger},
        wall::components::Wall,
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, dispatch_wall_effects);
        app
    }

    // -- Behavior 1: no panic when no Wall entities exist --

    #[test]
    fn no_panic_when_no_wall_entities_exist() {
        let mut app = test_app();
        // World is empty aside from MinimalPlugins boilerplate entities.
        app.update();

        // Verify no Wall entities were spuriously created.
        let wall_count = app
            .world_mut()
            .query_filtered::<Entity, With<Wall>>()
            .iter(app.world())
            .count();
        assert_eq!(
            wall_count, 0,
            "no Wall entities should exist after running dispatch_wall_effects on an empty world"
        );
    }

    // -- Behavior 2: no panic and no spurious inserts when Wall entities exist --

    #[test]
    fn no_spurious_inserts_on_wall_entities_without_effect_definitions() {
        let mut app = test_app();

        // Spawn 3 Wall entities (matching left/right/ceiling pattern).
        let wall_left = app.world_mut().spawn(Wall).id();
        let wall_right = app.world_mut().spawn(Wall).id();
        let wall_ceiling = app.world_mut().spawn(Wall).id();

        app.update();

        // No BoundEffects should have been inserted on any wall.
        assert!(
            app.world().get::<BoundEffects>(wall_left).is_none(),
            "left wall should not have BoundEffects inserted by dispatch_wall_effects"
        );
        assert!(
            app.world().get::<BoundEffects>(wall_right).is_none(),
            "right wall should not have BoundEffects inserted by dispatch_wall_effects"
        );
        assert!(
            app.world().get::<BoundEffects>(wall_ceiling).is_none(),
            "ceiling wall should not have BoundEffects inserted by dispatch_wall_effects"
        );

        // No StagedEffects should have been inserted on any wall.
        assert!(
            app.world().get::<StagedEffects>(wall_left).is_none(),
            "left wall should not have StagedEffects inserted by dispatch_wall_effects"
        );
        assert!(
            app.world().get::<StagedEffects>(wall_right).is_none(),
            "right wall should not have StagedEffects inserted by dispatch_wall_effects"
        );
        assert!(
            app.world().get::<StagedEffects>(wall_ceiling).is_none(),
            "ceiling wall should not have StagedEffects inserted by dispatch_wall_effects"
        );
    }

    // -- Behavior 3: pre-existing BoundEffects unchanged --

    #[test]
    fn does_not_modify_existing_bound_effects_on_wall_entity() {
        let mut app = test_app();

        let pre_existing_effects = BoundEffects(vec![(
            "test_chip".to_string(),
            EffectNode::When {
                trigger: Trigger::Bump,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            },
        )]);

        let wall_entity = app.world_mut().spawn((Wall, pre_existing_effects)).id();

        app.update();

        let bound = app
            .world()
            .get::<BoundEffects>(wall_entity)
            .expect("Wall entity should still have BoundEffects after dispatch_wall_effects runs");

        assert_eq!(
            bound.0.len(),
            1,
            "BoundEffects should still have exactly 1 entry, got {}",
            bound.0.len()
        );

        let (chip_name, node) = &bound.0[0];
        assert_eq!(chip_name, "test_chip", "chip name should be unchanged");
        assert_eq!(
            *node,
            EffectNode::When {
                trigger: Trigger::Bump,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            },
            "effect node should be unchanged"
        );
    }
}
