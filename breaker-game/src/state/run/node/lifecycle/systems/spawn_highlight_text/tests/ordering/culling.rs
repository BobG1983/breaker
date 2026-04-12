//! Tests for max-visible culling, no-culling below max, no-messages edge
//! case, and regression for new-messages-alone exceeding max.

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
            timer:    0.1,
            duration: 0.8,
        },
        Transform::from_xyz(0.0, 100.0, 10.0),
    ));
    app.world_mut().spawn((
        HighlightPopup,
        Text2d::new("OLD2"),
        FadeOut {
            timer:    0.2,
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
