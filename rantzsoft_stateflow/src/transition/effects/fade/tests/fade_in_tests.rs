use bevy::prelude::*;

use super::{super::effect::*, helpers::effect_test_app};
use crate::transition::{
    effects::{
        post_process::{EffectType, TransitionEffect},
        shared::TransitionProgress,
    },
    messages::{TransitionOver, TransitionReady, TransitionRunComplete},
    resources::{EndingTransition, RunningTransition, StartingTransition},
};

// =======================================================================
// Section 3: FadeIn (post-process)
// =======================================================================

// --- Spec Behavior 14: FadeIn start inserts TransitionEffect with progress 1.0 ---

#[test]
fn fade_in_start_inserts_transition_effect_on_camera_at_full_progress() {
    let (mut app, _camera) = effect_test_app();
    app.insert_resource(FadeInConfig {
        duration: 0.5,
        color:    Color::BLACK,
    });
    app.insert_resource(StartingTransition::<FadeIn>::new());
    app.add_systems(Update, fade_in_start);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(effects.len(), 1);
    let effect = effects[0];
    assert_eq!(
        effect.color,
        Vec4::new(0.0, 0.0, 0.0, 1.0),
        "color should be black (linear RGBA)"
    );
    assert_eq!(effect.direction, Vec4::ZERO);
    assert_eq!(effect.effect_type, EffectType::FADE);
    assert!(
        (effect.progress - 1.0).abs() < f32::EPSILON,
        "FadeIn progress should start at 1.0 (fully opaque)"
    );
}

// --- Spec Behavior 15: FadeIn start sends TransitionReady ---

#[test]
fn fade_in_start_sends_transition_ready() {
    let (mut app, _camera) = effect_test_app();
    app.insert_resource(FadeInConfig {
        duration: 0.5,
        color:    Color::BLACK,
    });
    app.insert_resource(StartingTransition::<FadeIn>::new());
    app.add_systems(Update, fade_in_start);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionReady>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

#[test]
fn fade_in_start_inserts_transition_progress() {
    let (mut app, _camera) = effect_test_app();
    app.insert_resource(FadeInConfig {
        duration: 0.5,
        color:    Color::BLACK,
    });
    app.insert_resource(StartingTransition::<FadeIn>::new());
    app.add_systems(Update, fade_in_start);
    app.update();

    assert!(app.world().contains_resource::<TransitionProgress>());
    let progress = app.world().resource::<TransitionProgress>();
    assert!(progress.elapsed.abs() < f32::EPSILON);
    assert!((progress.duration - 0.5).abs() < f32::EPSILON);
    assert!(!progress.completed);
}

#[test]
fn fade_in_start_removes_config() {
    let (mut app, _camera) = effect_test_app();
    app.insert_resource(FadeInConfig {
        duration: 0.5,
        color:    Color::BLACK,
    });
    app.insert_resource(StartingTransition::<FadeIn>::new());
    app.add_systems(Update, fade_in_start);
    app.update();

    assert!(
        !app.world().contains_resource::<FadeInConfig>(),
        "FadeInConfig should be removed after start"
    );
}

// --- Spec Behavior 16: FadeIn run decreases progress ---

#[test]
fn fade_in_run_decreases_progress_on_camera() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color:       Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction:   Vec4::ZERO,
        effect_type: EffectType::FADE,
        progress:    1.0,
    });
    app.insert_resource(RunningTransition::<FadeIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   0.25,
        duration:  1.0,
        completed: false,
    });
    app.add_systems(Update, fade_in_run);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert!(
        (effects[0].progress - 0.75).abs() < 0.01,
        "progress should be ~0.75 (1.0 - 0.25), got {}",
        effects[0].progress
    );
}

