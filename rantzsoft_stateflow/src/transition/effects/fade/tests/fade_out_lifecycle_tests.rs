//! Tests for the Out -> In transition lifecycle.
//!
//! An Out transition hides content behind an overlay. When it completes,
//! the overlay must remain at full opacity and time must stay paused.
//! A subsequent In transition replaces the overlay, plays its reveal,
//! and on completion removes the overlay and unpauses time.

use std::sync::Arc;

use bevy::{prelude::*, state::app::StatesPlugin};

use super::helpers::effect_test_app;
use crate::{
    RantzStateflowPlugin, Route, RoutingTableAppExt,
    transition::{
        effects::{fade::effect::*, post_process::TransitionEffect, shared::TransitionProgress},
        resources::{ActiveTransition, EndingTransition, PendingTransition},
    },
};

// ── Test states: A -> B (Out) -> C (In) ─────────────────────────────

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Screen {
    #[default]
    A,
    B,
    C,
}

fn lifecycle_app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<Screen>()
        .add_plugins(RantzStateflowPlugin::new().register_state::<Screen>());
    let camera = app.world_mut().spawn(Camera2d).id();
    (app, camera)
}

/// Drive the app forward, force-completing `TransitionProgress` each frame.
fn drive_transition(app: &mut App, max_frames: usize) -> usize {
    for frame in 0..max_frames {
        if app.world().contains_resource::<TransitionProgress>() {
            let mut progress = app.world_mut().resource_mut::<TransitionProgress>();
            if !progress.completed {
                progress.elapsed = progress.duration + 0.1;
            }
        }
        app.update();

        if !app.world().contains_resource::<ActiveTransition>()
            && !app.world().contains_resource::<PendingTransition>()
        {
            return frame + 1;
        }
    }
    max_frames
}

// ── Unit tests: `FadeOut` end behavior ───────────────────────────────

/// After `FadeOut` completes, the overlay must remain on the camera at
/// progress=1.0 (fully opaque).
#[test]
fn fade_out_end_preserves_overlay_on_camera() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color:       Vec4::new(1.0, 1.0, 1.0, 1.0),
        direction:   Vec4::ZERO,
        effect_type: crate::transition::effects::post_process::EffectType::FADE,
        progress:    1.0,
    });
    app.insert_resource(EndingTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   0.5,
        duration:  0.5,
        completed: true,
    });
    app.add_systems(Update, fade_out_end);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(
        effects.len(),
        1,
        "TransitionEffect must remain on camera after FadeOut end"
    );
    assert!(
        (effects[0].progress - 1.0).abs() < f32::EPSILON,
        "overlay progress must stay at 1.0 (fully opaque)"
    );
}

/// `FadeOut` end must still send `TransitionOver`.
#[test]
fn fade_out_end_sends_transition_over() {
    let (mut app, camera) = effect_test_app();
    app.world_mut()
        .entity_mut(camera)
        .insert(TransitionEffect::default());
    app.insert_resource(EndingTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   0.5,
        duration:  0.5,
        completed: true,
    });
    app.add_systems(Update, fade_out_end);
    app.update();

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<crate::transition::messages::TransitionOver>>();
    assert_eq!(
        msgs.iter_current_update_messages().count(),
        1,
        "exactly 1 TransitionOver should be sent"
    );
}

/// `FadeOut` end removes `TransitionProgress`.
#[test]
fn fade_out_end_removes_transition_progress() {
    let (mut app, camera) = effect_test_app();
    app.world_mut()
        .entity_mut(camera)
        .insert(TransitionEffect::default());
    app.insert_resource(EndingTransition::<FadeOut>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   0.5,
        duration:  0.5,
        completed: true,
    });
    app.add_systems(Update, fade_out_end);
    app.update();

    assert!(
        !app.world().contains_resource::<TransitionProgress>(),
        "TransitionProgress should be removed by FadeOut end"
    );
}

