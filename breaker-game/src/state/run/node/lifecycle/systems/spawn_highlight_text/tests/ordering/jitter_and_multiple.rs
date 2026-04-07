//! Tests for x-jitter from `GameRng`, deterministic seeding, and
//! distinct positions/stagger for multiple popups.

use bevy::prelude::*;

use crate::{
    fx::FadeOut,
    shared::GameRng,
    state::run::{
        components::HighlightPopup, messages::HighlightTriggered,
        node::lifecycle::systems::spawn_highlight_text::tests::helpers::*,
        resources::HighlightKind,
    },
};

// ---------------------------------------------------------------
// Behavior 8: Jitter x-position from GameRng
// ---------------------------------------------------------------

#[test]
fn popup_x_jitter_within_configured_range() {
    // Default: popup_jitter_min_x = -10.0, popup_jitter_max_x = 10.0
    let mut app = test_app();
    app.insert_resource(GameRng::from_seed(42));
    app.insert_resource(TestHighlightMsg(vec![HighlightTriggered {
        kind: HighlightKind::ClutchClear,
    }]));
    app.update();

    let x = app
        .world_mut()
        .query_filtered::<&Transform, With<HighlightPopup>>()
        .iter(app.world())
        .next()
        .expect("popup should exist")
        .translation
        .x;

    assert!(
        (-10.0..=10.0).contains(&x),
        "popup x should be within [-10.0, 10.0], got {x}"
    );
}

#[test]
fn popup_x_jitter_is_deterministic_for_same_seed() {
    // Same seed should produce same jitter
    let run_with_seed = |seed: u64| -> f32 {
        let mut app = test_app();
        app.insert_resource(GameRng::from_seed(seed));
        app.insert_resource(TestHighlightMsg(vec![HighlightTriggered {
            kind: HighlightKind::ClutchClear,
        }]));
        app.update();

        app.world_mut()
            .query_filtered::<&Transform, With<HighlightPopup>>()
            .iter(app.world())
            .next()
            .expect("popup should exist")
            .translation
            .x
    };

    let x1 = run_with_seed(42);
    let x2 = run_with_seed(42);
    assert!(
        (x1 - x2).abs() < f32::EPSILON,
        "same seed should produce same jitter: {x1} vs {x2}"
    );
}

// ---------------------------------------------------------------
// Behavior 9: Multiple popups get distinct positions and stagger
// ---------------------------------------------------------------

#[test]
fn three_popups_get_distinct_y_positions_and_staggered_timers() {
    // 3 messages in one frame:
    // y positions: 100, 150, 200
    // FadeOut timers: 0.8, 0.9, 1.0 (stagger adds 0.1 per spawn_order)
    let mut app = test_app();
    app.insert_resource(TestHighlightMsg(vec![
        HighlightTriggered {
            kind: HighlightKind::ClutchClear,
        },
        HighlightTriggered {
            kind: HighlightKind::NoDamageNode,
        },
        HighlightTriggered {
            kind: HighlightKind::PerfectStreak,
        },
    ]));
    app.update();

    let mut popups: Vec<(f32, f32)> = app
        .world_mut()
        .query_filtered::<(&Transform, &FadeOut), With<HighlightPopup>>()
        .iter(app.world())
        .map(|(t, f)| (t.translation.y, f.timer))
        .collect();
    popups.sort_by(|a, b| a.0.total_cmp(&b.0));

    assert_eq!(popups.len(), 3, "should have 3 popups");

    // Y positions: 100, 150, 200
    assert!(
        (popups[0].0 - 100.0).abs() < f32::EPSILON,
        "first y should be 100.0, got {}",
        popups[0].0
    );
    assert!(
        (popups[1].0 - 150.0).abs() < f32::EPSILON,
        "second y should be 150.0, got {}",
        popups[1].0
    );
    assert!(
        (popups[2].0 - 200.0).abs() < f32::EPSILON,
        "third y should be 200.0, got {}",
        popups[2].0
    );

    // Sort by timer for stagger check
    let mut timers: Vec<f32> = popups.iter().map(|(_, t)| *t).collect();
    timers.sort_by(f32::total_cmp);

    assert!(
        (timers[0] - 0.8).abs() < f32::EPSILON,
        "first timer should be 0.8, got {}",
        timers[0]
    );
    assert!(
        (timers[1] - 0.9).abs() < f32::EPSILON,
        "second timer should be 0.9, got {}",
        timers[1]
    );
    assert!(
        (timers[2] - 1.0).abs() < f32::EPSILON,
        "third timer should be 1.0, got {}",
        timers[2]
    );
}
