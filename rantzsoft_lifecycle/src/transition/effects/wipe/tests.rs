use bevy::prelude::*;

use super::{
    super::shared::{ScreenSize, TransitionOverlay, TransitionProgress, WipeDirection},
    effect::*,
};
use crate::transition::{
    messages::{TransitionOver, TransitionReady, TransitionRunComplete},
    resources::{EndingTransition, RunningTransition, StartingTransition},
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
// Section 5: WipeOut
// =======================================================================

// --- Behavior 24: WipeOut implements Transition and OutTransition ---

#[test]
fn wipe_out_satisfies_transition_and_out_transition() {
    use crate::transition::traits::OutTransition;
    let _effect: Box<dyn OutTransition> = Box::new(WipeOut {
        duration: 0.5,
        color: Color::BLACK,
        direction: WipeDirection::Left,
    });
}

// --- Behavior 25: WipeOut start spawns off-screen overlay ---

#[test]
fn wipe_out_start_spawns_overlay_entity() {
    let mut app = effect_test_app();
    app.insert_resource(WipeOutConfig {
        duration: 0.5,
        color: Color::BLACK,
        direction: WipeDirection::Left,
    });
    app.insert_resource(StartingTransition::<WipeOut>::new());
    app.add_systems(Update, wipe_out_start);
    app.update();

    let overlay_count = app
        .world_mut()
        .query_filtered::<Entity, With<TransitionOverlay>>()
        .iter(app.world())
        .count();
    assert_eq!(overlay_count, 1, "exactly 1 overlay should be spawned");
}

#[test]
fn wipe_out_start_overlay_has_correct_components() {
    let mut app = effect_test_app();
    app.insert_resource(ScreenSize(Vec2::new(1920.0, 1080.0)));
    app.insert_resource(WipeOutConfig {
        duration: 0.5,
        color: Color::BLACK,
        direction: WipeDirection::Left,
    });
    app.insert_resource(StartingTransition::<WipeOut>::new());
    app.add_systems(Update, wipe_out_start);
    app.update();

    let z_indices: Vec<&GlobalZIndex> = app
        .world_mut()
        .query_filtered::<&GlobalZIndex, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    assert_eq!(z_indices.len(), 1);
    assert_eq!(z_indices[0].0, i32::MAX - 1);

    let sprites: Vec<&Sprite> = app
        .world_mut()
        .query_filtered::<&Sprite, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    let size = sprites[0].custom_size.unwrap_or_default();
    assert!((size.x - 1920.0).abs() < f32::EPSILON);
    assert!((size.y - 1080.0).abs() < f32::EPSILON);
}

#[test]
fn wipe_out_start_sends_transition_ready_and_progress() {
    let mut app = effect_test_app();
    app.insert_resource(WipeOutConfig {
        duration: 0.5,
        color: Color::BLACK,
        direction: WipeDirection::Left,
    });
    app.insert_resource(StartingTransition::<WipeOut>::new());
    app.add_systems(Update, wipe_out_start);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionReady>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
    assert!(app.world().contains_resource::<TransitionProgress>());
    let progress = app.world().resource::<TransitionProgress>();
    assert!((progress.duration - 0.5).abs() < f32::EPSILON);
}

// --- Behavior 26: WipeOut run slides sprite ---

#[test]
fn wipe_out_run_does_not_send_complete_when_in_progress() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<WipeOut>::new());
    app.insert_resource(WipeOutConfig {
        duration: 1.0,
        color: Color::BLACK,
        direction: WipeDirection::Left,
    });
    app.insert_resource(ScreenSize(Vec2::new(1920.0, 1080.0)));
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 1.0,
        completed: false,
    });
    app.world_mut().spawn((
        Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        TransitionOverlay,
        Transform::from_xyz(-1920.0, 0.0, 0.0),
    ));
    app.add_systems(Update, wipe_out_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 0);
}

