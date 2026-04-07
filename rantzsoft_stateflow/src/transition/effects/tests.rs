use bevy::prelude::*;

use super::{
    fade::{self, FadeIn, FadeOut, FadeOutConfig},
    post_process::{EffectType, TransitionEffect},
    shared::TransitionProgress,
    *,
};
use crate::transition::{
    messages::{TransitionOver, TransitionReady, TransitionRunComplete},
    resources::StartingTransition,
};

fn effect_test_app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<TransitionReady>();
    app.add_message::<TransitionRunComplete>();
    app.add_message::<TransitionOver>();
    let camera = app.world_mut().spawn(Camera2d).id();
    (app, camera)
}

// =======================================================================
// Section 13: Shared Progress Tracking
// =======================================================================

// --- Spec Behavior 49: TransitionProgress computes correctly ---

#[test]
fn progress_fraction_computes_correctly() {
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

// --- Spec Behavior 50: Zero-duration effect completes immediately ---

#[test]
fn zero_duration_effect_completes_immediately_via_fade_out() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color: Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction: Vec4::ZERO,
        effect_type: EffectType::FADE,
        progress: 0.0,
    });
    app.insert_resource(crate::transition::resources::RunningTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.0,
        duration: 0.0,
        completed: false,
    });
    app.add_systems(Update, fade::fade_out_run);
    app.update();

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

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert!(
        (effects[0].progress - 1.0).abs() < f32::EPSILON,
        "progress should be 1.0 for zero-duration effect"
    );
}

// =======================================================================
// Section 14: Plugin Registration
// =======================================================================

#[test]
fn all_eleven_effects_are_registered_in_transition_registry() {
    use bevy::state::app::StatesPlugin;

    use crate::{RantzStateflowPlugin, transition::registry::TransitionRegistry};

    #[derive(bevy::prelude::States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum TestState {
        #[default]
        A,
    }

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin));
    app.init_state::<TestState>()
        .add_plugins(RantzStateflowPlugin::new().register_state::<TestState>());
    app.update();

    let registry = app.world().resource::<TransitionRegistry>();
    assert!(registry.contains::<FadeOut>());
    assert!(registry.contains::<FadeIn>());
    assert!(registry.contains::<Slide>());
    assert!(registry.contains::<WipeOut>());
    assert!(registry.contains::<WipeIn>());
    assert!(registry.contains::<IrisOut>());
    assert!(registry.contains::<IrisIn>());
    assert!(registry.contains::<DissolveOut>());
    assert!(registry.contains::<DissolveIn>());
    assert!(registry.contains::<PixelateOut>());
    assert!(registry.contains::<PixelateIn>());
}

// =======================================================================
// Section 18: End System Camera Safety
// =======================================================================

// --- Spec Behavior 89: End system does not panic when camera lacks TransitionEffect ---

#[test]
fn end_system_does_not_panic_when_camera_lacks_transition_effect() {
    let (mut app, _camera) = effect_test_app();
    // Camera exists but has no TransitionEffect
    app.insert_resource(crate::transition::resources::EndingTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.5,
        completed: true,
    });
    app.add_systems(Update, fade::fade_out_end);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionOver>>();
    assert_eq!(
        msgs.iter_current_update_messages().count(),
        1,
        "TransitionOver should still be sent even without TransitionEffect on camera"
    );
    assert!(
        !app.world().contains_resource::<TransitionProgress>(),
        "TransitionProgress should be removed"
    );
}

// --- Spec Behavior 90: End system does not despawn any entities ---

#[test]
fn end_system_does_not_despawn_any_entities() {
    let (mut app, camera) = effect_test_app();
    app.world_mut()
        .entity_mut(camera)
        .insert(TransitionEffect::default());
    let entity_count_before = app.world().entities().len();

    app.insert_resource(crate::transition::resources::EndingTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.5,
        completed: true,
    });
    app.add_systems(Update, fade::fade_out_end);
    app.update();

    let entity_count_after = app.world().entities().len();
    assert_eq!(
        entity_count_before, entity_count_after,
        "no entities should be despawned (only component removed)"
    );
    assert!(
        app.world().get_entity(camera).is_ok(),
        "camera entity should still exist"
    );
}

// =======================================================================
// Section 14 (continued): Built-in effect systems gated on marker resources
// =======================================================================

#[test]
fn inserting_fade_out_marker_causes_only_fade_out_start_to_fire() {
    use bevy::state::app::StatesPlugin;

    use crate::RantzStateflowPlugin;

    #[derive(bevy::prelude::States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum TestState {
        #[default]
        A,
    }

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin));
    app.init_state::<TestState>()
        .add_plugins(RantzStateflowPlugin::new().register_state::<TestState>());
    app.update();

    // Insert FadeOut marker + config
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
// Section 20: Test App Helper Update
// =======================================================================

// --- Spec Behavior 93: effect_test_app() creates app with Camera2d ---

#[test]
fn effect_test_app_has_camera2d_entity() {
    let (mut app, _camera) = effect_test_app();
    let camera_count = app
        .world_mut()
        .query_filtered::<Entity, With<Camera2d>>()
        .iter(app.world())
        .count();
    assert_eq!(
        camera_count, 1,
        "effect_test_app should have a Camera2d entity"
    );
}

#[test]
fn effect_test_app_has_no_screen_size_resource() {
    let (app, _camera) = effect_test_app();
    assert!(
        !app.world().contains_resource::<ScreenSize>(),
        "effect_test_app should NOT have ScreenSize resource"
    );
}
