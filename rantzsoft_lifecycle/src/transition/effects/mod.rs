//! Built-in transition effects.
//!
//! Each module defines a pair of transition effects (In/Out or `OneShot`)
//! with their config resources and start/run/end systems.

pub mod dissolve;
pub mod fade;
pub mod iris;
pub mod pixelate;
pub mod shared;
pub mod slide;
pub mod wipe;

// Public re-exports — effect structs
use bevy::prelude::*;
pub use dissolve::{DissolveIn, DissolveInConfig, DissolveOut, DissolveOutConfig};
pub use fade::{FadeIn, FadeInConfig, FadeOut, FadeOutConfig};
pub use iris::{IrisIn, IrisInConfig, IrisOut, IrisOutConfig};
pub use pixelate::{PixelateIn, PixelateInConfig, PixelateOut, PixelateOutConfig};
pub use shared::{ScreenSize, TransitionOverlay, TransitionProgress, WipeDirection};
pub use slide::{Slide, SlideConfig, SlideDirection};
pub use wipe::{WipeIn, WipeInConfig, WipeOut, WipeOutConfig};

use crate::transition::{
    orchestration::orchestrate_transitions,
    registry::TransitionRegistry,
    resources::{EndingTransition, RunningTransition, StartingTransition},
    traits::Transition,
};

/// Register all built-in transition effects with the app.
///
/// Called from `RantzLifecyclePlugin::build`.
pub(crate) fn register_builtin_transitions(app: &mut App) {
    app.init_resource::<ScreenSize>();

    // Fade
    register_effect::<FadeOut, _, _, _>(
        app,
        fade::fade_out_start,
        fade::fade_out_run,
        fade::fade_out_end,
    );
    register_effect::<FadeIn, _, _, _>(
        app,
        fade::fade_in_start,
        fade::fade_in_run,
        fade::fade_in_end,
    );

    // Dissolve
    register_effect::<DissolveOut, _, _, _>(
        app,
        dissolve::dissolve_out_start,
        dissolve::dissolve_out_run,
        dissolve::dissolve_out_end,
    );
    register_effect::<DissolveIn, _, _, _>(
        app,
        dissolve::dissolve_in_start,
        dissolve::dissolve_in_run,
        dissolve::dissolve_in_end,
    );

    // Pixelate
    register_effect::<PixelateOut, _, _, _>(
        app,
        pixelate::pixelate_out_start,
        pixelate::pixelate_out_run,
        pixelate::pixelate_out_end,
    );
    register_effect::<PixelateIn, _, _, _>(
        app,
        pixelate::pixelate_in_start,
        pixelate::pixelate_in_run,
        pixelate::pixelate_in_end,
    );

    // Wipe
    register_effect::<WipeOut, _, _, _>(
        app,
        wipe::wipe_out_start,
        wipe::wipe_out_run,
        wipe::wipe_out_end,
    );
    register_effect::<WipeIn, _, _, _>(
        app,
        wipe::wipe_in_start,
        wipe::wipe_in_run,
        wipe::wipe_in_end,
    );

    // Iris
    register_effect::<IrisOut, _, _, _>(
        app,
        iris::iris_out_start,
        iris::iris_out_run,
        iris::iris_out_end,
    );
    register_effect::<IrisIn, _, _, _>(
        app,
        iris::iris_in_start,
        iris::iris_in_run,
        iris::iris_in_end,
    );

    // Slide
    register_effect::<Slide, _, _, _>(app, slide::slide_start, slide::slide_run, slide::slide_end);
}

