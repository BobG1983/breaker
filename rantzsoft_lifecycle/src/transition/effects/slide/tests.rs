use bevy::prelude::*;

use super::effect::*;
use crate::transition::{
    effects::{
        post_process::{EffectType, TransitionEffect},
        shared::TransitionProgress,
    },
    messages::{TransitionOver, TransitionReady, TransitionRunComplete},
    resources::{EndingTransition, RunningTransition, StartingTransition},
};

fn effect_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<TransitionReady>();
    app.add_message::<TransitionRunComplete>();
    app.add_message::<TransitionOver>();
    app.world_mut().spawn(Camera2d);
    app
}

// =======================================================================
// Section 12: Slide (post-process)
// =======================================================================

// --- Spec Behavior 44: Slide start inserts TransitionEffect with SLIDE ---

#[test]
fn slide_start_inserts_transition_effect_on_camera_with_left_direction() {
    let mut app = effect_test_app();
    app.insert_resource(SlideConfig {
        duration: 0.4,
        direction: SlideDirection::Left,
    });
    app.insert_resource(StartingTransition::<Slide>::new());
    app.add_systems(Update, slide_start);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(effects.len(), 1);
    let effect = effects[0];
    assert_eq!(effect.effect_type, EffectType::SLIDE);
    assert!(
        effect.progress.abs() < f32::EPSILON,
        "progress should be 0.0"
    );
    assert_eq!(
        effect.direction,
        Vec4::new(-1.0, 0.0, 0.0, 0.0),
        "Left direction should encode as (-1, 0, 0, 0)"
    );

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionReady>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
    assert!(app.world().contains_resource::<TransitionProgress>());
    let progress = app.world().resource::<TransitionProgress>();
    assert!((progress.duration - 0.4).abs() < f32::EPSILON);
    assert!(progress.elapsed.abs() < f32::EPSILON);
}

#[test]
fn slide_start_does_not_insert_slide_start_end_resource() {
    let mut app = effect_test_app();
    app.insert_resource(SlideConfig {
        duration: 0.4,
        direction: SlideDirection::Left,
    });
    app.insert_resource(StartingTransition::<Slide>::new());
    app.add_systems(Update, slide_start);
    app.update();

    assert!(
        !app.world()
            .contains_resource::<crate::transition::effects::shared::SlideStartEnd>(),
        "SlideStartEnd should NOT be inserted (removed from slide implementation)"
    );
}

// --- Spec Behavior 45: Slide direction encoding (UV Y inverted) ---

#[test]
fn slide_start_encodes_right_as_positive_x() {
    let mut app = effect_test_app();
    app.insert_resource(SlideConfig {
        duration: 0.4,
        direction: SlideDirection::Right,
    });
    app.insert_resource(StartingTransition::<Slide>::new());
    app.add_systems(Update, slide_start);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(
        effects[0].direction,
        Vec4::new(1.0, 0.0, 0.0, 0.0),
        "Right should be (1, 0, 0, 0)"
    );
}

#[test]
fn slide_start_encodes_up_as_negative_y_uv_inverted() {
    let mut app = effect_test_app();
    app.insert_resource(SlideConfig {
        duration: 0.4,
        direction: SlideDirection::Up,
    });
    app.insert_resource(StartingTransition::<Slide>::new());
    app.add_systems(Update, slide_start);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(
        effects[0].direction,
        Vec4::new(0.0, -1.0, 0.0, 0.0),
        "Up should be (0, -1, 0, 0) in UV space"
    );
}

#[test]
fn slide_start_encodes_down_as_positive_y_uv_inverted() {
    let mut app = effect_test_app();
    app.insert_resource(SlideConfig {
        duration: 0.4,
        direction: SlideDirection::Down,
    });
    app.insert_resource(StartingTransition::<Slide>::new());
    app.add_systems(Update, slide_start);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(
        effects[0].direction,
        Vec4::new(0.0, 1.0, 0.0, 0.0),
        "Down should be (0, 1, 0, 0) in UV space"
    );
}

// --- Spec Behavior 46: Slide run updates progress ---

#[test]
fn slide_run_updates_progress_on_camera() {
    let mut app = effect_test_app();
    let camera_entity = app
        .world_mut()
        .query_filtered::<Entity, With<Camera2d>>()
        .iter(app.world())
        .next()
        .unwrap();
    app.world_mut()
        .entity_mut(camera_entity)
        .insert(TransitionEffect {
            color: Vec4::ZERO,
            direction: Vec4::new(-1.0, 0.0, 0.0, 0.0),
            effect_type: EffectType::SLIDE,
            progress: 0.0,
        });
    app.insert_resource(RunningTransition::<Slide>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.25,
        duration: 1.0,
        completed: false,
    });
    app.add_systems(Update, slide_run);
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

