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
// Section 2: FadeIn
// =======================================================================

// --- Behavior 7: FadeIn implements Transition and InTransition ---

#[test]
fn fade_in_satisfies_transition_and_in_transition() {
    use crate::transition::traits::InTransition;
    let _effect: Box<dyn InTransition> = Box::new(FadeIn {
        duration: 0.5,
        color: Color::BLACK,
    });
}

// --- Behavior 8: FadeIn start system spawns overlay at full opacity ---

#[test]
fn fade_in_start_spawns_overlay_at_full_opacity() {
    let mut app = effect_test_app();
    app.insert_resource(FadeInConfig {
        duration: 0.5,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<FadeIn>::new());
    app.add_systems(Update, fade_in_start);
    app.update();

    let sprites: Vec<&Sprite> = app
        .world_mut()
        .query_filtered::<&Sprite, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    assert_eq!(sprites.len(), 1, "exactly 1 overlay should exist");
    let alpha = sprites[0].color.alpha();
    assert!(
        (alpha - 1.0).abs() < f32::EPSILON,
        "FadeIn overlay alpha should start at 1.0, got {alpha}"
    );
}

#[test]
fn fade_in_start_overlay_has_correct_size_and_z_index() {
    let mut app = effect_test_app();
    app.insert_resource(ScreenSize(Vec2::new(1920.0, 1080.0)));
    app.insert_resource(FadeInConfig {
        duration: 0.5,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<FadeIn>::new());
    app.add_systems(Update, fade_in_start);
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
fn fade_in_start_sends_transition_ready() {
    let mut app = effect_test_app();
    app.insert_resource(FadeInConfig {
        duration: 0.5,
        color: Color::BLACK,
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
    let mut app = effect_test_app();
    app.insert_resource(FadeInConfig {
        duration: 0.5,
        color: Color::BLACK,
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
fn fade_in_start_with_blue_color_preserves_rgb() {
    let mut app = effect_test_app();
    app.insert_resource(FadeInConfig {
        duration: 0.5,
        color: Color::srgba(0.0, 0.0, 1.0, 1.0),
    });
    app.insert_resource(StartingTransition::<FadeIn>::new());
    app.add_systems(Update, fade_in_start);
    app.update();

    let sprites: Vec<&Sprite> = app
        .world_mut()
        .query_filtered::<&Sprite, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    let srgba = sprites[0].color.to_srgba();
    assert!(srgba.red.abs() < f32::EPSILON);
    assert!(srgba.green.abs() < f32::EPSILON);
    assert!((srgba.blue - 1.0).abs() < f32::EPSILON);
    assert!(
        (srgba.alpha - 1.0).abs() < f32::EPSILON,
        "FadeIn alpha should start at 1.0"
    );
}

// --- Behavior 9: FadeIn run system decreases alpha ---

#[test]
fn fade_in_run_decreases_alpha_based_on_progress() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<FadeIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.25,
        duration: 1.0,
        completed: false,
    });
    app.world_mut().spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 1.0),
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        TransitionOverlay,
    ));
    app.add_systems(Update, fade_in_run);
    app.update();

    let sprites: Vec<&Sprite> = app
        .world_mut()
        .query_filtered::<&Sprite, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    let alpha = sprites[0].color.alpha();
    assert!(
        (alpha - 0.75).abs() < 0.01,
        "alpha should be ~0.75 (1.0 - 0.25), got {alpha}"
    );
}

#[test]
fn fade_in_run_sends_complete_at_full_progress() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<FadeIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: false,
    });
    app.world_mut().spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 1.0),
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        TransitionOverlay,
    ));
    app.add_systems(Update, fade_in_run);
    app.update();

    let sprites: Vec<&Sprite> = app
        .world_mut()
        .query_filtered::<&Sprite, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    let alpha = sprites[0].color.alpha();
    assert!(
        alpha.abs() < f32::EPSILON,
        "alpha should be 0.0 at full progress"
    );

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

// --- Behavior 10: FadeIn run sends complete when fully transparent ---

#[test]
fn fade_in_run_clamps_alpha_to_zero_on_overshoot() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<FadeIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.4,
        completed: false,
    });
    app.world_mut().spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 1.0),
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        TransitionOverlay,
    ));
    app.add_systems(Update, fade_in_run);
    app.update();

    let sprites: Vec<&Sprite> = app
        .world_mut()
        .query_filtered::<&Sprite, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    let alpha = sprites[0].color.alpha();
    assert!(alpha >= 0.0, "alpha should never go negative, got {alpha}");
    assert!(
        alpha.abs() < f32::EPSILON,
        "alpha should be clamped to 0.0 on overshoot"
    );
}

#[test]
fn fade_in_run_does_not_double_send_complete_when_already_completed() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<FadeIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: true,
    });
    app.world_mut().spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        TransitionOverlay,
    ));
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

// --- Behavior 11: FadeIn end system ---

#[test]
fn fade_in_end_despawns_overlay_and_sends_transition_over() {
    let mut app = effect_test_app();
    app.insert_resource(EndingTransition::<FadeIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.5,
        completed: true,
    });
    app.world_mut()
        .spawn((Sprite::default(), TransitionOverlay));
    app.add_systems(Update, fade_in_end);
    app.update();

    let overlay_count = app
        .world_mut()
        .query_filtered::<Entity, With<TransitionOverlay>>()
        .iter(app.world())
        .count();
    assert_eq!(overlay_count, 0, "overlay should be despawned");
    assert!(
        !app.world().contains_resource::<TransitionProgress>(),
        "TransitionProgress should be removed"
    );

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionOver>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

#[test]
fn fade_in_end_does_not_panic_when_no_overlay_exists() {
    let mut app = effect_test_app();
    app.insert_resource(EndingTransition::<FadeIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
        duration: 0.5,
        completed: true,
    });
    app.add_systems(Update, fade_in_end);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionOver>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

// --- Behavior 12: FadeIn default configuration ---

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

// =======================================================================
// Section 13: insert_starting overrides (behavior 58)
// =======================================================================

#[test]
fn fade_in_insert_starting_inserts_marker_and_config() {
    use crate::transition::traits::Transition;
    let mut world = World::new();
    let effect = FadeIn {
        duration: 0.7,
        color: Color::srgba(1.0, 0.0, 0.0, 1.0),
    };
    effect.insert_starting(&mut world);

    assert!(
        world.contains_resource::<StartingTransition<FadeIn>>(),
        "StartingTransition<FadeIn> should be inserted"
    );
    assert!(
        world.contains_resource::<FadeInConfig>(),
        "FadeInConfig should be inserted by insert_starting"
    );
}
