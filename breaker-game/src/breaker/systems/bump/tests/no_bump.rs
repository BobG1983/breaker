//! Tests for `NoBump` message emission from `update_bump`.
//!
//! `NoBump` is sent when the retroactive post-hit timer expires without
//! the player pressing bump input during the window.

use super::helpers::*;
use crate::breaker::{
    components::{Breaker, BumpState},
    definition::BreakerDefinition,
};

// ---------------------------------------------------------------------------
// Behavior 1: NoBump sent when post_hit_timer transitions to zero without input
// ---------------------------------------------------------------------------

#[test]
fn no_bump_sent_when_post_hit_timer_expires_without_input() {
    let mut app = update_bump_with_no_bump_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app
        .world_mut()
        .spawn((
            Breaker,
            BumpState {
                post_hit_timer: 0.01, // less than default dt (1/64 ~ 0.015625)
                last_hit_bolt:  Some(bolt_entity),
                active:         false,
                timer:          0.0,
                cooldown:       0.0,
            },
            bump_param_bundle(&BreakerDefinition::default()),
        ))
        .id();

    app.insert_resource(TestInputActive(false));
    tick(&mut app);

    let captured = app.world().resource::<CapturedNoBumps>();
    assert_eq!(
        captured.0.len(),
        1,
        "exactly one NoBump message should be sent when post_hit_timer expires"
    );
    assert_eq!(captured.0[0].bolt, bolt_entity);
    assert_eq!(captured.0[0].breaker, breaker_entity);

    let bump = app.world().get::<BumpState>(breaker_entity).unwrap();
    assert!(
        bump.last_hit_bolt.is_none(),
        "last_hit_bolt should be cleared after NoBump"
    );
    assert!(
        bump.post_hit_timer <= 0.0,
        "post_hit_timer should be at 0.0"
    );
}

// ---------------------------------------------------------------------------
// Behavior 2: NoBump carries correct bolt and breaker entities
// ---------------------------------------------------------------------------

#[test]
fn no_bump_carries_correct_bolt_and_breaker_entities() {
    let mut app = update_bump_with_no_bump_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app
        .world_mut()
        .spawn((
            Breaker,
            BumpState {
                post_hit_timer: 0.01,
                last_hit_bolt:  Some(bolt_entity),
                active:         false,
                timer:          0.0,
                cooldown:       0.0,
            },
            bump_param_bundle(&BreakerDefinition::default()),
        ))
        .id();

    app.insert_resource(TestInputActive(false));
    tick(&mut app);

    let captured = app.world().resource::<CapturedNoBumps>();
    assert_eq!(captured.0.len(), 1, "should emit one NoBump");
    assert_eq!(
        captured.0[0].bolt, bolt_entity,
        "NoBump.bolt should match the bolt entity from last_hit_bolt"
    );
    assert_eq!(
        captured.0[0].breaker, breaker_entity,
        "NoBump.breaker should match the breaker entity"
    );
}

// ---------------------------------------------------------------------------
// Behavior 3: NoBump NOT sent when player presses bump during retroactive window
// ---------------------------------------------------------------------------

#[test]
fn no_bump_not_sent_when_bump_pressed_during_retroactive_window() {
    let mut app = update_bump_with_no_bump_test_app();
    let config = BreakerDefinition::default();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app
        .world_mut()
        .spawn((
            Breaker,
            BumpState {
                post_hit_timer: 0.10,
                last_hit_bolt:  Some(bolt_entity),
                active:         false,
                timer:          0.0,
                cooldown:       0.0,
            },
            bump_param_bundle(&config),
        ))
        .id();

    app.insert_resource(TestInputActive(true));
    tick(&mut app);

    let no_bumps = app.world().resource::<CapturedNoBumps>();
    assert!(
        no_bumps.0.is_empty(),
        "no NoBump should be sent when player presses bump during retroactive window"
    );

    let bumps = app.world().resource::<CapturedBumps>();
    assert_eq!(
        bumps.0.len(),
        1,
        "retroactive BumpPerformed should be emitted instead"
    );

    let bump = app.world().get::<BumpState>(breaker_entity).unwrap();
    assert!(
        bump.last_hit_bolt.is_none(),
        "last_hit_bolt should be consumed by the retroactive path"
    );
    assert!(
        bump.post_hit_timer <= 0.0,
        "post_hit_timer should be cleared by the retroactive path"
    );
}