#[test]
fn slide_run_sends_complete_at_full_progress() {
    let mut app = effect_test_app();
    let camera_entity = app
        .world_mut()
        .query_filtered::<Entity, With<Camera2d>>()
        .iter(app.world())
        .next()
        .unwrap();
    app.world_mut()
        .entity_mut(camera_entity)
        .insert(TransitionEffect {
            color: Vec4::ZERO,
            direction: Vec4::new(-1.0, 0.0, 0.0, 0.0),
            effect_type: EffectType::SLIDE,
            progress: 0.0,
        });
    app.insert_resource(RunningTransition::<Slide>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: false,
    });
    app.add_systems(Update, slide_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

#[test]
fn slide_run_does_not_double_send_when_already_completed() {
    let mut app = effect_test_app();
    let camera_entity = app
        .world_mut()
        .query_filtered::<Entity, With<Camera2d>>()
        .iter(app.world())
        .next()
        .unwrap();
    app.world_mut()
        .entity_mut(camera_entity)
        .insert(TransitionEffect::default());
    app.insert_resource(RunningTransition::<Slide>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: true,
    });
    app.add_systems(Update, slide_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 0);
}

// --- Spec Behavior 47: Slide run does NOT move camera position ---

#[test]
fn slide_run_does_not_move_camera_position() {
    let mut app = effect_test_app();
    // Move camera to an offset position
    let camera_entity = app
        .world_mut()
        .query_filtered::<Entity, With<Camera2d>>()
        .iter(app.world())
        .next()
        .unwrap();
    app.world_mut()
        .entity_mut(camera_entity)
        .insert(Transform::from_xyz(100.0, 50.0, 0.0));
    app.world_mut()
        .entity_mut(camera_entity)
        .insert(TransitionEffect {
            color: Vec4::ZERO,
            direction: Vec4::new(-1.0, 0.0, 0.0, 0.0),
            effect_type: EffectType::SLIDE,
            progress: 0.0,
        });
    app.insert_resource(RunningTransition::<Slide>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 1.0,
        completed: false,
    });
    app.add_systems(Update, slide_run);
    app.update();

    let transforms: Vec<&Transform> = app
        .world_mut()
        .query_filtered::<&Transform, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert!(
        (transforms[0].translation.x - 100.0).abs() < f32::EPSILON,
        "camera x should be unchanged at 100.0, got {}",
        transforms[0].translation.x
    );
    assert!(
        (transforms[0].translation.y - 50.0).abs() < f32::EPSILON,
        "camera y should be unchanged at 50.0, got {}",
        transforms[0].translation.y
    );
}

// --- Spec Behavior 48: Slide end ---

#[test]
fn slide_end_removes_transition_effect_and_sends_transition_over() {
    let mut app = effect_test_app();
    let camera_entity = app
        .world_mut()
        .query_filtered::<Entity, With<Camera2d>>()
        .iter(app.world())
        .next()
        .unwrap();
    app.world_mut()
        .entity_mut(camera_entity)
        .insert(TransitionEffect::default());
    app.insert_resource(EndingTransition::<Slide>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.4,
        duration: 0.4,
        completed: true,
    });
    app.add_systems(Update, slide_end);
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
fn slide_end_does_not_remove_slide_start_end_resource() {
    let mut app = effect_test_app();
    // SlideStartEnd should never be present in the new implementation
    app.insert_resource(EndingTransition::<Slide>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.4,
        duration: 0.4,
        completed: true,
    });
    app.add_systems(Update, slide_end);
    app.update();

    assert!(
        !app.world()
            .contains_resource::<crate::transition::effects::shared::SlideStartEnd>(),
        "SlideStartEnd should never be present"
    );
}

// --- Spec Behavior 72: Slide trait satisfaction ---

#[test]
fn slide_satisfies_transition_and_oneshot_transition() {
    use crate::transition::traits::OneShotTransition;
    let _effect: Box<dyn OneShotTransition> = Box::new(Slide {
        duration: 0.4,
        direction: SlideDirection::Left,
    });
}

// --- Spec Behaviors 83-84: Slide defaults ---

#[test]
fn slide_default_duration_is_0_3() {
    let effect = Slide::default();
    assert!((effect.duration - 0.3).abs() < f32::EPSILON);
}

#[test]
fn slide_default_direction_is_left() {
    let effect = Slide::default();
    assert_eq!(effect.direction, SlideDirection::Left);
}

#[test]
fn slide_direction_default_is_left() {
    assert_eq!(SlideDirection::default(), SlideDirection::Left);
}

// --- Spec Behavior 61: Slide insert_starting ---

#[test]
fn slide_insert_starting_inserts_marker_and_config() {
    use crate::transition::traits::Transition;
    let mut world = World::new();
    let effect = Slide {
        duration: 0.4,
        direction: SlideDirection::Right,
    };
    effect.insert_starting(&mut world);

    assert!(world.contains_resource::<StartingTransition<Slide>>());
    assert!(world.contains_resource::<SlideConfig>());
    let config = world.resource::<SlideConfig>();
    assert!((config.duration - 0.4).abs() < f32::EPSILON);
    assert_eq!(config.direction, SlideDirection::Right);
}
