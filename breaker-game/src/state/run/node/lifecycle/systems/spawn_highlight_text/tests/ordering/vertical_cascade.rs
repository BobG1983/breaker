//! Tests for vertical cascade positioning by `spawn_order` and existing
//! popup offset behavior.

use bevy::prelude::*;

use crate::{
    fx::FadeOut,
    state::run::{
        components::HighlightPopup, messages::HighlightTriggered,
        node::lifecycle::systems::spawn_highlight_text::tests::helpers::*,
        resources::HighlightKind,
    },
};

// ---------------------------------------------------------------
// Behavior 6: Vertical cascade by spawn_order
// ---------------------------------------------------------------

#[test]
fn first_popup_y_equals_popup_base_y() {
    // Default popup_base_y = 100.0
    let mut app = test_app();
    app.insert_resource(TestHighlightMsg(vec![HighlightTriggered {
        kind: HighlightKind::ClutchClear,
    }]));
    app.update();

    let transform = app
        .world_mut()
        .query_filtered::<&Transform, With<HighlightPopup>>()
        .iter(app.world())
        .next()
        .expect("popup should have Transform");

    // y = popup_base_y + popup_vertical_spacing * 0 = 100.0
    assert!(
        (transform.translation.y - 100.0).abs() < f32::EPSILON,
        "first popup y should be 100.0, got {}",
        transform.translation.y
    );
}

#[test]
fn second_popup_y_staggers_by_vertical_spacing() {
    // Default popup_base_y=100.0, popup_vertical_spacing=50.0
    // First: y=100.0, Second: y=150.0
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

    let mut ys: Vec<f32> = app
        .world_mut()
        .query_filtered::<&Transform, With<HighlightPopup>>()
        .iter(app.world())
        .map(|t| t.translation.y)
        .collect();
    ys.sort_by(f32::total_cmp);

    assert_eq!(ys.len(), 2, "should have 2 popups");
    assert!(
        (ys[0] - 100.0).abs() < f32::EPSILON,
        "first popup y should be 100.0, got {}",
        ys[0]
    );
    assert!(
        (ys[1] - 150.0).abs() < f32::EPSILON,
        "second popup y should be 150.0, got {}",
        ys[1]
    );
}

// ---------------------------------------------------------------
// Behavior 7: Existing popups affect spawn_order
// ---------------------------------------------------------------

#[test]
fn existing_popups_offset_spawn_order() {
    // 2 existing HighlightPopup + 1 new message
    // spawn_order = 2 + 0 = 2
    // y = 100.0 + 50.0 * 2 = 200.0
    // timer = popup_fade_duration_secs + popup_cascade_stagger_secs * 2 = 0.8 + 0.1 * 2 = 1.0
    let mut app = test_app();

    // Spawn 2 pre-existing popup entities
    app.world_mut().spawn((
        HighlightPopup,
        FadeOut {
            timer: 0.5,
            duration: 0.8,
        },
        Transform::from_xyz(0.0, 100.0, 10.0),
    ));
    app.world_mut().spawn((
        HighlightPopup,
        FadeOut {
            timer: 0.4,
            duration: 0.8,
        },
        Transform::from_xyz(0.0, 150.0, 10.0),
    ));

    app.insert_resource(TestHighlightMsg(vec![HighlightTriggered {
        kind: HighlightKind::PerfectStreak,
    }]));
    app.update();

    // The new popup should be at spawn_order=2
    // We can identify it as the one with Text2d component
    let (new_popup_transform, new_popup_fade) = app
        .world_mut()
        .query_filtered::<(&Transform, &FadeOut), (With<HighlightPopup>, With<Text2d>)>()
        .iter(app.world())
        .next()
        .expect("new popup should exist");

    let new_popup_y = new_popup_transform.translation.y;
    let new_popup_timer = new_popup_fade.timer;

    assert!(
        (new_popup_y - 200.0).abs() < f32::EPSILON,
        "new popup y with 2 existing should be 200.0, got {new_popup_y}"
    );

    // Timer stagger: 0.8 + 0.1 * 2 = 1.0
    assert!(
        (new_popup_timer - 1.0).abs() < f32::EPSILON,
        "new popup timer with spawn_order=2 should be 1.0 (0.8 + 0.1*2), got {new_popup_timer}"
    );
}