// ---------------------------------------------------------------------------
// Behavior 4: NoBump NOT sent when post_hit_timer is 0 and last_hit_bolt is None
// ---------------------------------------------------------------------------

#[test]
fn no_bump_not_sent_in_idle_state() {
    let mut app = update_bump_with_no_bump_test_app();

    let _breaker_entity = app
        .world_mut()
        .spawn((
            Breaker,
            BumpState {
                post_hit_timer: 0.0,
                last_hit_bolt:  None,
                active:         false,
                timer:          0.0,
                cooldown:       0.0,
            },
            bump_param_bundle(&BreakerDefinition::default()),
        ))
        .id();

    app.insert_resource(TestInputActive(false));
    tick(&mut app);

    let captured = app.world().resource::<CapturedNoBumps>();
    assert!(
        captured.0.is_empty(),
        "no NoBump should be sent in idle state (post_hit_timer=0, last_hit_bolt=None)"
    );
}

// ---------------------------------------------------------------------------
// Behavior 5: NoBump NOT sent when post_hit_timer positive but last_hit_bolt is None
// ---------------------------------------------------------------------------

#[test]
fn no_bump_not_sent_when_last_hit_bolt_is_none() {
    let mut app = update_bump_with_no_bump_test_app();

    let _breaker_entity = app
        .world_mut()
        .spawn((
            Breaker,
            BumpState {
                post_hit_timer: 0.01,
                last_hit_bolt:  None,
                active:         false,
                timer:          0.0,
                cooldown:       0.0,
            },
            bump_param_bundle(&BreakerDefinition::default()),
        ))
        .id();

    app.insert_resource(TestInputActive(false));
    tick(&mut app);

    let captured = app.world().resource::<CapturedNoBumps>();
    assert!(
        captured.0.is_empty(),
        "no NoBump should be sent when last_hit_bolt is None even if post_hit_timer was positive"
    );
}

// ---------------------------------------------------------------------------
// Behavior 6: NoBump clears last_hit_bolt after sending (no duplicates)
// ---------------------------------------------------------------------------

#[test]
fn no_bump_clears_last_hit_bolt_preventing_duplicate_messages() {
    let mut app = update_bump_with_no_bump_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app
        .world_mut()
        .spawn((
            Breaker,
            BumpState {
                post_hit_timer: 0.01,
                last_hit_bolt:  Some(bolt_entity),
                active:         false,
                timer:          0.0,
                cooldown:       0.0,
            },
            bump_param_bundle(&BreakerDefinition::default()),
        ))
        .id();

    app.insert_resource(TestInputActive(false));
    tick(&mut app);

    // Verify first tick sends exactly one
    let captured = app.world().resource::<CapturedNoBumps>();
    assert_eq!(captured.0.len(), 1, "first tick should send one NoBump");

    let bump = app.world().get::<BumpState>(breaker_entity).unwrap();
    assert!(
        bump.last_hit_bolt.is_none(),
        "last_hit_bolt should be None after NoBump"
    );

    // Second tick: no more NoBump messages
    tick(&mut app);

    let captured = app.world().resource::<CapturedNoBumps>();
    assert_eq!(
        captured.0.len(),
        1,
        "second tick should not produce additional NoBump (still 1 total, not 2)"
    );
}

// ---------------------------------------------------------------------------
// Behavior 7: NoBump sent only once even when timer overshoots past zero
// ---------------------------------------------------------------------------

