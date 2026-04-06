use bevy::prelude::*;

// Re-import the effect types and system fns from the parent module.
use super::super::effect::*;
use super::helpers::effect_test_app;
use crate::transition::{
    effects::{
        post_process::{EffectType, TransitionEffect},
        shared::TransitionProgress,
    },
    messages::{TransitionOver, TransitionReady, TransitionRunComplete},
    resources::{EndingTransition, RunningTransition, StartingTransition},
};

// =======================================================================
// Section 2: FadeOut (post-process)
// =======================================================================

// --- Spec Behavior 5: FadeOut start inserts TransitionEffect on camera ---

#[test]
fn fade_out_start_inserts_transition_effect_on_camera() {
    let (mut app, _camera) = effect_test_app();
    app.insert_resource(FadeOutConfig {
        duration: 0.5,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<FadeOut>::new());
    app.add_systems(Update, fade_out_start);
    app.update();

    let effects: Vec<(&TransitionEffect, Entity)> = app
        .world_mut()
        .query_filtered::<(&TransitionEffect, Entity), With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(
        effects.len(),
        1,
        "exactly 1 camera entity should have TransitionEffect"
    );
    let (effect, _) = effects[0];
    assert_eq!(
        effect.color,
        Vec4::new(0.0, 0.0, 0.0, 1.0),
        "color should be black (linear RGBA)"
    );
    assert_eq!(effect.direction, Vec4::ZERO);
    assert_eq!(effect.effect_type, EffectType::FADE);
    assert!(
        effect.progress.abs() < f32::EPSILON,
        "FadeOut progress should start at 0.0"
    );
}

#[test]
fn fade_out_start_does_not_spawn_new_entities() {
    let (mut app, _camera) = effect_test_app();
    // Count entities before
    let count_before = app.world().entities().len();
    app.insert_resource(FadeOutConfig {
        duration: 0.5,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<FadeOut>::new());
    app.add_systems(Update, fade_out_start);
    app.update();

    let count_after = app.world().entities().len();
    assert_eq!(
        count_before, count_after,
        "no new entities should be spawned (TransitionEffect goes on existing camera)"
    );
}

// --- Spec Behavior 6: FadeOut start sends TransitionReady and inserts TransitionProgress ---

#[test]
fn fade_out_start_sends_transition_ready() {
    let (mut app, _camera) = effect_test_app();
    app.insert_resource(FadeOutConfig {
        duration: 0.5,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<FadeOut>::new());
    app.add_systems(Update, fade_out_start);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionReady>>();
    let count = msgs.iter_current_update_messages().count();
    assert_eq!(count, 1, "exactly 1 TransitionReady should be sent");
}

#[test]
fn fade_out_start_inserts_transition_progress() {
    let (mut app, _camera) = effect_test_app();
    app.insert_resource(FadeOutConfig {
        duration: 0.5,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<FadeOut>::new());
    app.add_systems(Update, fade_out_start);
    app.update();

    assert!(
        app.world().contains_resource::<TransitionProgress>(),
        "TransitionProgress should be inserted by start system"
    );
    let progress = app.world().resource::<TransitionProgress>();
    assert!(
        progress.elapsed.abs() < f32::EPSILON,
        "elapsed should be 0.0"
    );
    assert!(
        (progress.duration - 0.5).abs() < f32::EPSILON,
        "duration should match config"
    );
    assert!(!progress.completed, "completed should be false");
}

#[test]
fn fade_out_start_removes_config_resource() {
    let (mut app, _camera) = effect_test_app();
    app.insert_resource(FadeOutConfig {
        duration: 0.5,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<FadeOut>::new());
    app.add_systems(Update, fade_out_start);
    app.update();

    assert!(
        !app.world().contains_resource::<FadeOutConfig>(),
        "FadeOutConfig should be removed after start"
    );
}

// --- Spec Behavior 7: FadeOut start with custom red color ---

#[test]
fn fade_out_start_with_red_color_preserves_color_in_transition_effect() {
    let (mut app, _camera) = effect_test_app();
    app.insert_resource(FadeOutConfig {
        duration: 0.5,
        color: Color::srgba(1.0, 0.0, 0.0, 1.0),
    });
    app.insert_resource(StartingTransition::<FadeOut>::new());
    app.add_systems(Update, fade_out_start);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(effects.len(), 1);
    let color = effects[0].color;
    // Color is stored as linear RGBA; red channel should be ~1.0
    assert!(color.x > 0.9, "red channel should be ~1.0, got {}", color.x);
    assert!(color.y < 0.01, "green channel should be ~0.0");
    assert!(color.z < 0.01, "blue channel should be ~0.0");
    assert!(
        (color.w - 1.0).abs() < 0.01,
        "alpha should be ~1.0, got {}",
        color.w
    );
}

// --- Spec Behavior 5 Edge Case: camera already has stale TransitionEffect ---

#[test]
fn fade_out_start_overwrites_stale_transition_effect_on_camera() {
    let (mut app, camera) = effect_test_app();
    // Pre-insert a stale TransitionEffect on the camera
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        direction: Vec4::new(1.0, 0.0, 0.0, 0.0),
        effect_type: EffectType::WIPE,
        progress: 0.75,
    });

    app.insert_resource(FadeOutConfig {
        duration: 0.5,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<FadeOut>::new());
    app.add_systems(Update, fade_out_start);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(effects.len(), 1);
    let effect = effects[0];
    assert_eq!(
        effect.effect_type,
        EffectType::FADE,
        "should be overwritten to FADE"
    );
    assert!(
        effect.progress.abs() < f32::EPSILON,
        "progress should be reset to 0.0"
    );
}

// --- Spec Behavior 8: FadeOut run updates TransitionEffect.progress ---

#[test]
fn fade_out_run_updates_progress_on_camera() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color: Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction: Vec4::ZERO,
        effect_type: EffectType::FADE,
        progress: 0.0,
    });
    app.insert_resource(RunningTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.25,
        duration: 1.0,
        completed: false,
    });
    app.add_systems(Update, fade_out_run);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert!(
        (effects[0].progress - 0.25).abs() < 0.01,
        "progress should be ~0.25, got {}",
        effects[0].progress
    );
}

