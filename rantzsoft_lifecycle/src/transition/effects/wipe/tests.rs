use bevy::prelude::*;

use super::effect::*;
use crate::transition::{
    effects::{
        post_process::{EffectType, TransitionEffect},
        shared::{TransitionProgress, WipeDirection},
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
// Section 10: WipeOut (post-process)
// =======================================================================

// --- Spec Behavior 37: WipeOut start inserts TransitionEffect with WIPE ---

#[test]
fn wipe_out_start_inserts_transition_effect_on_camera_with_left_direction() {
    let mut app = effect_test_app();
    app.insert_resource(WipeOutConfig {
        duration: 0.5,
        color: Color::BLACK,
        direction: WipeDirection::Left,
    });
    app.insert_resource(StartingTransition::<WipeOut>::new());
    app.add_systems(Update, wipe_out_start);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(effects.len(), 1);
    let effect = effects[0];
    assert_eq!(effect.effect_type, EffectType::WIPE);
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
    assert!((progress.duration - 0.5).abs() < f32::EPSILON);
}

#[test]
fn wipe_out_start_removes_config_resource() {
    let mut app = effect_test_app();
    app.insert_resource(WipeOutConfig {
        duration: 0.5,
        color: Color::BLACK,
        direction: WipeDirection::Left,
    });
    app.insert_resource(StartingTransition::<WipeOut>::new());
    app.add_systems(Update, wipe_out_start);
    app.update();

    assert!(
        !app.world().contains_resource::<WipeOutConfig>(),
        "WipeOutConfig should be removed by start system"
    );
}

// --- Spec Behavior 38: WipeOut direction encoding ---

#[test]
fn wipe_out_start_encodes_right_as_positive_x() {
    let mut app = effect_test_app();
    app.insert_resource(WipeOutConfig {
        duration: 0.5,
        color: Color::BLACK,
        direction: WipeDirection::Right,
    });
    app.insert_resource(StartingTransition::<WipeOut>::new());
    app.add_systems(Update, wipe_out_start);
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
fn wipe_out_start_encodes_up_as_positive_y() {
    let mut app = effect_test_app();
    app.insert_resource(WipeOutConfig {
        duration: 0.5,
        color: Color::BLACK,
        direction: WipeDirection::Up,
    });
    app.insert_resource(StartingTransition::<WipeOut>::new());
    app.add_systems(Update, wipe_out_start);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(
        effects[0].direction,
        Vec4::new(0.0, 1.0, 0.0, 0.0),
        "Up should be (0, 1, 0, 0)"
    );
}

#[test]
fn wipe_out_start_encodes_down_as_negative_y() {
    let mut app = effect_test_app();
    app.insert_resource(WipeOutConfig {
        duration: 0.5,
        color: Color::BLACK,
        direction: WipeDirection::Down,
    });
    app.insert_resource(StartingTransition::<WipeOut>::new());
    app.add_systems(Update, wipe_out_start);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(
        effects[0].direction,
        Vec4::new(0.0, -1.0, 0.0, 0.0),
        "Down should be (0, -1, 0, 0)"
    );
}

// --- Spec Behavior 39: WipeOut run updates progress ---

#[test]
fn wipe_out_run_updates_progress_on_camera() {
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
            direction: Vec4::new(-1.0, 0.0, 0.0, 0.0),
            effect_type: EffectType::WIPE,
            progress: 0.0,
        });
    // Note: config resource is NOT present in world (removed by start system)
    app.insert_resource(RunningTransition::<WipeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 1.0,
        completed: false,
    });
    app.add_systems(Update, wipe_out_run);
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
fn wipe_out_run_sends_complete_at_full_progress() {
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
            direction: Vec4::new(-1.0, 0.0, 0.0, 0.0),
            effect_type: EffectType::WIPE,
            progress: 0.0,
        });
    app.insert_resource(RunningTransition::<WipeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: false,
    });
    app.add_systems(Update, wipe_out_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

#[test]
fn wipe_out_run_does_not_double_send_when_already_completed() {
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
    app.insert_resource(RunningTransition::<WipeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: true,
    });
    app.add_systems(Update, wipe_out_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 0);
}

// --- Spec Behavior 40: WipeOut end ---

#[test]
fn wipe_out_end_removes_transition_effect_and_sends_transition_over() {
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
    app.insert_resource(EndingTransition::<WipeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.5,
        completed: true,
    });
    app.add_systems(Update, wipe_out_end);
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
// Section 11: WipeIn (post-process)
// =======================================================================

// --- Spec Behavior 41: WipeIn start ---

