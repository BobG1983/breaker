use bevy::prelude::*;

use super::helpers::*;
use crate::{
    fx::{FadeOut, PunchScale},
    run::{components::HighlightPopup, messages::HighlightTriggered, resources::HighlightKind},
    shared::{CleanupOnNodeExit, GameRng},
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
// Behavior 5: CleanupOnNodeExit and HighlightPopup markers
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
        .query_filtered::<Entity, (With<CleanupOnNodeExit>, With<HighlightPopup>)>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 1,
        "popup should have both CleanupOnNodeExit and HighlightPopup"
    );
}

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

// ---------------------------------------------------------------
// Behavior 10: Max visible culling
// ---------------------------------------------------------------

#[test]
fn excess_popups_culled_to_max_visible() {
    // max_visible=3 (default), 2 existing + 3 new = 5 total
    // After spawning 3 new and culling: total should be 3
    // Cull 2 oldest (smallest FadeOut.timer values: 0.1 and 0.2)
    let mut app = test_app();

    // Spawn 2 pre-existing popups with small timers (old)
    app.world_mut().spawn((
        HighlightPopup,
        Text2d::new("OLD1"),
        FadeOut {
            timer: 0.1,
            duration: 0.8,
        },
        Transform::from_xyz(0.0, 100.0, 10.0),
    ));
    app.world_mut().spawn((
        HighlightPopup,
        Text2d::new("OLD2"),
        FadeOut {
            timer: 0.2,
            duration: 0.8,
        },
        Transform::from_xyz(0.0, 150.0, 10.0),
    ));

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

    let total = app
        .world_mut()
        .query_filtered::<Entity, With<HighlightPopup>>()
        .iter(app.world())
        .count();

    assert_eq!(
        total, 3,
        "total popups after culling should be max_visible=3, got {total}"
    );

    // Verify the oldest entities (timer 0.1 and 0.2) were culled --
    // all surviving timers should be >= new popup base timer (0.8).
    let surviving_timers: Vec<f32> = app
        .world_mut()
        .query_filtered::<&FadeOut, With<HighlightPopup>>()
        .iter(app.world())
        .map(|f| f.timer)
        .collect();

    for timer in &surviving_timers {
        assert!(
            *timer >= 0.8,
            "surviving popup timer should be >= 0.8 (new popup base), got {timer}; \
             oldest popups (0.1, 0.2) should have been culled"
        );
    }

    // Verify the old text labels are gone
    let surviving_texts: Vec<String> = app
        .world_mut()
        .query_filtered::<&Text2d, With<HighlightPopup>>()
        .iter(app.world())
        .map(|t| t.0.clone())
        .collect();

    assert!(
        !surviving_texts.iter().any(|t| t == "OLD1"),
        "OLD1 (timer 0.1) should have been culled, but found in: {surviving_texts:?}"
    );
    assert!(
        !surviving_texts.iter().any(|t| t == "OLD2"),
        "OLD2 (timer 0.2) should have been culled, but found in: {surviving_texts:?}"
    );
}

// ---------------------------------------------------------------
// Behavior 11: No culling when at or below max
// ---------------------------------------------------------------

#[test]
fn no_culling_when_below_max_visible() {
    // 0 existing + 2 new = 2 total, max_visible=3
    // No culling needed
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

    let total = app
        .world_mut()
        .query_filtered::<Entity, With<HighlightPopup>>()
        .iter(app.world())
        .count();

    assert_eq!(
        total, 2,
        "should have 2 popups with no culling, got {total}"
    );
}

// ---------------------------------------------------------------
// Behavior 12: No messages spawns nothing
// ---------------------------------------------------------------

#[test]
fn no_messages_spawns_nothing() {
    let mut app = test_app();
    app.insert_resource(TestHighlightMsg(vec![]));
    app.update();

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<HighlightPopup>>()
        .iter(app.world())
        .count();
    assert_eq!(count, 0, "no messages should spawn no popup entities");
}

// ---------------------------------------------------------------
// Regression: new messages alone exceed max_visible with 0 existing
// ---------------------------------------------------------------

#[test]
fn culling_limits_popups_when_new_messages_alone_exceed_max_visible() {
    // Bug: when 5 HighlightTriggered messages arrive in one frame with
    // 0 existing popups and max_visible=3, all 5 spawn and none are
    // culled because the cull loop only iterates existing (pre-spawn)
    // entities -- newly-spawned entities are invisible to the query
    // until commands flush.
    //
    // Correct behavior: exactly 3 HighlightPopup entities should exist
    // after the system runs.
    let mut app = test_app();

    // 0 existing popups, 5 new messages, max_visible=3 (default)
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
        HighlightTriggered {
            kind: HighlightKind::FastClear,
        },
        HighlightTriggered {
            kind: HighlightKind::Comeback,
        },
    ]));
    app.update();

    let total = app
        .world_mut()
        .query_filtered::<Entity, With<HighlightPopup>>()
        .iter(app.world())
        .count();

    assert_eq!(
        total, 3,
        "with 0 existing popups and 5 new messages, exactly popup_max_visible=3 \
         popups should exist after culling, got {total}"
    );
}