// --- Spec Behavior 9: FadeOut run does not send complete when in progress ---

#[test]
fn fade_out_run_does_not_send_complete_when_in_progress() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color: Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction: Vec4::ZERO,
        effect_type: EffectType::FADE,
        progress: 0.0,
    });
    app.insert_resource(RunningTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.25,
        duration: 1.0,
        completed: false,
    });
    app.add_systems(Update, fade_out_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(
        msgs.iter_current_update_messages().count(),
        0,
        "TransitionRunComplete should NOT be sent when progress < 1.0"
    );
}

// --- Spec Behavior 10: FadeOut run sends complete at elapsed >= duration ---

#[test]
fn fade_out_run_sends_complete_when_elapsed_equals_duration() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color: Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction: Vec4::ZERO,
        effect_type: EffectType::FADE,
        progress: 0.0,
    });
    app.insert_resource(RunningTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.5,
        completed: false,
    });
    app.add_systems(Update, fade_out_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(
        msgs.iter_current_update_messages().count(),
        1,
        "TransitionRunComplete should be sent when elapsed >= duration"
    );

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert!(
        (effects[0].progress - 1.0).abs() < f32::EPSILON,
        "progress should be 1.0 at completion"
    );

    let progress = app.world().resource::<TransitionProgress>();
    assert!(progress.completed, "completed flag should be true");
}

#[test]
fn fade_out_run_clamps_progress_to_one_when_overshooting() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color: Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction: Vec4::ZERO,
        effect_type: EffectType::FADE,
        progress: 0.0,
    });
    app.insert_resource(RunningTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.6,
        duration: 0.5,
        completed: false,
    });
    app.add_systems(Update, fade_out_run);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert!(
        (effects[0].progress - 1.0).abs() < f32::EPSILON,
        "progress should be clamped to 1.0, got {}",
        effects[0].progress
    );
}