#[test]
fn wipe_in_start_inserts_transition_effect_on_camera_at_full_progress() {
    let mut app = effect_test_app();
    app.insert_resource(WipeInConfig {
        duration: 0.5,
        color: Color::BLACK,
        direction: WipeDirection::Left,
    });
    app.insert_resource(StartingTransition::<WipeIn>::new());
    app.add_systems(Update, wipe_in_start);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(effects.len(), 1);
    let effect = effects[0];
    assert_eq!(effect.effect_type, EffectType::WIPE);
    assert!(
        (effect.progress - 1.0).abs() < f32::EPSILON,
        "WipeIn should start at progress 1.0"
    );
    assert_eq!(
        effect.direction,
        Vec4::new(-1.0, 0.0, 0.0, 0.0),
        "Left direction"
    );

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionReady>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
    assert!(app.world().contains_resource::<TransitionProgress>());
}

#[test]
fn wipe_in_start_removes_config_resource() {
    let mut app = effect_test_app();
    app.insert_resource(WipeInConfig {
        duration: 0.5,
        color: Color::BLACK,
        direction: WipeDirection::Left,
    });
    app.insert_resource(StartingTransition::<WipeIn>::new());
    app.add_systems(Update, wipe_in_start);
    app.update();

    assert!(
        !app.world().contains_resource::<WipeInConfig>(),
        "WipeInConfig should be removed by start system"
    );
}

// --- Spec Behavior 42: WipeIn run ---

#[test]
fn wipe_in_run_decreases_progress_on_camera() {
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
            direction: Vec4::new(-1.0, 0.0, 0.0, 0.0),
            effect_type: EffectType::WIPE,
            progress: 1.0,
        });
    // Note: config resource NOT present (removed by start)
    app.insert_resource(RunningTransition::<WipeIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: false,
    });
    app.add_systems(Update, wipe_in_run);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert!(
        effects[0].progress.abs() < f32::EPSILON,
        "progress should be 0.0 at full completion, got {}",
        effects[0].progress
    );

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

#[test]
fn wipe_in_run_does_not_double_send_when_already_completed() {
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
    app.insert_resource(RunningTransition::<WipeIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: true,
    });
    app.add_systems(Update, wipe_in_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 0);
}

// --- Spec Behavior 43: WipeIn end ---

#[test]
fn wipe_in_end_removes_transition_effect_and_sends_transition_over() {
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
    app.insert_resource(EndingTransition::<WipeIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.5,
        completed: true,
    });
    app.add_systems(Update, wipe_in_end);
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

// --- Spec Behaviors 70-71: Wipe trait satisfaction ---

#[test]
fn wipe_out_satisfies_transition_and_out_transition() {
    use crate::transition::traits::OutTransition;
    let _effect: Box<dyn OutTransition> = Box::new(WipeOut {
        duration: 0.5,
        color: Color::BLACK,
        direction: WipeDirection::Left,
    });
}

#[test]
fn wipe_in_satisfies_transition_and_in_transition() {
    use crate::transition::traits::InTransition;
    let _effect: Box<dyn InTransition> = Box::new(WipeIn {
        duration: 0.5,
        color: Color::BLACK,
        direction: WipeDirection::Left,
    });
}

// --- Spec Behaviors 81-82: Wipe defaults ---

#[test]
fn wipe_out_default_duration_is_0_3() {
    let effect = WipeOut::default();
    assert!((effect.duration - 0.3).abs() < f32::EPSILON);
}

#[test]
fn wipe_out_default_direction_is_left() {
    let effect = WipeOut::default();
    assert_eq!(effect.direction, WipeDirection::Left);
}

#[test]
fn wipe_in_default_duration_is_0_3() {
    let effect = WipeIn::default();
    assert!((effect.duration - 0.3).abs() < f32::EPSILON);
}

#[test]
fn wipe_in_default_direction_is_left() {
    let effect = WipeIn::default();
    assert_eq!(effect.direction, WipeDirection::Left);
}

// --- Spec Behavior 85: WipeDirection default ---

#[test]
fn wipe_direction_default_is_left() {
    assert_eq!(WipeDirection::default(), WipeDirection::Left);
}

// --- Spec Behaviors 59-60: Wipe insert_starting ---

#[test]
fn wipe_out_insert_starting_inserts_marker_and_config() {
    use crate::transition::traits::Transition;
    let mut world = World::new();
    let effect = WipeOut {
        duration: 0.5,
        color: Color::BLACK,
        direction: WipeDirection::Left,
    };
    effect.insert_starting(&mut world);

    assert!(world.contains_resource::<StartingTransition<WipeOut>>());
    assert!(world.contains_resource::<WipeOutConfig>());
}

#[test]
fn wipe_in_insert_starting_inserts_marker_and_config() {
    use crate::transition::traits::Transition;
    let mut world = World::new();
    let effect = WipeIn {
        duration: 0.5,
        color: Color::BLACK,
        direction: WipeDirection::Right,
    };
    effect.insert_starting(&mut world);

    assert!(world.contains_resource::<StartingTransition<WipeIn>>());
    assert!(world.contains_resource::<WipeInConfig>());
}
