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
// Section 6: PixelateOut (post-process)
// =======================================================================

// --- Spec Behavior 25: PixelateOut start ---

#[test]
fn pixelate_out_start_inserts_transition_effect_on_camera() {
    let (mut app, _camera) = effect_test_app();
    app.insert_resource(PixelateOutConfig {
        duration: 0.6,
        color:    Color::BLACK,
    });
    app.insert_resource(StartingTransition::<PixelateOut>::new());
    app.add_systems(Update, pixelate_out_start);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(effects.len(), 1);
    let effect = effects[0];
    assert_eq!(effect.effect_type, EffectType::PIXELATE);
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
    assert!((progress.duration - 0.6).abs() < f32::EPSILON);
}

// --- Spec Behavior 26: PixelateOut run ---

#[test]
fn pixelate_out_run_increases_progress_on_camera() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color:       Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction:   Vec4::ZERO,
        effect_type: EffectType::PIXELATE,
        progress:    0.0,
    });
    app.insert_resource(RunningTransition::<PixelateOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   0.25,
        duration:  1.0,
        completed: false,
    });
    app.add_systems(Update, pixelate_out_run);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert!(
        effects[0].progress > 0.0,
        "progress should have increased from 0.0, got {}",
        effects[0].progress
    );
}

#[test]
fn pixelate_out_run_sends_complete_at_full_progress() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color:       Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction:   Vec4::ZERO,
        effect_type: EffectType::PIXELATE,
        progress:    0.0,
    });
    app.insert_resource(RunningTransition::<PixelateOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   1.0,
        duration:  1.0,
        completed: false,
    });
    app.add_systems(Update, pixelate_out_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

#[test]
fn pixelate_out_run_does_not_double_send_when_already_completed() {
    let (mut app, camera) = effect_test_app();
    app.world_mut()
        .entity_mut(camera)
        .insert(TransitionEffect::default());
    app.insert_resource(RunningTransition::<PixelateOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   1.0,
        duration:  1.0,
        completed: true,
    });
    app.add_systems(Update, pixelate_out_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 0);
}

// --- Spec Behavior 27: PixelateOut end ---

#[test]
fn pixelate_out_end_removes_transition_effect_and_sends_transition_over() {
    let (mut app, camera) = effect_test_app();
    app.world_mut()
        .entity_mut(camera)
        .insert(TransitionEffect::default());
    app.insert_resource(EndingTransition::<PixelateOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   0.6,
        duration:  0.6,
        completed: true,
    });
    app.add_systems(Update, pixelate_out_end);
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
// Section 7: PixelateIn (post-process)
// =======================================================================

// --- Spec Behavior 28: PixelateIn start ---

#[test]
fn pixelate_in_start_inserts_transition_effect_on_camera_at_full_progress() {
    let (mut app, _camera) = effect_test_app();
    app.insert_resource(PixelateInConfig {
        duration: 0.6,
        color:    Color::BLACK,
    });
    app.insert_resource(StartingTransition::<PixelateIn>::new());
    app.add_systems(Update, pixelate_in_start);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].effect_type, EffectType::PIXELATE);
    assert!(
        (effects[0].progress - 1.0).abs() < f32::EPSILON,
        "PixelateIn should start at progress 1.0"
    );

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionReady>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
    assert!(app.world().contains_resource::<TransitionProgress>());
}

// --- Spec Behavior 29: PixelateIn run ---

#[test]
fn pixelate_in_run_decreases_progress_on_camera() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color:       Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction:   Vec4::ZERO,
        effect_type: EffectType::PIXELATE,
        progress:    1.0,
    });
    app.insert_resource(RunningTransition::<PixelateIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   0.5,
        duration:  1.0,
        completed: false,
    });
    app.add_systems(Update, pixelate_in_run);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert!(
        effects[0].progress < 1.0,
        "progress should have decreased from 1.0, got {}",
        effects[0].progress
    );
    assert!(
        effects[0].progress >= 0.0,
        "progress should not go negative"
    );
}

#[test]
fn pixelate_in_run_sends_complete_at_full_progress() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color:       Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction:   Vec4::ZERO,
        effect_type: EffectType::PIXELATE,
        progress:    1.0,
    });
    app.insert_resource(RunningTransition::<PixelateIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   1.0,
        duration:  1.0,
        completed: false,
    });
    app.add_systems(Update, pixelate_in_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

#[test]
fn pixelate_in_run_does_not_double_send_when_already_completed() {
    let (mut app, camera) = effect_test_app();
    app.world_mut()
        .entity_mut(camera)
        .insert(TransitionEffect::default());
    app.insert_resource(RunningTransition::<PixelateIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   1.0,
        duration:  1.0,
        completed: true,
    });
    app.add_systems(Update, pixelate_in_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 0);
}

// --- Spec Behavior 30: PixelateIn end ---

#[test]
fn pixelate_in_end_removes_transition_effect_and_sends_transition_over() {
    let (mut app, camera) = effect_test_app();
    app.world_mut()
        .entity_mut(camera)
        .insert(TransitionEffect::default());
    app.insert_resource(EndingTransition::<PixelateIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   0.6,
        duration:  0.6,
        completed: true,
    });
    app.add_systems(Update, pixelate_in_end);
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

// --- Spec Behaviors 66-67: Pixelate trait satisfaction ---

#[test]
fn pixelate_out_satisfies_transition_and_out_transition() {
    use crate::transition::traits::OutTransition;
    let _effect: Box<dyn OutTransition> = Box::new(PixelateOut {
        duration: 0.6,
        color:    Color::BLACK,
    });
}

#[test]
fn pixelate_in_satisfies_transition_and_in_transition() {
    use crate::transition::traits::InTransition;
    let _effect: Box<dyn InTransition> = Box::new(PixelateIn {
        duration: 0.6,
        color:    Color::BLACK,
    });
}

// --- Spec Behaviors 77-78: Pixelate defaults ---

#[test]
fn pixelate_out_default_duration_is_0_3() {
    let effect = PixelateOut::default();
    assert!((effect.duration - 0.3).abs() < f32::EPSILON);
}

#[test]
fn pixelate_in_default_duration_is_0_3() {
    let effect = PixelateIn::default();
    assert!((effect.duration - 0.3).abs() < f32::EPSILON);
}

// --- Spec Behaviors 55-56: Pixelate insert_starting ---

#[test]
fn pixelate_out_insert_starting_inserts_marker_and_config() {
    use crate::transition::traits::Transition;
    let mut world = World::new();
    let effect = PixelateOut {
        duration: 0.6,
        color:    Color::BLACK,
    };
    effect.insert_starting(&mut world);

    assert!(world.contains_resource::<StartingTransition<PixelateOut>>());
    assert!(world.contains_resource::<PixelateOutConfig>());
}

#[test]
fn pixelate_in_insert_starting_inserts_marker_and_config() {
    use crate::transition::traits::Transition;
    let mut world = World::new();
    let effect = PixelateIn {
        duration: 0.6,
        color:    Color::BLACK,
    };
    effect.insert_starting(&mut world);

    assert!(world.contains_resource::<StartingTransition<PixelateIn>>());
    assert!(world.contains_resource::<PixelateInConfig>());
}
