//! System that triggers `AnimateIn` -> `Playing` transition when all birthing
//! animations have completed.

use bevy::prelude::*;
use rantzsoft_stateflow::ChangeState;

use crate::prelude::*;

/// Sends [`ChangeState<NodeState>`] when no entities have [`Birthing`],
/// signaling that all bolt birthing animations have completed and gameplay
/// can begin.
pub(crate) fn all_animate_in_complete(
    query: Query<(), With<Birthing>>,
    mut state_writer: MessageWriter<ChangeState<NodeState>>,
) {
    if query.is_empty() {
        state_writer.write(ChangeState::new());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::birthing::BIRTHING_DURATION;

    fn test_app() -> App {
        TestAppBuilder::new()
            .with_message::<ChangeState<NodeState>>()
            .with_system(Update, all_animate_in_complete)
            .build()
    }

    // Behavior 24: all_animate_in_complete sends ChangeState when no Birthing entities remain
    #[test]
    fn sends_change_state_when_no_birthing_entities() {
        let mut app = test_app();

        // No entities with Birthing at all
        app.update();

        let messages = app.world().resource::<Messages<ChangeState<NodeState>>>();
        assert!(
            messages.iter_current_update_messages().count() > 0,
            "ChangeState<NodeState> should be sent when no Birthing entities remain"
        );
    }

    // Edge case: Zero bolt entities total -- should still send ChangeState
    #[test]
    fn sends_change_state_with_zero_entities() {
        let mut app = test_app();

        // Completely empty world (besides MinimalPlugins resources)
        app.update();

        let messages = app.world().resource::<Messages<ChangeState<NodeState>>>();
        assert!(
            messages.iter_current_update_messages().count() > 0,
            "ChangeState should be sent even with zero entities"
        );
    }

    // Behavior 25: all_animate_in_complete does NOT send ChangeState while Birthing entities exist
    #[test]
    fn does_not_send_change_state_while_birthing_exists() {
        use rantzsoft_physics2d::collision_layers::CollisionLayers;
        use rantzsoft_spatial2d::components::Scale2D;

        let mut app = test_app();

        app.world_mut().spawn((
            Scale2D { x: 0.0, y: 0.0 },
            CollisionLayers::default(),
            Birthing {
                timer:          Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
                target_scale:   Scale2D { x: 8.0, y: 8.0 },
                stashed_layers: CollisionLayers::new(0x01, 0x0E),
            },
        ));

        app.update();

        let messages = app.world().resource::<Messages<ChangeState<NodeState>>>();
        assert_eq!(
            messages.iter_current_update_messages().count(),
            0,
            "ChangeState should NOT be sent while Birthing entities exist"
        );
    }

    // Edge case: Multiple birthing entities, one completes -- should NOT send until ALL complete
    #[test]
    fn does_not_send_until_all_birthing_complete() {
        use rantzsoft_physics2d::collision_layers::CollisionLayers;
        use rantzsoft_spatial2d::components::Scale2D;

        let mut app = test_app();

        // Two birthing entities
        app.world_mut().spawn((
            Scale2D { x: 0.0, y: 0.0 },
            CollisionLayers::default(),
            Birthing {
                timer:          Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
                target_scale:   Scale2D { x: 8.0, y: 8.0 },
                stashed_layers: CollisionLayers::new(0x01, 0x0E),
            },
        ));

        app.world_mut().spawn((
            Scale2D { x: 0.0, y: 0.0 },
            CollisionLayers::default(),
            Birthing {
                timer:          Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
                target_scale:   Scale2D { x: 16.0, y: 16.0 },
                stashed_layers: CollisionLayers::new(0x02, 0x0D),
            },
        ));

        app.update();

        let messages = app.world().resource::<Messages<ChangeState<NodeState>>>();
        assert_eq!(
            messages.iter_current_update_messages().count(),
            0,
            "ChangeState should NOT be sent while multiple Birthing entities exist"
        );
    }

    // Behavior 26: all_animate_in_complete sends ChangeState -- no once-guard needed
    #[test]
    fn sends_change_state_no_once_guard_needed() {
        let mut app = test_app();

        // No birthing entities
        app.update();

        let messages = app.world().resource::<Messages<ChangeState<NodeState>>>();
        assert!(
            messages.iter_current_update_messages().count() > 0,
            "ChangeState should be sent when no Birthing entities exist"
        );
    }

    // Behavior 28: Full AnimateIn lifecycle integration test
    //
    // begin_node_birthing runs once (OnEnter), then tick_birthing + all_animate_in_complete
    // run each FixedUpdate frame until birthing completes and ChangeState is sent.
    #[test]
    fn full_animate_in_lifecycle() {
        use rantzsoft_physics2d::collision_layers::CollisionLayers;
        use rantzsoft_spatial2d::components::{PreviousScale, Scale2D};

        use crate::bolt::systems::tick_birthing;

        fn tick_fixed(app: &mut App) {
            let timestep = app.world().resource::<Time<Fixed>>().timestep();
            app.world_mut()
                .resource_mut::<Time<Fixed>>()
                .accumulate_overstep(timestep);
            app.update();
        }

        // Phase 1: Simulate begin_node_birthing (OnEnter) by setting up the entity
        // with Birthing, zeroed scale, and zeroed layers -- as begin_node_birthing would.
        let mut app = TestAppBuilder::new()
            .with_message::<ChangeState<NodeState>>()
            .with_system(
                FixedUpdate,
                (tick_birthing, all_animate_in_complete).chain(),
            )
            .build();

        // Spawn bolt as begin_node_birthing would leave it: Birthing inserted,
        // scale/layers zeroed, original values stashed in Birthing.
        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                Scale2D { x: 0.0, y: 0.0 },
                PreviousScale { x: 0.0, y: 0.0 },
                CollisionLayers::default(),
                Birthing {
                    timer:          Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
                    target_scale:   Scale2D { x: 8.0, y: 8.0 },
                    stashed_layers: CollisionLayers::new(
                        BOLT_LAYER,
                        CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
                    ),
                },
            ))
            .id();

        // First few ticks: Birthing still present, no ChangeState
        tick_fixed(&mut app);
        assert!(
            app.world().get::<Birthing>(entity).is_some(),
            "Birthing should still be present after 1 tick"
        );

        // Tick enough for full completion (20+ ticks)
        for _ in 0..21 {
            tick_fixed(&mut app);
        }

        // Phase 3: Verify completion
        let scale = app.world().get::<Scale2D>(entity).expect("entity exists");
        assert!(
            (scale.x - 8.0).abs() < f32::EPSILON,
            "Scale2D.x should be restored to 8.0, got {}",
            scale.x
        );
        assert!(
            (scale.y - 8.0).abs() < f32::EPSILON,
            "Scale2D.y should be restored to 8.0, got {}",
            scale.y
        );

        let layers = app.world().get::<CollisionLayers>(entity).unwrap();
        assert_eq!(
            *layers,
            CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
            "CollisionLayers should be restored after birthing completion"
        );

        assert!(
            app.world().get::<Birthing>(entity).is_none(),
            "Birthing should be removed after completion"
        );

        // ChangeState should have been sent by all_animate_in_complete
        // (may be one frame after Birthing removal due to command deferral -- acceptable)
        // Run one more update to allow the system to observe the removal
        tick_fixed(&mut app);

        let messages = app.world().resource::<Messages<ChangeState<NodeState>>>();
        assert!(
            messages.iter_current_update_messages().count() > 0,
            "ChangeState<NodeState> should be sent after all birthing completes"
        );
    }
}