#[test]
fn fade_in_run_clamps_progress_to_zero_on_overshoot() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color:       Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction:   Vec4::ZERO,
        effect_type: EffectType::FADE,
        progress:    1.0,
    });
    app.insert_resource(RunningTransition::<FadeIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   0.5,
        duration:  0.4,
        completed: false,
    });
    app.add_systems(Update, fade_in_run);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert!(
        effects[0].progress >= 0.0,
        "progress should not go negative, got {}",
        effects[0].progress
    );
    assert!(
        effects[0].progress.abs() < f32::EPSILON,
        "progress should be clamped to 0.0"
    );
}

// --- Spec Behavior 17: FadeIn run sends complete at full progress ---

#[test]
fn fade_in_run_sends_complete_at_full_progress() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color:       Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction:   Vec4::ZERO,
        effect_type: EffectType::FADE,
        progress:    1.0,
    });
    app.insert_resource(RunningTransition::<FadeIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   1.0,
        duration:  1.0,
        completed: false,
    });
    app.add_systems(Update, fade_in_run);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert!(
        effects[0].progress.abs() < f32::EPSILON,
        "progress should be 0.0 at full FadeIn"
    );

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

#[test]
fn fade_in_run_does_not_double_send_complete_when_already_completed() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color:       Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction:   Vec4::ZERO,
        effect_type: EffectType::FADE,
        progress:    0.0,
    });
    app.insert_resource(RunningTransition::<FadeIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   1.0,
        duration:  1.0,
        completed: true,
    });
    app.add_systems(Update, fade_in_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(
        msgs.iter_current_update_messages().count(),
        0,
        "should NOT double-send when already completed"
    );
}

// --- Spec Behavior 18: FadeIn end ---

#[test]
fn fade_in_end_removes_transition_effect_and_sends_transition_over() {
    let (mut app, camera) = effect_test_app();
    app.world_mut()
        .entity_mut(camera)
        .insert(TransitionEffect::default());
    app.insert_resource(EndingTransition::<FadeIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   0.5,
        duration:  0.5,
        completed: true,
    });
    app.add_systems(Update, fade_in_end);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(effects.len(), 0, "TransitionEffect should be removed");
    assert!(!app.world().contains_resource::<TransitionProgress>());

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionOver>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

#[test]
fn fade_in_end_does_not_panic_when_camera_lacks_transition_effect() {
    let (mut app, _camera) = effect_test_app();
    // Camera exists but has no TransitionEffect
    app.insert_resource(EndingTransition::<FadeIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   0.5,
        duration:  0.5,
        completed: true,
    });
    app.add_systems(Update, fade_in_end);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionOver>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

// --- Spec Behavior 74: FadeIn default ---

#[test]
fn fade_in_default_duration_is_0_3() {
    let effect = FadeIn::default();
    assert!((effect.duration - 0.3).abs() < f32::EPSILON);
}

#[test]
fn fade_in_default_color_is_black() {
    let effect = FadeIn::default();
    let srgba = effect.color.to_srgba();
    assert!(srgba.red.abs() < f32::EPSILON);
    assert!(srgba.green.abs() < f32::EPSILON);
    assert!(srgba.blue.abs() < f32::EPSILON);
}

// --- Spec Behavior 63: FadeIn trait satisfaction ---

#[test]
fn fade_in_satisfies_transition_and_in_transition() {
    use crate::transition::traits::InTransition;
    let _effect: Box<dyn InTransition> = Box::new(FadeIn {
        duration: 0.5,
        color:    Color::BLACK,
    });
}

// --- Spec Behavior 52: FadeIn insert_starting ---

#[test]
fn fade_in_insert_starting_inserts_marker_and_config() {
    use crate::transition::traits::Transition;
    let mut world = World::new();
    let effect = FadeIn {
        duration: 0.7,
        color:    Color::srgba(1.0, 0.0, 0.0, 1.0),
    };
    effect.insert_starting(&mut world);

    assert!(world.contains_resource::<StartingTransition<FadeIn>>());
    assert!(world.contains_resource::<FadeInConfig>());
}
