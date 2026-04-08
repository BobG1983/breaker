//! System to initiate birthing animation on all bolt entities at `AnimateIn` entry.

use bevy::prelude::*;

use crate::{bolt::components::Bolt, prelude::*};

/// Query for bolt entities eligible for birthing (no existing `Birthing` component).
type BirthingEligibleQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Scale2D, &'static CollisionLayers),
    (With<Bolt>, Without<Birthing>),
>;

/// Inserts [`Birthing`] on all bolt entities that do not already have it,
/// zeroing their `Scale2D`, `PreviousScale`, and `CollisionLayers`.
///
/// Runs on `OnEnter(NodeState::AnimateIn)`.
pub(crate) fn begin_node_birthing(mut commands: Commands, query: BirthingEligibleQuery) {
    for (entity, scale, layers) in &query {
        let target_scale = *scale;
        let stashed_layers = *layers;

        commands.entity(entity).insert((
            Scale2D { x: 0.0, y: 0.0 },
            PreviousScale { x: 0.0, y: 0.0 },
            CollisionLayers::default(),
            Birthing::new(target_scale, stashed_layers),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{
        BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, WALL_LAYER, birthing::BIRTHING_DURATION,
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, begin_node_birthing);
        app
    }

    // Behavior 18: begin_node_birthing inserts Birthing on all bolts at AnimateIn entry
    #[test]
    fn inserts_birthing_on_all_bolts() {
        let mut app = test_app();

        let entity_a = app
            .world_mut()
            .spawn((
                Bolt,
                Scale2D { x: 8.0, y: 8.0 },
                PreviousScale { x: 8.0, y: 8.0 },
                CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
            ))
            .id();

        let entity_b = app
            .world_mut()
            .spawn((
                Bolt,
                Scale2D { x: 12.0, y: 12.0 },
                PreviousScale { x: 12.0, y: 12.0 },
                CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
            ))
            .id();

        app.update();

        // Entity A should have Birthing
        let birthing_a = app
            .world()
            .get::<Birthing>(entity_a)
            .expect("Entity A should have Birthing component");
        assert!(
            (birthing_a.timer.duration().as_secs_f32() - BIRTHING_DURATION).abs() < f32::EPSILON,
            "Entity A birthing timer should be {BIRTHING_DURATION}s"
        );
        assert!(
            (birthing_a.target_scale.x - 8.0).abs() < f32::EPSILON,
            "Entity A target_scale.x should be 8.0, got {}",
            birthing_a.target_scale.x
        );
        assert!(
            (birthing_a.target_scale.y - 8.0).abs() < f32::EPSILON,
            "Entity A target_scale.y should be 8.0, got {}",
            birthing_a.target_scale.y
        );
        assert_eq!(
            birthing_a.stashed_layers,
            CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
            "Entity A stashed_layers should match original"
        );

        // Entity A scale should be zeroed
        let scale_a = app.world().get::<Scale2D>(entity_a).unwrap();
        assert!(
            scale_a.x.abs() < f32::EPSILON && scale_a.y.abs() < f32::EPSILON,
            "Entity A Scale2D should be zeroed, got ({}, {})",
            scale_a.x,
            scale_a.y
        );

        // Entity A collision layers should be zeroed
        let layers_a = app.world().get::<CollisionLayers>(entity_a).unwrap();
        assert_eq!(
            layers_a.membership, 0,
            "Entity A CollisionLayers membership should be zeroed"
        );
        assert_eq!(
            layers_a.mask, 0,
            "Entity A CollisionLayers mask should be zeroed"
        );

        // Entity B should also have Birthing
        let birthing_b = app
            .world()
            .get::<Birthing>(entity_b)
            .expect("Entity B should have Birthing component");
        assert!(
            (birthing_b.target_scale.x - 12.0).abs() < f32::EPSILON,
            "Entity B target_scale.x should be 12.0, got {}",
            birthing_b.target_scale.x
        );
        assert!(
            (birthing_b.target_scale.y - 12.0).abs() < f32::EPSILON,
            "Entity B target_scale.y should be 12.0, got {}",
            birthing_b.target_scale.y
        );
    }

    // Behavior 19: begin_node_birthing skips bolts that already have Birthing
    #[test]
    fn skips_bolts_that_already_have_birthing() {
        let mut app = test_app();

        // Entity A: already has Birthing with partially elapsed timer
        let partially_elapsed_timer = {
            let mut t = Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once);
            t.tick(std::time::Duration::from_secs_f32(0.1));
            t
        };
        let entity_a = app
            .world_mut()
            .spawn((
                Bolt,
                Scale2D { x: 3.0, y: 3.0 },
                PreviousScale { x: 3.0, y: 3.0 },
                CollisionLayers::default(),
                Birthing {
                    timer: partially_elapsed_timer,
                    target_scale: Scale2D { x: 8.0, y: 8.0 },
                    stashed_layers: CollisionLayers::new(0x01, 0x0E),
                },
            ))
            .id();

        // Entity B: no Birthing
        let entity_b = app
            .world_mut()
            .spawn((
                Bolt,
                Scale2D { x: 8.0, y: 8.0 },
                PreviousScale { x: 8.0, y: 8.0 },
                CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
            ))
            .id();

        app.update();

        // Entity A's Birthing should not be reset -- timer should still be partially elapsed
        let birthing_a = app
            .world()
            .get::<Birthing>(entity_a)
            .expect("Entity A should still have Birthing");
        assert!(
            birthing_a.timer.elapsed_secs() > 0.05,
            "Entity A's partially elapsed timer should not be reset, elapsed: {}",
            birthing_a.timer.elapsed_secs()
        );

        // Entity B should have new Birthing
        assert!(
            app.world().get::<Birthing>(entity_b).is_some(),
            "Entity B should get a new Birthing component"
        );
    }

    // Behavior 20: begin_node_birthing only affects bolt entities
    #[test]
    fn only_affects_bolt_entities() {
        let mut app = test_app();

        // Entity A: has Bolt marker
        let entity_a = app
            .world_mut()
            .spawn((
                Bolt,
                Scale2D { x: 8.0, y: 8.0 },
                PreviousScale { x: 8.0, y: 8.0 },
                CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
            ))
            .id();

        // Entity B: no Bolt marker (simulates a cell or other entity) -- should NOT get Birthing
        let entity_b = app
            .world_mut()
            .spawn((
                Scale2D { x: 16.0, y: 16.0 },
                PreviousScale { x: 16.0, y: 16.0 },
                CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
            ))
            .id();

        app.update();

        assert!(
            app.world().get::<Birthing>(entity_a).is_some(),
            "Bolt entity should get Birthing"
        );
        assert!(
            app.world().get::<Birthing>(entity_b).is_none(),
            "Non-bolt entity should NOT get Birthing"
        );

        // Entity B should be unaffected
        let scale_b = app.world().get::<Scale2D>(entity_b).unwrap();
        assert!(
            (scale_b.x - 16.0).abs() < f32::EPSILON,
            "Non-bolt entity Scale2D.x should be unchanged"
        );
        let layers_b = app.world().get::<CollisionLayers>(entity_b).unwrap();
        assert_eq!(
            layers_b.membership, CELL_LAYER,
            "Non-bolt entity CollisionLayers should be unchanged"
        );
    }

    // Behavior 21: begin_node_birthing zeroes PreviousScale in addition to Scale2D
    #[test]
    fn zeroes_previous_scale() {
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                Scale2D { x: 8.0, y: 8.0 },
                PreviousScale { x: 8.0, y: 8.0 },
                CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
            ))
            .id();

        app.update();

        let scale = app.world().get::<Scale2D>(entity).unwrap();
        assert!(
            scale.x.abs() < f32::EPSILON && scale.y.abs() < f32::EPSILON,
            "Scale2D should be zeroed, got ({}, {})",
            scale.x,
            scale.y
        );

        let prev_scale = app.world().get::<PreviousScale>(entity).unwrap();
        assert!(
            prev_scale.x.abs() < f32::EPSILON && prev_scale.y.abs() < f32::EPSILON,
            "PreviousScale should be zeroed, got ({}, {})",
            prev_scale.x,
            prev_scale.y
        );

        let layers = app.world().get::<CollisionLayers>(entity).unwrap();
        assert_eq!(
            *layers,
            CollisionLayers::default(),
            "CollisionLayers should be zeroed"
        );
    }

    // Edge case: Bolt with zero scale gets Birthing with zero target_scale
    #[test]
    fn bolt_with_zero_scale_gets_zero_target_scale() {
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                Scale2D { x: 0.0, y: 0.0 },
                PreviousScale { x: 0.0, y: 0.0 },
                CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
            ))
            .id();

        app.update();

        let birthing = app
            .world()
            .get::<Birthing>(entity)
            .expect("Bolt should have Birthing");
        assert!(
            birthing.target_scale.x.abs() < f32::EPSILON
                && birthing.target_scale.y.abs() < f32::EPSILON,
            "Zero-scale bolt should have zero target_scale, got ({}, {})",
            birthing.target_scale.x,
            birthing.target_scale.y
        );
    }

    // Edge case: Single bolt entity
    #[test]
    fn single_bolt_entity() {
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                Scale2D { x: 8.0, y: 8.0 },
                PreviousScale { x: 8.0, y: 8.0 },
                CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
            ))
            .id();

        app.update();

        assert!(
            app.world().get::<Birthing>(entity).is_some(),
            "Single bolt entity should get Birthing"
        );
    }
}
