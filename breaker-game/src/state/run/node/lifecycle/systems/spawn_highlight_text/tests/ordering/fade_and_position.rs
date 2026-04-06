//! Tests for `FadeOut` duration, cascade stagger, z-position, `PunchScale`,
//! and `CleanupOnExit<NodeState>` / `HighlightPopup` markers.

use bevy::prelude::*;
use rantzsoft_lifecycle::CleanupOnExit;

use crate::{
    fx::{FadeOut, PunchScale},
    state::{
        run::{
            components::HighlightPopup, messages::HighlightTriggered,
            node::lifecycle::systems::spawn_highlight_text::tests::helpers::*,
            resources::HighlightKind,
        },
        types::NodeState,
    },
};

// ---------------------------------------------------------------
// Behavior 2: FadeOut uses popup_fade_duration_secs from config
// ---------------------------------------------------------------

#[test]
fn popup_fade_uses_config_duration() {
    // Default popup_fade_duration_secs = 0.8
    let mut app = test_app();
    app.insert_resource(TestHighlightMsg(vec![HighlightTriggered {
        kind: HighlightKind::ClutchClear,
    }]));
    app.update();

    let fade = app
        .world_mut()
        .query::<&FadeOut>()
        .iter(app.world())
        .next()
        .expect("popup should have FadeOut");

    // First popup (spawn_order=0): timer = duration = popup_fade_duration_secs
    assert!(
        (fade.duration - 0.8).abs() < f32::EPSILON,
        "FadeOut.duration should be 0.8 (popup_fade_duration_secs), got {}",
        fade.duration
    );
    assert!(
        (fade.timer - 0.8).abs() < f32::EPSILON,
        "FadeOut.timer for first popup should be 0.8, got {}",
        fade.timer
    );
}

#[test]
fn second_popup_gets_cascade_stagger_added_to_timer() {
    // Default popup_cascade_stagger_secs = 0.1
    // Second popup in same frame: timer = 0.8 + 0.1*1 = 0.9
    let mut app = test_app();
    app.insert_resource(TestHighlightMsg(vec![
        HighlightTriggered {
            kind: HighlightKind::ClutchClear,
        },
        HighlightTriggered {
            kind: HighlightKind::NoDamageNode,
        },
    ]));
    app.update();

    let mut fades: Vec<(f32, f32)> = app
        .world_mut()
        .query::<&FadeOut>()
        .iter(app.world())
        .map(|f| (f.timer, f.duration))
        .collect();
    fades.sort_by(|a, b| a.0.total_cmp(&b.0));

    assert_eq!(fades.len(), 2, "should have 2 popup entities");
    // First popup: timer = 0.8
    assert!(
        (fades[0].0 - 0.8).abs() < f32::EPSILON,
        "first popup timer should be 0.8, got {}",
        fades[0].0
    );
    // Second popup: timer = 0.8 + 0.1*1 = 0.9
    assert!(
        (fades[1].0 - 0.9).abs() < f32::EPSILON,
        "second popup timer should be 0.9 (staggered), got {}",
        fades[1].0
    );
    // Both should have same duration
    assert!(
        (fades[0].1 - 0.8).abs() < f32::EPSILON,
        "both fades should have duration 0.8"
    );
    assert!(
        (fades[1].1 - 0.8).abs() < f32::EPSILON,
        "both fades should have duration 0.8"
    );
}

// ---------------------------------------------------------------
// Behavior 3: Popup spawns at x=0.0 with z=10.0
// ---------------------------------------------------------------

#[test]
fn popup_spawns_at_z_10() {
    let mut app = test_app();
    app.insert_resource(TestHighlightMsg(vec![HighlightTriggered {
        kind: HighlightKind::ClutchClear,
    }]));
    app.update();

    let transform = app
        .world_mut()
        .query::<&Transform>()
        .iter(app.world())
        .next()
        .expect("popup should have Transform");

    assert!(
        (transform.translation.z - 10.0).abs() < f32::EPSILON,
        "popup z should be 10.0, got {}",
        transform.translation.z
    );
}

// ---------------------------------------------------------------
// Behavior 4: PunchScale from config
// ---------------------------------------------------------------

#[test]
fn popup_has_punch_scale_from_config() {
    // Default: overshoot_duration=0.1, overshoot_scale=1.15
    let mut app = test_app();
    app.insert_resource(TestHighlightMsg(vec![HighlightTriggered {
        kind: HighlightKind::ClutchClear,
    }]));
    app.update();

    let punch = app
        .world_mut()
        .query::<&PunchScale>()
        .iter(app.world())
        .next()
        .expect("popup should have PunchScale");

    assert!(
        (punch.timer - 0.1).abs() < f32::EPSILON,
        "PunchScale.timer should be 0.1, got {}",
        punch.timer
    );
    assert!(
        (punch.duration - 0.1).abs() < f32::EPSILON,
        "PunchScale.duration should be 0.1, got {}",
        punch.duration
    );
    assert!(
        (punch.overshoot - 1.15).abs() < f32::EPSILON,
        "PunchScale.overshoot should be 1.15, got {}",
        punch.overshoot
    );
}

// ---------------------------------------------------------------
// Behavior 5: CleanupOnExit<NodeState> and HighlightPopup markers
// ---------------------------------------------------------------

#[test]
fn popup_has_cleanup_and_highlight_markers() {
    let mut app = test_app();
    app.insert_resource(TestHighlightMsg(vec![HighlightTriggered {
        kind: HighlightKind::ClutchClear,
    }]));
    app.update();

    let count = app
        .world_mut()
        .query_filtered::<Entity, (With<CleanupOnExit<NodeState>>, With<HighlightPopup>)>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 1,
        "popup should have both CleanupOnExit<NodeState> and HighlightPopup"
    );
}
