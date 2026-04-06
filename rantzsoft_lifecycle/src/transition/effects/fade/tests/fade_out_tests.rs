use bevy::prelude::*;

use super::{
    super::{
        super::shared::{ScreenSize, TransitionOverlay, TransitionProgress},
        effect::*,
    },
    helpers::effect_test_app,
};
use crate::transition::{
    messages::{TransitionOver, TransitionReady, TransitionRunComplete},
    resources::{EndingTransition, RunningTransition, StartingTransition},
};

// =======================================================================
// Section 1: FadeOut
// =======================================================================

// --- Behavior 1: FadeOut implements Transition and OutTransition ---

#[test]
fn fade_out_satisfies_transition_and_out_transition() {
    use crate::transition::traits::OutTransition;
    let _effect: Box<dyn OutTransition> = Box::new(FadeOut {
        duration: 0.5,
        color: Color::BLACK,
    });
    // Compiles = passes. Trait bounds are compile-time.
}

#[test]
fn fade_out_with_custom_color_satisfies_transition() {
    use crate::transition::traits::Transition;
    let _effect: Box<dyn Transition> = Box::new(FadeOut {
        duration: 0.0,
        color: Color::srgba(1.0, 0.0, 0.0, 1.0),
    });
}

// --- Behavior 2: FadeOut start system spawns overlay and sends TransitionReady ---

#[test]
fn fade_out_start_spawns_overlay_sprite_entity() {
    let mut app = effect_test_app();
    app.insert_resource(FadeOutConfig {
        duration: 0.5,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<FadeOut>::new());
    app.add_systems(Update, fade_out_start);
    app.update();

    let overlay_count = app
        .world_mut()
        .query_filtered::<Entity, With<TransitionOverlay>>()
        .iter(app.world())
        .count();
    assert_eq!(
        overlay_count, 1,
        "exactly 1 overlay entity should be spawned"
    );
}

#[test]
fn fade_out_start_overlay_has_sprite_with_zero_alpha() {
    let mut app = effect_test_app();
    app.insert_resource(FadeOutConfig {
        duration: 0.5,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<FadeOut>::new());
    app.add_systems(Update, fade_out_start);
    app.update();

    let sprites: Vec<&Sprite> = app
        .world_mut()
        .query_filtered::<&Sprite, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    assert_eq!(sprites.len(), 1, "overlay should have a Sprite component");
    let sprite = sprites[0];
    let alpha = sprite.color.alpha();
    assert!(
        alpha.abs() < f32::EPSILON,
        "FadeOut overlay alpha should start at 0.0, got {alpha}"
    );
}

#[test]
fn fade_out_start_overlay_has_correct_size() {
    let mut app = effect_test_app();
    app.insert_resource(ScreenSize(Vec2::new(1920.0, 1080.0)));
    app.insert_resource(FadeOutConfig {
        duration: 0.5,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<FadeOut>::new());
    app.add_systems(Update, fade_out_start);
    app.update();

    let sprites: Vec<&Sprite> = app
        .world_mut()
        .query_filtered::<&Sprite, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    let size = sprites[0].custom_size.unwrap_or_default();
    assert!(
        (size.x - 1920.0).abs() < f32::EPSILON,
        "sprite width should match ScreenSize"
    );
    assert!(
        (size.y - 1080.0).abs() < f32::EPSILON,
        "sprite height should match ScreenSize"
    );
}

#[test]
fn fade_out_start_overlay_has_global_z_index() {
    let mut app = effect_test_app();
    app.insert_resource(FadeOutConfig {
        duration: 0.5,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<FadeOut>::new());
    app.add_systems(Update, fade_out_start);
    app.update();

    let z_indices: Vec<&GlobalZIndex> = app
        .world_mut()
        .query_filtered::<&GlobalZIndex, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    assert_eq!(z_indices.len(), 1);
    assert_eq!(
        z_indices[0].0,
        i32::MAX - 1,
        "overlay should be at GlobalZIndex(i32::MAX - 1)"
    );
}

#[test]
fn fade_out_start_sends_transition_ready() {
    let mut app = effect_test_app();
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
    let mut app = effect_test_app();
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
fn fade_out_start_with_red_color_preserves_rgb_channels() {
    let mut app = effect_test_app();
    app.insert_resource(FadeOutConfig {
        duration: 0.5,
        color: Color::srgba(1.0, 0.0, 0.0, 1.0),
    });
    app.insert_resource(StartingTransition::<FadeOut>::new());
    app.add_systems(Update, fade_out_start);
    app.update();

    let sprites: Vec<&Sprite> = app
        .world_mut()
        .query_filtered::<&Sprite, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    let color = sprites[0].color;
    let srgba = color.to_srgba();
    assert!(
        (srgba.red - 1.0).abs() < f32::EPSILON,
        "red channel should be 1.0"
    );
    assert!(
        srgba.green.abs() < f32::EPSILON,
        "green channel should be 0.0"
    );
    assert!(
        srgba.blue.abs() < f32::EPSILON,
        "blue channel should be 0.0"
    );
    assert!(
        srgba.alpha.abs() < f32::EPSILON,
        "alpha should start at 0.0 for FadeOut"
    );
}

// --- Behavior 3: FadeOut run system advances overlay alpha ---

#[test]
fn fade_out_run_sets_alpha_to_progress_fraction() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.25,
        duration: 1.0,
        completed: false,
    });
    // Spawn overlay entity
    app.world_mut().spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        TransitionOverlay,
    ));
    app.add_systems(Update, fade_out_run);
    app.update();

    let sprites: Vec<&Sprite> = app
        .world_mut()
        .query_filtered::<&Sprite, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    let alpha = sprites[0].color.alpha();
    assert!(
        (alpha - 0.25).abs() < 0.01,
        "alpha should be ~0.25 at 25% progress, got {alpha}"
    );
}

