//! Registration of built-in transition effects with the Bevy app.

use bevy::prelude::*;

use super::{
    dissolve::{self, DissolveIn, DissolveOut},
    fade::{self, FadeIn, FadeOut},
    iris::{self, IrisIn, IrisOut},
    pixelate::{self, PixelateIn, PixelateOut},
    slide::{self, Slide},
    wipe::{self, WipeIn, WipeOut},
};
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
    // Post-process shader pipeline
    super::post_process::setup_post_process(app);

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