/// Register a single transition effect type with its three phase systems.
fn register_effect<T: Transition, M1, M2, M3>(
    app: &mut App,
    start_system: impl IntoSystem<(), (), M1> + Send + Sync + 'static,
    run_system: impl IntoSystem<(), (), M2> + Send + Sync + 'static,
    end_system: impl IntoSystem<(), (), M3> + Send + Sync + 'static,
) {
    app.world_mut()
        .resource_mut::<TransitionRegistry>()
        .register::<T>();

    app.add_systems(
        Update,
        (
            start_system
                .run_if(resource_exists::<StartingTransition<T>>)
                .before(orchestrate_transitions),
            run_system
                .run_if(resource_exists::<RunningTransition<T>>)
                .before(orchestrate_transitions),
            end_system
                .run_if(resource_exists::<EndingTransition<T>>)
                .before(orchestrate_transitions),
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::{
        shared::{TransitionOverlay, TransitionProgress},
        *,
    };
    use crate::transition::{
        messages::{TransitionOver, TransitionReady, TransitionRunComplete},
        resources::StartingTransition,
    };

    fn effect_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<TransitionReady>();
        app.add_message::<TransitionRunComplete>();
        app.add_message::<TransitionOver>();
        app.insert_resource(ScreenSize::default());
        app
    }

    // =======================================================================
    // Section 14: Plugin Registration (behaviors 69-70)
    // =======================================================================

    // --- Behavior 69: All 11 effects registered in TransitionRegistry ---

    #[test]
    fn all_eleven_effects_are_registered_in_transition_registry() {
        use bevy::state::app::StatesPlugin;

        use crate::{RantzLifecyclePlugin, transition::registry::TransitionRegistry};

        #[derive(bevy::prelude::States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
        enum TestState {
            #[default]
            A,
        }

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<TestState>()
            .add_plugins(RantzLifecyclePlugin::new().register_state::<TestState>());
        app.update();

        let registry = app.world().resource::<TransitionRegistry>();
        assert!(
            registry.contains::<FadeOut>(),
            "TransitionRegistry should contain FadeOut"
        );
        assert!(
            registry.contains::<FadeIn>(),
            "TransitionRegistry should contain FadeIn"
        );
        assert!(
            registry.contains::<Slide>(),
            "TransitionRegistry should contain Slide"
        );
        assert!(
            registry.contains::<WipeOut>(),
            "TransitionRegistry should contain WipeOut"
        );
        assert!(
            registry.contains::<WipeIn>(),
            "TransitionRegistry should contain WipeIn"
        );
        assert!(
            registry.contains::<IrisOut>(),
            "TransitionRegistry should contain IrisOut"
        );
        assert!(
            registry.contains::<IrisIn>(),
            "TransitionRegistry should contain IrisIn"
        );
        assert!(
            registry.contains::<DissolveOut>(),
            "TransitionRegistry should contain DissolveOut"
        );
        assert!(
            registry.contains::<DissolveIn>(),
            "TransitionRegistry should contain DissolveIn"
        );
        assert!(
            registry.contains::<PixelateOut>(),
            "TransitionRegistry should contain PixelateOut"
        );
        assert!(
            registry.contains::<PixelateIn>(),
            "TransitionRegistry should contain PixelateIn"
        );
    }

    // --- Behavior 70: Built-in effect systems are gated on marker resources ---

    #[test]
    fn inserting_fade_out_marker_causes_only_fade_out_start_to_fire() {
        use bevy::state::app::StatesPlugin;

        use crate::RantzLifecyclePlugin;

        #[derive(bevy::prelude::States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
        enum TestState {
            #[default]
            A,
        }

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<TestState>()
            .add_plugins(RantzLifecyclePlugin::new().register_state::<TestState>());
        app.update();

        // Insert FadeOut marker + config + screen size (already default)
        app.insert_resource(StartingTransition::<FadeOut>::new());
        app.insert_resource(FadeOutConfig {
            duration: 0.5,
            color: Color::BLACK,
        });
        app.update();

        // FadeOut start system should have fired and sent TransitionReady
        let ready_msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionReady>>();
        assert_eq!(
            ready_msgs.iter_current_update_messages().count(),
            1,
            "FadeOut start system should send exactly 1 TransitionReady"
        );

        // No other messages should be sent (only start system fired)
        let run_msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
        assert_eq!(
            run_msgs.iter_current_update_messages().count(),
            0,
            "no TransitionRunComplete should be sent (only start phase)"
        );
    }

    // =======================================================================
    // Section 15: Progress Tracking Shared Behavior (behaviors 71-73)
    // =======================================================================

    // --- Behavior 71: TransitionProgress initialized to elapsed 0.0 ---
    // Covered by individual effect start tests which check:
    //   progress.elapsed == 0.0, progress.completed == false

    // --- Behavior 72: Progress is elapsed / duration clamped to [0.0, 1.0] ---

    #[test]
    fn progress_fraction_computes_correctly() {
        // This is a unit test on the shared computation pattern.
        // Each effect's run system should compute: progress = (elapsed / duration).clamp(0.0, 1.0)
        let progress = TransitionProgress {
            elapsed: 0.3,
            duration: 0.5,
            completed: false,
        };
        let fraction = (progress.elapsed / progress.duration).clamp(0.0, 1.0);
        assert!(
            (fraction - 0.6).abs() < f32::EPSILON,
            "0.3 / 0.5 should be 0.6, got {fraction}"
        );
    }

    #[test]
    fn progress_fraction_clamps_to_one_on_overshoot() {
        let progress = TransitionProgress {
            elapsed: 0.6,
            duration: 0.5,
            completed: false,
        };
        let fraction = (progress.elapsed / progress.duration).clamp(0.0, 1.0);
        assert!(
            (fraction - 1.0).abs() < f32::EPSILON,
            "overshooting should clamp to 1.0"
        );
    }

    // --- Behavior 73: Zero-duration effect completes immediately ---

    #[test]
    fn zero_duration_effect_completes_immediately_via_fade_out() {
        // Use FadeOut as the representative effect for zero-duration behavior.
        let mut app = effect_test_app();
        app.insert_resource(crate::transition::resources::RunningTransition::<FadeOut>::new());
        app.insert_resource(TransitionProgress {
            elapsed: 0.0,
            duration: 0.0,
            completed: false,
        });
        app.world_mut().spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(1920.0, 1080.0)),
                ..default()
            },
            TransitionOverlay,
        ));
        app.add_systems(Update, fade::fade_out_run);
        app.update();

        // For zero duration, progress should be 1.0 (not NaN) and complete should fire.
        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
        assert_eq!(
            msgs.iter_current_update_messages().count(),
            1,
            "zero-duration effect should send TransitionRunComplete on first frame"
        );

        let progress = app.world().resource::<TransitionProgress>();
        assert!(
            progress.completed,
            "zero-duration effect should set completed = true"
        );
    }

    // =======================================================================
    // Section 16: Overlay Entity Marker (behaviors 74-75)
    // =======================================================================

    // --- Behavior 74: All overlay entities tagged with TransitionOverlay ---
    // Covered by every start system test that checks for TransitionOverlay component.

    // --- Behavior 75: End system despawns only TransitionOverlay entities ---

    #[test]
    fn end_system_only_despawns_overlay_entities_not_others() {
        let mut app = effect_test_app();
        app.insert_resource(crate::transition::resources::EndingTransition::<FadeOut>::new());
        app.insert_resource(TransitionProgress {
            elapsed: 0.5,
            duration: 0.5,
            completed: true,
        });

        // Spawn an overlay entity and a non-overlay entity
        app.world_mut()
            .spawn((Sprite::default(), TransitionOverlay));
        let non_overlay = app.world_mut().spawn(Camera2d).id();

        app.add_systems(Update, fade::fade_out_end);
        app.update();

        let overlay_count = app
            .world_mut()
            .query_filtered::<Entity, With<TransitionOverlay>>()
            .iter(app.world())
            .count();
        assert_eq!(overlay_count, 0, "overlay should be despawned");

        assert!(
            app.world().get_entity(non_overlay).is_ok(),
            "non-overlay entity (camera) should NOT be despawned"
        );
    }

    #[test]
    fn end_system_despawns_all_overlay_entities_if_multiple_exist() {
        let mut app = effect_test_app();
        app.insert_resource(crate::transition::resources::EndingTransition::<FadeOut>::new());
        app.insert_resource(TransitionProgress {
            elapsed: 0.5,
            duration: 0.5,
            completed: true,
        });

        // Spawn two overlay entities
        app.world_mut()
            .spawn((Sprite::default(), TransitionOverlay));
        app.world_mut()
            .spawn((Sprite::default(), TransitionOverlay));

        app.add_systems(Update, fade::fade_out_end);
        app.update();

        let overlay_count = app
            .world_mut()
            .query_filtered::<Entity, With<TransitionOverlay>>()
            .iter(app.world())
            .count();
        assert_eq!(overlay_count, 0, "all overlay entities should be despawned");
    }
}
