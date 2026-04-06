use bevy::prelude::*;

use super::{
    super::shared::{ScreenSize, TransitionOverlay, TransitionProgress},
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
// Section 7: IrisOut
// =======================================================================

// --- Behavior 33: IrisOut implements Transition and OutTransition ---

#[test]
fn iris_out_satisfies_transition_and_out_transition() {
    use crate::transition::traits::OutTransition;
    let _effect: Box<dyn OutTransition> = Box::new(IrisOut {
        duration: 0.5,
        color: Color::BLACK,
    });
}

// --- Behavior 34: IrisOut start spawns overlay at zero scale ---

#[test]
fn iris_out_start_spawns_overlay_at_zero_scale() {
    let mut app = effect_test_app();
    app.insert_resource(IrisOutConfig {
        duration: 0.5,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<IrisOut>::new());
    app.add_systems(Update, iris_out_start);
    app.update();

    let overlay_count = app
        .world_mut()
        .query_filtered::<Entity, With<TransitionOverlay>>()
        .iter(app.world())
        .count();
    assert_eq!(overlay_count, 1);

    let transforms: Vec<&Transform> = app
        .world_mut()
        .query_filtered::<&Transform, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    assert_eq!(
        transforms[0].scale,
        Vec3::ZERO,
        "IrisOut should start at zero scale"
    );
}

#[test]
fn iris_out_start_overlay_has_correct_size_and_z_index() {
    let mut app = effect_test_app();
    app.insert_resource(ScreenSize(Vec2::new(1920.0, 1080.0)));
    app.insert_resource(IrisOutConfig {
        duration: 0.5,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<IrisOut>::new());
    app.add_systems(Update, iris_out_start);
    app.update();

    let sprites: Vec<&Sprite> = app
        .world_mut()
        .query_filtered::<&Sprite, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    let size = sprites[0].custom_size.unwrap_or_default();
    assert!((size.x - 1920.0).abs() < f32::EPSILON);
    assert!((size.y - 1080.0).abs() < f32::EPSILON);

    let z_indices: Vec<&GlobalZIndex> = app
        .world_mut()
        .query_filtered::<&GlobalZIndex, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    assert_eq!(z_indices[0].0, i32::MAX - 1);
}

#[test]
fn iris_out_start_sends_transition_ready_and_progress() {
    let mut app = effect_test_app();
    app.insert_resource(IrisOutConfig {
        duration: 0.5,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<IrisOut>::new());
    app.add_systems(Update, iris_out_start);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionReady>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
    assert!(app.world().contains_resource::<TransitionProgress>());
    let progress = app.world().resource::<TransitionProgress>();
    assert!((progress.duration - 0.5).abs() < f32::EPSILON);
}

// --- Behavior 35: IrisOut run grows overlay scale ---

#[test]
fn iris_out_run_grows_scale_based_on_progress() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<IrisOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.25,
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
        Transform::from_scale(Vec3::ZERO),
    ));
    app.add_systems(Update, iris_out_run);
    app.update();

    let transforms: Vec<&Transform> = app
        .world_mut()
        .query_filtered::<&Transform, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    let scale = transforms[0].scale;
    assert!(
        (scale.x - 0.25).abs() < 0.01,
        "scale x should be ~0.25 at 25% progress, got {}",
        scale.x
    );
    assert!((scale.y - 0.25).abs() < 0.01, "scale y should be ~0.25");
}

#[test]
fn iris_out_run_sends_complete_at_full_progress() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<IrisOut>::new());
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
        Transform::from_scale(Vec3::ZERO),
    ));
    app.add_systems(Update, iris_out_run);
    app.update();

    let transforms: Vec<&Transform> = app
        .world_mut()
        .query_filtered::<&Transform, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    assert_eq!(
        transforms[0].scale,
        Vec3::ONE,
        "scale should be 1.0 at full progress"
    );

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

#[test]
fn iris_out_run_does_not_double_send_when_already_completed() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<IrisOut>::new());
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
        Transform::from_scale(Vec3::ONE),
    ));
    app.add_systems(Update, iris_out_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 0);
}

// --- Behavior 36: IrisOut end ---

#[test]
fn iris_out_end_despawns_overlay_and_sends_transition_over() {
    let mut app = effect_test_app();
    app.insert_resource(EndingTransition::<IrisOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.5,
        completed: true,
    });
    app.world_mut()
        .spawn((Sprite::default(), TransitionOverlay));
    app.add_systems(Update, iris_out_end);
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
// Section 8: IrisIn
// =======================================================================

