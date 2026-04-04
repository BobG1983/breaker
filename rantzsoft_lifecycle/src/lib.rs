//! Declarative state routing, screen transitions, and lifecycle messages
//! for Bevy 0.18 games.
//!
//! Provides:
//! - [`Route`] builder API for declaring state-to-state transitions
//! - [`RoutingTable<S>`] resource per state type
//! - Message-triggered and condition-triggered dispatch systems
//! - [`CleanupOnExit<S>`] component for state-scoped entity cleanup
//! - [`ChangeState<S>`] / [`StateChanged<S>`] lifecycle messages
//!
//! Zero game knowledge — this crate contains no game-specific types,
//! vocabulary, or configuration.

pub mod cleanup;
pub mod dispatch;
pub mod messages;
mod plugin;
pub mod route;
pub mod routing_table;
pub mod transition;

pub use cleanup::{CleanupOnExit, cleanup_on_exit};
pub use messages::{ChangeState, StateChanged, TransitionEnd, TransitionStart};
pub use plugin::RantzLifecyclePlugin;
pub use route::Route;
pub use routing_table::{RoutingTable, RoutingTableAppExt};
pub use transition::{
    effects::{
        DissolveIn, DissolveInConfig, DissolveOut, DissolveOutConfig, FadeIn, FadeInConfig,
        FadeOut, FadeOutConfig, IrisIn, IrisInConfig, IrisOut, IrisOutConfig, PixelateIn,
        PixelateInConfig, PixelateOut, PixelateOutConfig, ScreenSize, Slide, SlideConfig,
        SlideDirection, TransitionOverlay, TransitionProgress, WipeDirection, WipeIn, WipeInConfig,
        WipeOut, WipeOutConfig,
    },
    registry::TransitionRegistry,
    resources::{ActiveTransition, EndingTransition, RunningTransition, StartingTransition},
    traits::{InTransition, OneShotTransition, OutTransition, Transition},
    types::TransitionType,
};

#[cfg(test)]
mod integration_tests {
    use std::sync::Arc;

    use bevy::{prelude::*, state::app::StatesPlugin};

    use crate::{
        ChangeState, RantzLifecyclePlugin, Route, RoutingTableAppExt,
        transition::{
            effects::shared::TransitionProgress,
            resources::{ActiveTransition, PendingTransition},
        },
    };

    #[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum TestState {
        #[default]
        A,
        B,
    }

    /// Build a minimal app with the lifecycle plugin and a registered test state.
    fn integration_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<TestState>()
            .add_plugins(RantzLifecyclePlugin::new().register_state::<TestState>());
        app
    }

    /// Drive the app forward until the transition completes, by force-completing
    /// the progress each frame. Returns the number of updates run.
    fn drive_transition_to_completion(app: &mut App, max_frames: usize) -> usize {
        for frame in 0..max_frames {
            // If there is a TransitionProgress resource, force it past duration
            if app.world().contains_resource::<TransitionProgress>() {
                let mut progress = app.world_mut().resource_mut::<TransitionProgress>();
                if !progress.completed {
                    progress.elapsed = progress.duration + 0.1;
                }
            }
            app.update();

            // Check if transition has completed
            if !app.world().contains_resource::<ActiveTransition>()
                && !app.world().contains_resource::<PendingTransition>()
            {
                return frame + 1;
            }
        }
        max_frames
    }

    // --- Test 1: Route with FadeIn transition completes full lifecycle ---

    #[test]
    fn route_with_fade_in_transition_completes_full_lifecycle() {
        use crate::FadeIn;

        let mut app = integration_app();

        app.add_route(
            Route::from(TestState::A)
                .to(TestState::B)
                .with_transition(crate::TransitionType::In(Arc::new(FadeIn::default())))
                .when(|_| true),
        );

        // Initial update — state is A, condition should fire
        app.update();

        // Drive transition to completion
        let frames = drive_transition_to_completion(&mut app, 20);
        assert!(frames < 20, "transition should complete within 20 frames");

        // One more update to apply the state change
        app.update();

        // Assert: State<TestState> == B
        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::B,
            "state should be B after transition completes"
        );

        // Assert: ActiveTransition removed
        assert!(
            !app.world().contains_resource::<ActiveTransition>(),
            "ActiveTransition should be removed after transition completes"
        );

        // Assert: Time<Virtual> unpaused
        assert!(
            !app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should be unpaused after transition completes"
        );
    }

    // --- Test 2: Route with OutIn transition completes full lifecycle ---

    #[test]
    fn route_with_out_in_transition_completes_full_lifecycle() {
        use crate::{FadeIn, FadeOut};

        let mut app = integration_app();

        app.add_route(
            Route::from(TestState::A)
                .to(TestState::B)
                .with_transition(crate::TransitionType::OutIn {
                    out_e: Arc::new(FadeOut::default()),
                    in_e: Arc::new(FadeIn::default()),
                })
                .when(|_| true),
        );

        // Initial update
        app.update();

        // Drive transition to completion (OutIn has two phases)
        let frames = drive_transition_to_completion(&mut app, 30);
        assert!(
            frames < 30,
            "OutIn transition should complete within 30 frames"
        );

        // One more update to ensure state is applied
        app.update();

        // Assert: State<TestState> == B
        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::B,
            "state should be B after OutIn transition completes"
        );

        // Assert: ActiveTransition removed
        assert!(
            !app.world().contains_resource::<ActiveTransition>(),
            "ActiveTransition should be removed after OutIn transition completes"
        );

        // Assert: Time<Virtual> unpaused
        assert!(
            !app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should be unpaused after OutIn transition completes"
        );
    }

    // --- Test 3: Plain route (no transition) still changes state ---

    #[test]
    fn plain_route_without_transition_changes_state() {
        let mut app = integration_app();

        // Route without .with_transition()
        app.add_route(Route::from(TestState::A).to(TestState::B));

        app.update();

        // Trigger via ChangeState message
        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<ChangeState<TestState>>>()
            .write(ChangeState::new());
        app.update();
        app.update();

        // Assert: State changed
        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::B,
            "plain route should still change state"
        );

        // Assert: No ActiveTransition was inserted
        assert!(
            !app.world().contains_resource::<ActiveTransition>(),
            "no ActiveTransition should be inserted for plain routes"
        );

        // Assert: No TransitionStart sent (check current frame messages)
        // Note: messages may have been consumed by now, but ActiveTransition
        // absence confirms no transition was started.
    }
}
