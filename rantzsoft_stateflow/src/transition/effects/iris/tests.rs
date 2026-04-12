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
// Section 8: IrisOut (post-process)
// =======================================================================

// --- Spec Behavior 31: IrisOut start ---

#[test]
fn iris_out_start_inserts_transition_effect_on_camera() {
    let (mut app, _camera) = effect_test_app();
    app.insert_resource(IrisOutConfig {
        duration: 0.5,
        color:    Color::BLACK,
    });
    app.insert_resource(StartingTransition::<IrisOut>::new());
    app.add_systems(Update, iris_out_start);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(effects.len(), 1);
    let effect = effects[0];
    assert_eq!(effect.effect_type, EffectType::IRIS);
    assert!(
        effect.progress.abs() < f32::EPSILON,
        "progress should be 0.0"
    );
    assert_eq!(effect.color, Vec4::new(0.0, 0.0, 0.0, 1.0));

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionReady>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
    assert!(app.world().contains_resource::<TransitionProgress>());
    let progress = app.world().resource::<TransitionProgress>();
    assert!((progress.duration - 0.5).abs() < f32::EPSILON);
}

// --- Spec Behavior 32: IrisOut run ---

#[test]
fn iris_out_run_increases_progress_on_camera() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color:       Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction:   Vec4::ZERO,
        effect_type: EffectType::IRIS,
        progress:    0.0,
    });
    app.insert_resource(RunningTransition::<IrisOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   0.25,
        duration:  1.0,
        completed: false,
    });
    app.add_systems(Update, iris_out_run);
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
fn iris_out_run_sends_complete_at_full_progress() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color:       Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction:   Vec4::ZERO,
        effect_type: EffectType::IRIS,
        progress:    0.0,
    });
    app.insert_resource(RunningTransition::<IrisOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   1.0,
        duration:  1.0,
        completed: false,
    });
    app.add_systems(Update, iris_out_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert!(
        (effects[0].progress - 1.0).abs() < f32::EPSILON,
        "progress should be 1.0 at completion"
    );
}

#[test]
fn iris_out_run_does_not_double_send_when_already_completed() {
    let (mut app, camera) = effect_test_app();
    app.world_mut()
        .entity_mut(camera)
        .insert(TransitionEffect::default());
    app.insert_resource(RunningTransition::<IrisOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   1.0,
        duration:  1.0,
        completed: true,
    });
    app.add_systems(Update, iris_out_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 0);
}

// --- Spec Behavior 33: IrisOut end ---

#[test]
fn iris_out_end_removes_transition_effect_and_sends_transition_over() {
    let (mut app, camera) = effect_test_app();
    app.world_mut()
        .entity_mut(camera)
        .insert(TransitionEffect::default());
    app.insert_resource(EndingTransition::<IrisOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   0.5,
        duration:  0.5,
        completed: true,
    });
    app.add_systems(Update, iris_out_end);
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

// =======================================================================
// Section 9: IrisIn (post-process)
// =======================================================================

// --- Spec Behavior 34: IrisIn start ---

#[test]
fn iris_in_start_inserts_transition_effect_on_camera_at_full_progress() {
    let (mut app, _camera) = effect_test_app();
    app.insert_resource(IrisInConfig {
        duration: 0.5,
        color:    Color::BLACK,
    });
    app.insert_resource(StartingTransition::<IrisIn>::new());
    app.add_systems(Update, iris_in_start);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].effect_type, EffectType::IRIS);
    assert!(
        (effects[0].progress - 1.0).abs() < f32::EPSILON,
        "IrisIn should start at progress 1.0"
    );

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionReady>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
    assert!(app.world().contains_resource::<TransitionProgress>());
}

// --- Spec Behavior 35: IrisIn run ---

#[test]
fn iris_in_run_decreases_progress_on_camera() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color:       Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction:   Vec4::ZERO,
        effect_type: EffectType::IRIS,
        progress:    1.0,
    });
    app.insert_resource(RunningTransition::<IrisIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   0.5,
        duration:  1.0,
        completed: false,
    });
    app.add_systems(Update, iris_in_run);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert!(
        (effects[0].progress - 0.5).abs() < 0.01,
        "progress should be ~0.5, got {}",
        effects[0].progress
    );
}

#[test]
fn iris_in_run_sends_complete_at_full_progress() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color:       Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction:   Vec4::ZERO,
        effect_type: EffectType::IRIS,
        progress:    1.0,
    });
    app.insert_resource(RunningTransition::<IrisIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   1.0,
        duration:  1.0,
        completed: false,
    });
    app.add_systems(Update, iris_in_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

#[test]
fn iris_in_run_does_not_double_send_when_already_completed() {
    let (mut app, camera) = effect_test_app();
    app.world_mut()
        .entity_mut(camera)
        .insert(TransitionEffect::default());
    app.insert_resource(RunningTransition::<IrisIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   1.0,
        duration:  1.0,
        completed: true,
    });
    app.add_systems(Update, iris_in_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 0);
}

// --- Spec Behavior 36: IrisIn end ---

#[test]
fn iris_in_end_removes_transition_effect_and_sends_transition_over() {
    let (mut app, camera) = effect_test_app();
    app.world_mut()
        .entity_mut(camera)
        .insert(TransitionEffect::default());
    app.insert_resource(EndingTransition::<IrisIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   0.5,
        duration:  0.5,
        completed: true,
    });
    app.add_systems(Update, iris_in_end);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(effects.len(), 0);
    assert!(!app.world().contains_resource::<TransitionProgress>());

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionOver>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

// --- Spec Behaviors 68-69: Iris trait satisfaction ---

#[test]
fn iris_out_satisfies_transition_and_out_transition() {
    use crate::transition::traits::OutTransition;
    let _effect: Box<dyn OutTransition> = Box::new(IrisOut {
        duration: 0.5,
        color:    Color::BLACK,
    });
}

#[test]
fn iris_in_satisfies_transition_and_in_transition() {
    use crate::transition::traits::InTransition;
    let _effect: Box<dyn InTransition> = Box::new(IrisIn {
        duration: 0.5,
        color:    Color::BLACK,
    });
}

// --- Spec Behaviors 79-80: Iris defaults ---

#[test]
fn iris_out_default_duration_is_0_3() {
    let effect = IrisOut::default();
    assert!((effect.duration - 0.3).abs() < f32::EPSILON);
}

#[test]
fn iris_in_default_duration_is_0_3() {
    let effect = IrisIn::default();
    assert!((effect.duration - 0.3).abs() < f32::EPSILON);
}

// --- Spec Behaviors 57-58: Iris insert_starting ---

#[test]
fn iris_out_insert_starting_inserts_marker_and_config() {
    use crate::transition::traits::Transition;
    let mut world = World::new();
    let effect = IrisOut {
        duration: 0.5,
        color:    Color::BLACK,
    };
    effect.insert_starting(&mut world);

    assert!(world.contains_resource::<StartingTransition<IrisOut>>());
    assert!(world.contains_resource::<IrisOutConfig>());
}

#[test]
fn iris_in_insert_starting_inserts_marker_and_config() {
    use crate::transition::traits::Transition;
    let mut world = World::new();
    let effect = IrisIn {
        duration: 0.5,
        color:    Color::BLACK,
    };
    effect.insert_starting(&mut world);

    assert!(world.contains_resource::<StartingTransition<IrisIn>>());
    assert!(world.contains_resource::<IrisInConfig>());
}