// --- Behavior 37: IrisIn implements Transition and InTransition ---

#[test]
fn iris_in_satisfies_transition_and_in_transition() {
    use crate::transition::traits::InTransition;
    let _effect: Box<dyn InTransition> = Box::new(IrisIn {
        duration: 0.5,
        color: Color::BLACK,
    });
}

// --- Behavior 38: IrisIn start spawns overlay at full scale ---

#[test]
fn iris_in_start_spawns_overlay_at_full_scale() {
    let mut app = effect_test_app();
    app.insert_resource(IrisInConfig {
        duration: 0.5,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<IrisIn>::new());
    app.add_systems(Update, iris_in_start);
    app.update();

    let overlay_count = app
        .world_mut()
        .query_filtered::<Entity, With<TransitionOverlay>>()
        .iter(app.world())
        .count();
    assert_eq!(overlay_count, 1);

    let transforms: Vec<&Transform> = app
        .world_mut()
        .query_filtered::<&Transform, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    assert_eq!(
        transforms[0].scale,
        Vec3::ONE,
        "IrisIn should start at full scale"
    );
}

#[test]
fn iris_in_start_sends_transition_ready_and_progress() {
    let mut app = effect_test_app();
    app.insert_resource(IrisInConfig {
        duration: 0.5,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<IrisIn>::new());
    app.add_systems(Update, iris_in_start);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionReady>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
    assert!(app.world().contains_resource::<TransitionProgress>());
}

// --- Behavior 39: IrisIn run shrinks overlay scale ---

#[test]
fn iris_in_run_shrinks_scale_based_on_progress() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<IrisIn>::new());
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
        Transform::from_scale(Vec3::ONE),
    ));
    app.add_systems(Update, iris_in_run);
    app.update();

    let transforms: Vec<&Transform> = app
        .world_mut()
        .query_filtered::<&Transform, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    let scale = transforms[0].scale;
    assert!(
        (scale.x - 0.5).abs() < 0.01,
        "scale should be ~0.5 (1.0 - 0.5) at 50% progress, got {}",
        scale.x
    );
}

#[test]
fn iris_in_run_sends_complete_at_full_progress() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<IrisIn>::new());
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
        Transform::from_scale(Vec3::ONE),
    ));
    app.add_systems(Update, iris_in_run);
    app.update();

    let transforms: Vec<&Transform> = app
        .world_mut()
        .query_filtered::<&Transform, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    assert_eq!(
        transforms[0].scale,
        Vec3::ZERO,
        "scale should be zero at full progress"
    );

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

#[test]
fn iris_in_run_does_not_double_send_when_already_completed() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<IrisIn>::new());
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
        Transform::from_scale(Vec3::ZERO),
    ));
    app.add_systems(Update, iris_in_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 0);
}

// --- Behavior 40: IrisIn end ---

#[test]
fn iris_in_end_despawns_overlay_and_sends_transition_over() {
    let mut app = effect_test_app();
    app.insert_resource(EndingTransition::<IrisIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.5,
        completed: true,
    });
    app.world_mut()
        .spawn((Sprite::default(), TransitionOverlay));
    app.add_systems(Update, iris_in_end);
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
// Section 13: insert_starting overrides (behaviors 63-64)
// =======================================================================

#[test]
fn iris_out_insert_starting_inserts_marker_and_config() {
    use crate::transition::traits::Transition;
    let mut world = World::new();
    let effect = IrisOut {
        duration: 0.5,
        color: Color::BLACK,
    };
    effect.insert_starting(&mut world);

    assert!(world.contains_resource::<StartingTransition<IrisOut>>());
    assert!(
        world.contains_resource::<IrisOutConfig>(),
        "IrisOutConfig should be inserted by insert_starting"
    );
}

#[test]
fn iris_in_insert_starting_inserts_marker_and_config() {
    use crate::transition::traits::Transition;
    let mut world = World::new();
    let effect = IrisIn {
        duration: 0.5,
        color: Color::BLACK,
    };
    effect.insert_starting(&mut world);

    assert!(world.contains_resource::<StartingTransition<IrisIn>>());
    assert!(
        world.contains_resource::<IrisInConfig>(),
        "IrisInConfig should be inserted by insert_starting"
    );
}
