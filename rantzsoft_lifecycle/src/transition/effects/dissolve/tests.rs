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
// Section 9: DissolveOut
// =======================================================================

// --- Behavior 41: DissolveOut implements Transition and OutTransition ---

#[test]
fn dissolve_out_satisfies_transition_and_out_transition() {
    use crate::transition::traits::OutTransition;
    let _effect: Box<dyn OutTransition> = Box::new(DissolveOut {
        duration: 0.8,
        color: Color::BLACK,
    });
}

// --- Behavior 42: DissolveOut start spawns overlay at zero alpha ---

#[test]
fn dissolve_out_start_spawns_overlay_at_zero_alpha() {
    let mut app = effect_test_app();
    app.insert_resource(DissolveOutConfig {
        duration: 0.8,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<DissolveOut>::new());
    app.add_systems(Update, dissolve_out_start);
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
    let alpha = sprites[0].color.alpha();
    assert!(
        alpha.abs() < f32::EPSILON,
        "DissolveOut should start at alpha 0.0"
    );

    let size = sprites[0].custom_size.unwrap_or_default();
    assert!((size.x - 1920.0).abs() < f32::EPSILON);
    assert!((size.y - 1080.0).abs() < f32::EPSILON);

    let z_indices: Vec<&GlobalZIndex> = app
        .world_mut()
        .query_filtered::<&GlobalZIndex, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    assert_eq!(z_indices[0].0, i32::MAX - 1);

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionReady>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
    assert!(app.world().contains_resource::<TransitionProgress>());
    let progress = app.world().resource::<TransitionProgress>();
    assert!((progress.duration - 0.8).abs() < f32::EPSILON);
}

// --- Behavior 43: DissolveOut run increases alpha with stepped curve ---

#[test]
fn dissolve_out_run_increases_alpha_at_mid_progress() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<DissolveOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
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
    app.add_systems(Update, dissolve_out_run);
    app.update();

    let sprites: Vec<&Sprite> = app
        .world_mut()
        .query_filtered::<&Sprite, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    let alpha = sprites[0].color.alpha();
    assert!(
        alpha > 0.0,
        "alpha should have increased from 0.0 at 50% progress, got {alpha}"
    );
    assert!(alpha <= 1.0, "alpha should not exceed 1.0, got {alpha}");
}

#[test]
fn dissolve_out_run_sends_complete_at_full_progress() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<DissolveOut>::new());
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
    app.add_systems(Update, dissolve_out_run);
    app.update();

    let sprites: Vec<&Sprite> = app
        .world_mut()
        .query_filtered::<&Sprite, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    let alpha = sprites[0].color.alpha();
    assert!(
        (alpha - 1.0).abs() < f32::EPSILON,
        "alpha should be 1.0 at full progress"
    );

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
}

#[test]
fn dissolve_out_run_does_not_double_send_when_already_completed() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<DissolveOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 1.0,
        duration: 1.0,
        completed: true,
    });
    app.world_mut().spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 1.0),
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        TransitionOverlay,
    ));
    app.add_systems(Update, dissolve_out_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 0);
}

// --- Behavior 44: DissolveOut end ---

#[test]
fn dissolve_out_end_despawns_overlay_and_sends_transition_over() {
    let mut app = effect_test_app();
    app.insert_resource(EndingTransition::<DissolveOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.8,
        duration: 0.8,
        completed: true,
    });
    app.world_mut()
        .spawn((Sprite::default(), TransitionOverlay));
    app.add_systems(Update, dissolve_out_end);
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
// Section 10: DissolveIn
// =======================================================================

// --- Behavior 45: DissolveIn implements Transition and InTransition ---

#[test]
fn dissolve_in_satisfies_transition_and_in_transition() {
    use crate::transition::traits::InTransition;
    let _effect: Box<dyn InTransition> = Box::new(DissolveIn {
        duration: 0.8,
        color: Color::BLACK,
    });
}

// --- Behavior 46: DissolveIn start spawns fully opaque overlay ---

#[test]
fn dissolve_in_start_spawns_fully_opaque_overlay() {
    let mut app = effect_test_app();
    app.insert_resource(DissolveInConfig {
        duration: 0.8,
        color: Color::BLACK,
    });
    app.insert_resource(StartingTransition::<DissolveIn>::new());
    app.add_systems(Update, dissolve_in_start);
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
    let alpha = sprites[0].color.alpha();
    assert!(
        (alpha - 1.0).abs() < f32::EPSILON,
        "DissolveIn should start at alpha 1.0"
    );

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionReady>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 1);
    assert!(app.world().contains_resource::<TransitionProgress>());
}

// --- Behavior 47: DissolveIn run decreases alpha with stepped curve ---

#[test]
fn dissolve_in_run_decreases_alpha_at_mid_progress() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<DissolveIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.5,
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
    app.add_systems(Update, dissolve_in_run);
    app.update();

    let sprites: Vec<&Sprite> = app
        .world_mut()
        .query_filtered::<&Sprite, With<TransitionOverlay>>()
        .iter(app.world())
        .collect();
    let alpha = sprites[0].color.alpha();
    assert!(alpha >= 0.0, "alpha should not go negative, got {alpha}");
    assert!(
        alpha < 1.0,
        "alpha should have decreased from 1.0 at 50% progress, got {alpha}"
    );
}

#[test]
fn dissolve_in_run_sends_complete_at_full_progress() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<DissolveIn>::new());
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
    app.add_systems(Update, dissolve_in_run);
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

#[test]
fn dissolve_in_run_does_not_double_send_when_already_completed() {
    let mut app = effect_test_app();
    app.insert_resource(RunningTransition::<DissolveIn>::new());
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
    app.add_systems(Update, dissolve_in_run);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
    assert_eq!(msgs.iter_current_update_messages().count(), 0);
}

// --- Behavior 48: DissolveIn end ---

#[test]
fn dissolve_in_end_despawns_overlay_and_sends_transition_over() {
    let mut app = effect_test_app();
    app.insert_resource(EndingTransition::<DissolveIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed: 0.8,
        duration: 0.8,
        completed: true,
    });
    app.world_mut()
        .spawn((Sprite::default(), TransitionOverlay));
    app.add_systems(Update, dissolve_in_end);
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
// Section 13: insert_starting overrides (behaviors 65-66)
// =======================================================================

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
    assert!(
        world.contains_resource::<DissolveOutConfig>(),
        "DissolveOutConfig should be inserted by insert_starting"
    );
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
    assert!(
        world.contains_resource::<DissolveInConfig>(),
        "DissolveInConfig should be inserted by insert_starting"
    );
}