/// `FadeIn` start replaces any existing overlay left by `FadeOut`.
#[test]
fn fade_in_start_replaces_existing_overlay() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color:       Vec4::ONE,
        direction:   Vec4::ZERO,
        effect_type: crate::transition::effects::post_process::EffectType::FADE,
        progress:    1.0,
    });
    app.insert_resource(FadeInConfig {
        duration: 0.6,
        color:    Color::BLACK,
    });
    app.insert_resource(crate::transition::resources::StartingTransition::<FadeIn>::new());
    app.add_systems(Update, fade_in_start);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(effects.len(), 1, "should have exactly 1 TransitionEffect");
    assert!(
        (effects[0].progress - 1.0).abs() < f32::EPSILON,
        "FadeIn should start at progress=1.0 (fully opaque)"
    );
}

/// After `FadeIn` completes, the overlay must be removed from the camera.
#[test]
fn fade_in_end_removes_overlay_from_camera() {
    let (mut app, camera) = effect_test_app();
    app.world_mut().entity_mut(camera).insert(TransitionEffect {
        color:       Vec4::new(0.0, 0.0, 0.0, 1.0),
        direction:   Vec4::ZERO,
        effect_type: crate::transition::effects::post_process::EffectType::FADE,
        progress:    0.0,
    });
    app.insert_resource(EndingTransition::<FadeIn>::new());
    app.insert_resource(TransitionProgress {
        elapsed:   0.6,
        duration:  0.6,
        completed: true,
    });
    app.add_systems(Update, fade_in_end);
    app.update();

    let effects: Vec<&TransitionEffect> = app
        .world_mut()
        .query_filtered::<&TransitionEffect, With<Camera2d>>()
        .iter(app.world())
        .collect();
    assert_eq!(
        effects.len(),
        0,
        "TransitionEffect must be removed after FadeIn end"
    );
}

// ── Integration: Full Out -> In lifecycle with real routes ────────────

/// A -> B via `FadeOut` (Out transition). After Out completes:
/// - State is B
/// - Overlay exists on camera at progress=1.0
/// - `Time<Virtual>` is paused
///
/// Then B -> C via `FadeIn` (In transition). After In completes:
/// - State is C
/// - Overlay is gone
/// - `Time<Virtual>` is unpaused
#[test]
fn out_then_in_lifecycle() {
    let (mut app, camera) = lifecycle_app();

    // A -> B: Out transition (hides content, defers state change)
    app.add_route(
        Route::from(Screen::A)
            .to(Screen::B)
            .with_transition(crate::transition::types::TransitionType::Out(Arc::new(
                FadeOut {
                    duration: 0.5,
                    color:    Color::WHITE,
                },
            )))
            .when(|_| true),
    );

    // B -> C: In transition (reveals new content, applies state immediately)
    app.add_route(
        Route::from(Screen::B)
            .to(Screen::C)
            .with_transition(crate::transition::types::TransitionType::In(Arc::new(
                FadeIn {
                    duration: 0.6,
                    color:    Color::WHITE,
                },
            )))
            .when(|_| true),
    );

    // Drive Out transition A -> B to completion
    let frames = drive_transition(&mut app, 20);
    assert!(frames < 20, "Out transition should complete");

    // One more update for Bevy to flush NextState -> State
    app.update();

    // ── Assert intermediate state: B, overlay persists, time paused ──
    let state = *app.world().resource::<State<Screen>>().get();
    assert_eq!(state, Screen::B, "state should be B after Out completes");

    let has_overlay = app.world().get::<TransitionEffect>(camera).is_some();
    assert!(
        has_overlay,
        "overlay must persist after Out transition — removing it exposes stale content"
    );

    let time_paused = app.world().resource::<Time<Virtual>>().is_paused();
    assert!(
        time_paused,
        "Time<Virtual> must remain paused after Out transition"
    );

    // Drive In transition B -> C to completion
    let frames = drive_transition(&mut app, 20);
    assert!(frames < 20, "In transition should complete");

    // One more update for state flush
    app.update();

    // ── Assert final state: C, overlay gone, time unpaused ──
    let state = *app.world().resource::<State<Screen>>().get();
    assert_eq!(state, Screen::C, "state should be C after In completes");

    let has_overlay = app.world().get::<TransitionEffect>(camera).is_some();
    assert!(
        !has_overlay,
        "overlay must be removed after In transition completes"
    );

    let time_paused = app.world().resource::<Time<Virtual>>().is_paused();
    assert!(
        !time_paused,
        "Time<Virtual> must be unpaused after In transition completes"
    );
}