#[test]
fn wipe_out_run_sends_complete_at_full_progress() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<WipeOut>::new());
    app.insert_resource(WipeOutConfig {
        duration: 1.0,
        color: Color::BLACK,
        direction: WipeDirection::Left,
    });
    app.insert_resource(ScreenSize(Vec2::new(1920.0, 1080.0)));
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: false,
    });
    app.world_mut().spawn((
        Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        TransitionOverlay,
        Transform::from_xyz(-1920.0, 0.0, 0.0),
    ));
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
    app.insert_resource(RunningTransition::<WipeOut>::new());
    app.insert_resource(WipeOutConfig {
        duration: 1.0,
        color: Color::BLACK,
        direction: WipeDirection::Left,
    });
    app.insert_resource(ScreenSize(Vec2::new(1920.0, 1080.0)));
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: true,
    });
    app.world_mut().spawn((
        Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        TransitionOverlay,
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    app.add_systems(Update, wipe_out_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 0);
}

// --- Behavior 27: WipeOut end ---

#[test]
fn wipe_out_end_despawns_overlay_and_sends_transition_over() {
    let mut app = effect_test_app();
    app.insert_resource(EndingTransition::<WipeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.5,
        completed: true,
    });
    app.world_mut()
        .spawn((Sprite::default(), TransitionOverlay));
    app.add_systems(Update, wipe_out_end);
    app.update();

    let overlay_count = app
        .world_mut()
        .query_filtered::<Entity, With<TransitionOverlay>>()
        .iter(app.world())
        .count();
    assert_eq!(overlay_count, 0);
    assert!(!app.world().contains_resource::<TransitionProgress>());

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionOver>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

// --- Behavior 28: WipeOut default ---

#[test]
fn wipe_out_default_duration_is_0_3() {
    let effect = WipeOut::default();
    assert!((effect.duration - 0.3).abs() < f32::EPSILON);
}

#[test]
fn wipe_out_default_color_is_black() {
    let effect = WipeOut::default();
    let srgba = effect.color.to_srgba();
    assert!(srgba.red.abs() < f32::EPSILON);
    assert!(srgba.green.abs() < f32::EPSILON);
    assert!(srgba.blue.abs() < f32::EPSILON);
}

#[test]
fn wipe_out_default_direction_is_left() {
    let effect = WipeOut::default();
    assert_eq!(effect.direction, WipeDirection::Left);
}

// =======================================================================
// Section 6: WipeIn
// =======================================================================

// --- Behavior 29: WipeIn implements Transition and InTransition ---

#[test]
fn wipe_in_satisfies_transition_and_in_transition() {
    use crate::transition::traits::InTransition;
    let _effect: Box<dyn InTransition> = Box::new(WipeIn {
        duration: 0.5,
        color: Color::BLACK,
        direction: WipeDirection::Left,
    });
}

// --- Behavior 30: WipeIn start spawns full-coverage overlay ---

#[test]
fn wipe_in_start_spawns_full_coverage_overlay() {
    let mut app = effect_test_app();
    app.insert_resource(ScreenSize(Vec2::new(1920.0, 1080.0)));
    app.insert_resource(WipeInConfig {
        duration: 0.5,
        color: Color::BLACK,
        direction: WipeDirection::Left,
    });
    app.insert_resource(StartingTransition::<WipeIn>::new());
    app.add_systems(Update, wipe_in_start);
    app.update();

    let overlay_count = app
        .world_mut()
        .query_filtered::<Entity, With<TransitionOverlay>>()
        .iter(app.world())
        .count();
    assert_eq!(overlay_count, 1);

    let sprites: Vec<&Sprite> = app
        .world_mut()
        .query_filtered::<&Sprite, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    let size = sprites[0].custom_size.unwrap_or_default();
    assert!((size.x - 1920.0).abs() < f32::EPSILON);
    assert!((size.y - 1080.0).abs() < f32::EPSILON);

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionReady>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
    assert!(app.world().contains_resource::<TransitionProgress>());
}

// --- Behavior 31: WipeIn run slides overlay off-screen ---

#[test]
fn wipe_in_run_sends_complete_at_full_progress() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<WipeIn>::new());
    app.insert_resource(WipeInConfig {
        duration: 1.0,
        color: Color::BLACK,
        direction: WipeDirection::Left,
    });
    app.insert_resource(ScreenSize(Vec2::new(1920.0, 1080.0)));
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: false,
    });
    app.world_mut().spawn((
        Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        TransitionOverlay,
        Transform::default(),
    ));
    app.add_systems(Update, wipe_in_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

#[test]
fn wipe_in_run_does_not_double_send_when_already_completed() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<WipeIn>::new());
    app.insert_resource(WipeInConfig {
        duration: 1.0,
        color: Color::BLACK,
        direction: WipeDirection::Left,
    });
    app.insert_resource(ScreenSize(Vec2::new(1920.0, 1080.0)));
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: true,
    });
    app.world_mut().spawn((
        Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        TransitionOverlay,
        Transform::default(),
    ));
    app.add_systems(Update, wipe_in_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 0);
}

// --- Behavior 32: WipeIn end ---

#[test]
fn wipe_in_end_despawns_overlay_and_sends_transition_over() {
    let mut app = effect_test_app();
    app.insert_resource(EndingTransition::<WipeIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.5,
        completed: true,
    });
    app.world_mut()
        .spawn((Sprite::default(), TransitionOverlay));
    app.add_systems(Update, wipe_in_end);
    app.update();

    let overlay_count = app
        .world_mut()
        .query_filtered::<Entity, With<TransitionOverlay>>()
        .iter(app.world())
        .count();
    assert_eq!(overlay_count, 0);
    assert!(!app.world().contains_resource::<TransitionProgress>());

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionOver>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

// =======================================================================
// Section 13: insert_starting overrides (behaviors 61-62)
// =======================================================================

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
    assert!(
        world.contains_resource::<WipeOutConfig>(),
        "WipeOutConfig should be inserted by insert_starting"
    );
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
    assert!(
        world.contains_resource::<WipeInConfig>(),
        "WipeInConfig should be inserted by insert_starting"
    );
}
