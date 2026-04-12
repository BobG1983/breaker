//! Anchor bump multiplier tests -- retroactive grading with anchor.

use bevy::prelude::*;

use crate::{
    bolt::messages::BoltImpactBreaker,
    breaker::{
        components::{
            Breaker, BumpEarlyWindow, BumpLateWindow, BumpPerfectCooldown, BumpPerfectWindow,
            BumpState, BumpWeakCooldown, SettleDuration,
        },
        messages::BumpGrade,
        systems::bump::tests::helpers::*,
    },
    effect_v3::effects::anchor::{AnchorActive, AnchorPlanted},
};

// -- Behavior 3b: Retroactive grade uses widened perfect window via pre-seeded post_hit_timer --

#[test]
fn retroactive_grade_uses_widened_perfect_window_via_pre_seeded_timer() {
    // Pre-seed post_hit_timer to widened value 0.45 (from 3a).
    // Tick 11 frames (11 * 1/64 = 0.171875s). time_since_hit ~= 0.171875.
    // Effective perfect window = 0.30. Since 0.171875 <= 0.30, grade is Perfect.
    let mut app = combined_bump_test_app();

    app.world_mut().spawn((
        Breaker,
        AnchorPlanted,
        AnchorActive {
            bump_force_multiplier: 2.0,
            perfect_window_multiplier: 2.0,
            plant_delay: 0.3,
        },
        BumpState {
            post_hit_timer: 0.45,
            ..Default::default()
        },
        BumpPerfectWindow(0.15),
        BumpEarlyWindow(0.15),
        BumpLateWindow(0.15),
        BumpPerfectCooldown(0.0),
        BumpWeakCooldown(0.15),
        SettleDuration(0.25),
    ));

    // Tick 11 frames without bump input to drain the timer
    app.insert_resource(TestInputActive(false));
    for _ in 0..11 {
        tick(&mut app);
    }

    // Now activate bump input for the retroactive press
    app.insert_resource(TestInputActive(true));
    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1, "should emit one BumpPerformed");
    assert_eq!(
        captured.0[0].grade,
        BumpGrade::Perfect,
        "retroactive bump after 11 frames (~0.172s) with widened post_hit_timer 0.45 should be Perfect"
    );
}

#[test]
fn retroactive_grade_with_unwidened_timer_is_late() {
    // Edge case: post_hit_timer 0.30 (un-widened). Same 11 frames.
    // time_since_hit = 0.30 - remaining ~= 0.171875. But time_since_hit is computed
    // as (perfect + late) - remaining = 0.30 - (0.30 - 11*dt) = 11*dt ~= 0.171875.
    // raw perfect_window = 0.15. Since 0.171875 > 0.15, grade is Late.
    let mut app = combined_bump_test_app();

    app.world_mut().spawn((
        Breaker,
        // NO AnchorPlanted -- unwidened post_hit_timer
        BumpState {
            post_hit_timer: 0.30,
            ..Default::default()
        },
        BumpPerfectWindow(0.15),
        BumpEarlyWindow(0.15),
        BumpLateWindow(0.15),
        BumpPerfectCooldown(0.0),
        BumpWeakCooldown(0.15),
        SettleDuration(0.25),
    ));

    // Tick 11 frames without bump input
    app.insert_resource(TestInputActive(false));
    for _ in 0..11 {
        tick(&mut app);
    }

    // Now activate bump input
    app.insert_resource(TestInputActive(true));
    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1, "should emit one BumpPerformed");
    assert_eq!(
        captured.0[0].grade,
        BumpGrade::Late,
        "retroactive bump after 11 frames with un-widened post_hit_timer 0.30 should be Late"
    );
}

// -- Behavior 8: Un-plant before retroactive press still uses widened window --

#[test]
fn unplant_before_retroactive_press_still_uses_widened_window() {
    // Bolt hits while planted (post_hit_timer seeded at widened 0.45).
    // Then AnchorPlanted removed. Then 11 frames elapse. Then bump input.
    // post_hit_timer was already seeded at 0.45. The un-plant does NOT shrink it.
    // time_since_hit ~= 0.171875 <= 0.30 effective perfect window -> Perfect.
    let mut app = combined_bump_test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            AnchorPlanted,
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 2.0,
                plant_delay: 0.3,
            },
            BumpState::default(),
            BumpPerfectWindow(0.15),
            BumpEarlyWindow(0.15),
            BumpLateWindow(0.15),
            BumpPerfectCooldown(0.0),
            BumpWeakCooldown(0.15),
            SettleDuration(0.25),
        ))
        .id();

    // Step 1: Bolt hits while planted -- grade_bump sets post_hit_timer to widened value
    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));
    app.insert_resource(TestInputActive(false));
    tick(&mut app);

    // Verify post_hit_timer was seeded at widened value
    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(
        (bump.post_hit_timer - 0.45).abs() < 0.02,
        "post_hit_timer should be ~0.45 (widened) after bolt hit while planted, got {}",
        bump.post_hit_timer
    );

    // Step 2: Remove AnchorPlanted (un-plant). Clear the hit message.
    app.world_mut().entity_mut(entity).remove::<AnchorPlanted>();
    app.insert_resource(TestHitMessage(None));

    // Step 3: Tick 11 frames without bump input to drain the timer
    for _ in 0..11 {
        tick(&mut app);
    }

    // Step 4: Activate bump input for retroactive press
    app.insert_resource(TestInputActive(true));
    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(
        captured.0.len(),
        1,
        "should emit one BumpPerformed (retroactive after un-plant)"
    );
    assert_eq!(
        captured.0[0].grade,
        BumpGrade::Perfect,
        "retroactive bump after un-plant should still be Perfect (timer was seeded at widened value)"
    );
}

#[test]
fn bolt_hit_after_unplanting_uses_unwidened_timer() {
    // Edge case: If the bolt hits AFTER un-planting, post_hit_timer is 0.30 (un-widened).
    // Same 11-frame delay grades as Late.
    let mut app = combined_bump_test_app();

    app.world_mut().spawn((
        Breaker,
        // NO AnchorPlanted -- bolt hits after un-planting
        AnchorActive {
            bump_force_multiplier: 2.0,
            perfect_window_multiplier: 2.0,
            plant_delay: 0.3,
        },
        BumpState::default(),
        BumpPerfectWindow(0.15),
        BumpEarlyWindow(0.15),
        BumpLateWindow(0.15),
        BumpPerfectCooldown(0.0),
        BumpWeakCooldown(0.15),
        SettleDuration(0.25),
    ));

    // Bolt hits with no AnchorPlanted -- post_hit_timer = 0.30 (un-widened)
    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));
    app.insert_resource(TestInputActive(false));
    tick(&mut app);

    // Clear hit message
    app.insert_resource(TestHitMessage(None));

    // Tick 11 frames
    for _ in 0..11 {
        tick(&mut app);
    }

    // Activate bump input
    app.insert_resource(TestInputActive(true));
    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(
        captured.0.len(),
        1,
        "should emit one BumpPerformed (retroactive after un-plant)"
    );
    assert_eq!(
        captured.0[0].grade,
        BumpGrade::Late,
        "bolt hit after un-planting should use un-widened timer, grading Late after 11 frames"
    );
}