#[test]
fn no_bump_sent_only_once_when_timer_overshoots_zero() {
    let mut app = update_bump_with_no_bump_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let _breaker_entity = app
        .world_mut()
        .spawn((
            Breaker,
            BumpState {
                post_hit_timer: 0.008, // dt (~0.015625) overshoots this to negative, clamped to 0
                last_hit_bolt:  Some(bolt_entity),
                active:         false,
                timer:          0.0,
                cooldown:       0.0,
            },
            bump_param_bundle(&BreakerDefinition::default()),
        ))
        .id();

    app.insert_resource(TestInputActive(false));
    tick(&mut app);

    let captured = app.world().resource::<CapturedNoBumps>();
    assert_eq!(
        captured.0.len(),
        1,
        "exactly one NoBump even when timer overshoots zero"
    );

    // Second tick should produce no more
    tick(&mut app);

    let captured = app.world().resource::<CapturedNoBumps>();
    assert_eq!(
        captured.0.len(),
        1,
        "no additional NoBump on second tick after overshoot (still 1 total)"
    );
}

// ---------------------------------------------------------------------------
// Behavior 8: NoBump NOT sent when forward window active (bolt hasn't hit yet)
// ---------------------------------------------------------------------------

#[test]
fn no_bump_not_sent_when_forward_window_is_active() {
    let mut app = update_bump_with_no_bump_test_app();

    let _breaker_entity = app
        .world_mut()
        .spawn((
            Breaker,
            BumpState {
                post_hit_timer: 0.0,
                last_hit_bolt:  None,
                active:         true,
                timer:          0.10,
                cooldown:       0.0,
            },
            bump_param_bundle(&BreakerDefinition::default()),
        ))
        .id();

    app.insert_resource(TestInputActive(false));
    tick(&mut app);

    let captured = app.world().resource::<CapturedNoBumps>();
    assert!(
        captured.0.is_empty(),
        "no NoBump when forward window is active (last_hit_bolt is None)"
    );
}

// ---------------------------------------------------------------------------
// Behavior 9: NoBump IS sent even when cooldown is active
// ---------------------------------------------------------------------------

#[test]
fn no_bump_sent_even_when_cooldown_is_active() {
    let mut app = update_bump_with_no_bump_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let _breaker_entity = app
        .world_mut()
        .spawn((
            Breaker,
            BumpState {
                post_hit_timer: 0.01,
                last_hit_bolt:  Some(bolt_entity),
                active:         false,
                timer:          0.0,
                cooldown:       0.10,
            },
            bump_param_bundle(&BreakerDefinition::default()),
        ))
        .id();

    app.insert_resource(TestInputActive(false));
    tick(&mut app);

    let captured = app.world().resource::<CapturedNoBumps>();
    assert_eq!(
        captured.0.len(),
        1,
        "NoBump should still be sent even when cooldown is active"
    );
}

// ---------------------------------------------------------------------------
// Behavior 10: NoBump IS sent when BoltServing present
// ---------------------------------------------------------------------------

#[test]
fn no_bump_sent_when_bolt_serving_present() {
    use crate::bolt::components::BoltServing;

    let mut app = update_bump_with_no_bump_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let _breaker_entity = app
        .world_mut()
        .spawn((
            Breaker,
            BumpState {
                post_hit_timer: 0.01,
                last_hit_bolt:  Some(bolt_entity),
                active:         false,
                timer:          0.0,
                cooldown:       0.0,
            },
            bump_param_bundle(&BreakerDefinition::default()),
        ))
        .id();

    // Spawn a serving bolt entity
    app.world_mut().spawn(BoltServing);

    app.insert_resource(TestInputActive(false));
    tick(&mut app);

    let captured = app.world().resource::<CapturedNoBumps>();
    assert_eq!(
        captured.0.len(),
        1,
        "NoBump should be sent even when BoltServing is present (serving only blocks input activation)"
    );
}