#[test]
fn fade_out_run_does_not_send_complete_when_in_progress() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.25,
        duration: 1.0,
        completed: false,
    });
    app.world_mut().spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        TransitionOverlay,
    ));
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

// --- Behavior 4: FadeOut run sends TransitionRunComplete at completion ---

#[test]
fn fade_out_run_sends_complete_when_elapsed_equals_duration() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.5,
        completed: false,
    });
    app.world_mut().spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        TransitionOverlay,
    ));
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
}

#[test]
fn fade_out_run_clamps_alpha_to_one_when_overshooting() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.6,
        duration: 0.5,
        completed: false,
    });
    app.world_mut().spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        TransitionOverlay,
    ));
    app.add_systems(Update, fade_out_run);
    app.update();

    let sprites: Vec<&Sprite> = app
        .world_mut()
        .query_filtered::<&Sprite, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    let alpha = sprites[0].color.alpha();
    assert!(
        (alpha - 1.0).abs() < f32::EPSILON,
        "alpha should be clamped to 1.0, got {alpha}"
    );
}

#[test]
fn fade_out_run_sets_completed_flag_on_completion() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: false,
    });
    app.world_mut().spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        TransitionOverlay,
    ));
    app.add_systems(Update, fade_out_run);
    app.update();

    let progress = app.world().resource::<TransitionProgress>();
    assert!(
        progress.completed,
        "completed flag should be set to true on completion"
    );
}

#[test]
fn fade_out_run_does_not_double_send_complete_when_already_completed() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: true, // already completed
    });
    app.world_mut().spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 1.0),
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        TransitionOverlay,
    ));
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

// --- Behavior 5: FadeOut end despawns overlay and sends TransitionOver ---

#[test]
fn fade_out_end_despawns_overlay_entity() {
    let mut app = effect_test_app();
    app.insert_resource(EndingTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.5,
        completed: true,
    });
    app.world_mut().spawn((
        Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        TransitionOverlay,
    ));
    app.add_systems(Update, fade_out_end);
    app.update();

    let overlay_count = app
        .world_mut()
        .query_filtered::<Entity, With<TransitionOverlay>>()
        .iter(app.world())
        .count();
    assert_eq!(overlay_count, 0, "overlay entity should be despawned");
}

#[test]
fn fade_out_end_removes_transition_progress() {
    let mut app = effect_test_app();
    app.insert_resource(EndingTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.5,
        completed: true,
    });
    app.world_mut()
        .spawn((Sprite::default(), TransitionOverlay));
    app.add_systems(Update, fade_out_end);
    app.update();

    assert!(
        !app.world().contains_resource::<TransitionProgress>(),
        "TransitionProgress should be removed by end system"
    );
}

#[test]
fn fade_out_end_sends_transition_over() {
    let mut app = effect_test_app();
    app.insert_resource(EndingTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.5,
        completed: true,
    });
    app.world_mut()
        .spawn((Sprite::default(), TransitionOverlay));
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

#[test]
fn fade_out_end_does_not_panic_when_no_overlay_exists() {
    let mut app = effect_test_app();
    app.insert_resource(EndingTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.5,
        completed: true,
    });
    // No overlay entity spawned
    app.add_systems(Update, fade_out_end);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionOver>>();
    assert_eq!(
        msgs.iter_current_update_messages().count(),
        1,
        "TransitionOver should still be sent even without overlay entity"
    );
}

// --- Behavior 6: FadeOut default configuration ---

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

#[test]
fn fade_out_default_duration_is_positive() {
    let effect = FadeOut::default();
    assert!(effect.duration > 0.0, "default duration should be positive");
}

// =======================================================================
// Section 13: insert_starting overrides (behavior 57)
// =======================================================================

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
