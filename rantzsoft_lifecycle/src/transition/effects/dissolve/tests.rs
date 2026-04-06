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
// Section 4: DissolveOut (post-process)
// =======================================================================

// --- Spec Behavior 19: DissolveOut start inserts TransitionEffect on camera ---

#[test]
fn dissolve_out_start_inserts_transition_effect_on_camera() {
    let mut app = effect_test_app();
    app.insert_resource(DissolveOutConfig {
        duration: 0.8,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<DissolveOut>::new());
    app.add_systems(Update, dissolve_out_start);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(effects.len(), 1);
    let effect = effects[0];
    assert_eq!(effect.effect_type, EffectType::DISSOLVE);
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
    assert!((progress.duration - 0.8).abs() < f32::EPSILON);
}

// --- Spec Behavior 20: DissolveOut run updates progress ---

#[test]
fn dissolve_out_run_increases_progress_on_camera() {
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
            color: Vec4::new(0.0, 0.0, 0.0, 1.0),
            direction: Vec4::ZERO,
            effect_type: EffectType::DISSOLVE,
            progress: 0.0,
        });
    app.insert_resource(RunningTransition::<DissolveOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 1.0,
        completed: false,
    });
    app.add_systems(Update, dissolve_out_run);
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
fn dissolve_out_run_sends_complete_at_full_progress() {
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
            color: Vec4::new(0.0, 0.0, 0.0, 1.0),
            direction: Vec4::ZERO,
            effect_type: EffectType::DISSOLVE,
            progress: 0.0,
        });
    app.insert_resource(RunningTransition::<DissolveOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: false,
    });
    app.add_systems(Update, dissolve_out_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

#[test]
fn dissolve_out_run_does_not_double_send_when_already_completed() {
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
            color: Vec4::new(0.0, 0.0, 0.0, 1.0),
            direction: Vec4::ZERO,
            effect_type: EffectType::DISSOLVE,
            progress: 1.0,
        });
    app.insert_resource(RunningTransition::<DissolveOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: true,
    });
    app.add_systems(Update, dissolve_out_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 0);
}

// --- Spec Behavior 21: DissolveOut end ---

#[test]
fn dissolve_out_end_removes_transition_effect_and_sends_transition_over() {
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
    app.insert_resource(EndingTransition::<DissolveOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.8,
        duration: 0.8,
        completed: true,
    });
    app.add_systems(Update, dissolve_out_end);
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
// Section 5: DissolveIn (post-process)
// =======================================================================

// --- Spec Behavior 22: DissolveIn start ---

#[test]
fn dissolve_in_start_inserts_transition_effect_on_camera_at_full_progress() {
    let mut app = effect_test_app();
    app.insert_resource(DissolveInConfig {
        duration: 0.8,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<DissolveIn>::new());
    app.add_systems(Update, dissolve_in_start);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].effect_type, EffectType::DISSOLVE);
    assert!(
        (effects[0].progress - 1.0).abs() < f32::EPSILON,
        "DissolveIn should start at progress 1.0"
    );

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionReady>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
    assert!(app.world().contains_resource::<TransitionProgress>());
}

// --- Spec Behavior 23: DissolveIn run decreases progress ---

#[test]
fn dissolve_in_run_decreases_progress_on_camera() {
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
            color: Vec4::new(0.0, 0.0, 0.0, 1.0),
            direction: Vec4::ZERO,
            effect_type: EffectType::DISSOLVE,
            progress: 1.0,
        });
    app.insert_resource(RunningTransition::<DissolveIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 1.0,
        completed: false,
    });
    app.add_systems(Update, dissolve_in_run);
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
fn dissolve_in_run_sends_complete_at_full_progress() {
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
            color: Vec4::new(0.0, 0.0, 0.0, 1.0),
            direction: Vec4::ZERO,
            effect_type: EffectType::DISSOLVE,
            progress: 1.0,
        });
    app.insert_resource(RunningTransition::<DissolveIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: false,
    });
    app.add_systems(Update, dissolve_in_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

#[test]
fn dissolve_in_run_does_not_double_send_when_already_completed() {
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
    app.insert_resource(RunningTransition::<DissolveIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: true,
    });
    app.add_systems(Update, dissolve_in_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 0);
}

// --- Spec Behavior 24: DissolveIn end ---

#[test]
fn dissolve_in_end_removes_transition_effect_and_sends_transition_over() {
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
    app.insert_resource(EndingTransition::<DissolveIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.8,
        duration: 0.8,
        completed: true,
    });
    app.add_systems(Update, dissolve_in_end);
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

// --- Spec Behaviors 64-65: Dissolve trait satisfaction ---

#[test]
fn dissolve_out_satisfies_transition_and_out_transition() {
    use crate::transition::traits::OutTransition;
    let _effect: Box<dyn OutTransition> = Box::new(DissolveOut {
        duration: 0.8,
        color: Color::BLACK,
    });
}

#[test]
fn dissolve_in_satisfies_transition_and_in_transition() {
    use crate::transition::traits::InTransition;
    let _effect: Box<dyn InTransition> = Box::new(DissolveIn {
        duration: 0.8,
        color: Color::BLACK,
    });
}

// --- Spec Behaviors 75-76: Dissolve defaults ---

#[test]
fn dissolve_out_default_duration_is_0_3() {
    let effect = DissolveOut::default();
    assert!((effect.duration - 0.3).abs() < f32::EPSILON);
}

#[test]
fn dissolve_in_default_duration_is_0_3() {
    let effect = DissolveIn::default();
    assert!((effect.duration - 0.3).abs() < f32::EPSILON);
}

// --- Spec Behaviors 53-54: Dissolve insert_starting ---

#[test]
fn dissolve_out_insert_starting_inserts_marker_and_config() {
    use crate::transition::traits::Transition;
    let mut world = World::new();
    let effect = DissolveOut {
        duration: 0.8,
        color: Color::BLACK,
    };
    effect.insert_starting(&mut world);

    assert!(world.contains_resource::<StartingTransition<DissolveOut>>());
    assert!(world.contains_resource::<DissolveOutConfig>());
}

#[test]
fn dissolve_in_insert_starting_inserts_marker_and_config() {
    use crate::transition::traits::Transition;
    let mut world = World::new();
    let effect = DissolveIn {
        duration: 0.8,
        color: Color::BLACK,
    };
    effect.insert_starting(&mut world);

    assert!(world.contains_resource::<StartingTransition<DissolveIn>>());
    assert!(world.contains_resource::<DissolveInConfig>());
}