// --- Spec Behavior 11: FadeOut run does not double-send ---

#[test]
fn fade_out_run_does_not_double_send_complete_when_already_completed() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color: Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction: Vec4::ZERO,
        effect_type: EffectType::FADE,
        progress: 1.0,
    });
    app.insert_resource(RunningTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: true,
    });
    app.add_systems(Update, fade_out_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(
        msgs.iter_current_update_messages().count(),
        0,
        "should NOT double-send TransitionRunComplete when already completed"
    );
}

// --- Spec Behavior 12: FadeOut end removes TransitionEffect ---

#[test]
fn fade_out_end_removes_transition_effect_from_camera() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color: Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction: Vec4::ZERO,
        effect_type: EffectType::FADE,
        progress: 1.0,
    });
    app.insert_resource(EndingTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.5,
        completed: true,
    });
    app.add_systems(Update, fade_out_end);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(
        effects.len(),
        0,
        "TransitionEffect should be removed from camera"
    );
    assert!(
        !app.world().contains_resource::<TransitionProgress>(),
        "TransitionProgress should be removed"
    );
}

#[test]
fn fade_out_end_sends_transition_over() {
    let (mut app, camera) = effect_test_app();
    app.world_mut()
        .entity_mut(camera)
        .insert(TransitionEffect::default());
    app.insert_resource(EndingTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.5,
        completed: true,
    });
    app.add_systems(Update, fade_out_end);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionOver>>();
    assert_eq!(
        msgs.iter_current_update_messages().count(),
        1,
        "exactly 1 TransitionOver should be sent"
    );
}

// --- Spec Behavior 13: FadeOut end does not despawn camera ---

#[test]
fn fade_out_end_does_not_despawn_camera_entity() {
    let (mut app, camera) = effect_test_app();
    app.world_mut()
        .entity_mut(camera)
        .insert(TransitionEffect::default());
    app.insert_resource(EndingTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.5,
        completed: true,
    });
    app.add_systems(Update, fade_out_end);
    app.update();

    assert!(
        app.world().get_entity(camera).is_ok(),
        "camera entity should still exist after end system"
    );
}

// --- Spec Behavior 73: FadeOut default ---

#[test]
fn fade_out_default_duration_is_0_3() {
    let effect = FadeOut::default();
    assert!(
        (effect.duration - 0.3).abs() < f32::EPSILON,
        "default duration should be 0.3"
    );
}

#[test]
fn fade_out_default_color_is_black() {
    let effect = FadeOut::default();
    let srgba = effect.color.to_srgba();
    assert!(srgba.red.abs() < f32::EPSILON);
    assert!(srgba.green.abs() < f32::EPSILON);
    assert!(srgba.blue.abs() < f32::EPSILON);
    assert!((srgba.alpha - 1.0).abs() < f32::EPSILON);
}

// --- Spec Behavior 62: FadeOut satisfies Transition + OutTransition ---

#[test]
fn fade_out_satisfies_transition_and_out_transition() {
    use crate::transition::traits::OutTransition;
    let _effect: Box<dyn OutTransition> = Box::new(FadeOut {
        duration: 0.5,
        color: Color::BLACK,
    });
}

// --- Spec Behavior 51: FadeOut insert_starting ---

#[test]
fn fade_out_insert_starting_inserts_marker_and_config() {
    use crate::transition::traits::Transition;
    let mut world = World::new();
    let effect = FadeOut {
        duration: 0.5,
        color: Color::BLACK,
    };
    effect.insert_starting(&mut world);

    assert!(
        world.contains_resource::<StartingTransition<FadeOut>>(),
        "StartingTransition<FadeOut> should be inserted"
    );
    assert!(
        world.contains_resource::<FadeOutConfig>(),
        "FadeOutConfig should be inserted by insert_starting"
    );
}
