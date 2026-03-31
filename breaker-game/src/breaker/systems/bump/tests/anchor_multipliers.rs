//! Anchor bump multiplier tests -- perfect window widening when `AnchorPlanted` is present.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::messages::BoltImpactBreaker,
    breaker::{
        components::{
            Breaker, BumpEarlyWindow, BumpLateWindow, BumpPerfectCooldown, BumpPerfectWindow,
            BumpState, BumpWeakCooldown, SettleDuration,
        },
        messages::BumpGrade,
    },
    effect::{AnchorActive, AnchorPlanted},
};

// ── Behavior 1: Forward grade uses widened perfect window when planted ──

#[test]
fn forward_grade_uses_widened_perfect_window_when_planted() {
    // Given: AnchorPlanted + AnchorActive with perfect_window_multiplier 2.0,
    // BumpPerfectWindow(0.15). Forward bump active with timer 0.25.
    // Effective perfect window = 0.15 * 2.0 = 0.30. Since 0.25 <= 0.30, grade is Perfect.
    let mut app = grade_bump_test_app();

    app.world_mut().spawn((
        Breaker,
        AnchorPlanted,
        AnchorActive {
            bump_force_multiplier: 2.0,
            perfect_window_multiplier: 2.0,
            plant_delay: 0.3,
        },
        BumpState {
            active: true,
            timer: 0.25,
            ..Default::default()
        },
        BumpPerfectWindow(0.15),
        BumpEarlyWindow(0.15),
        BumpLateWindow(0.15),
        BumpPerfectCooldown(0.0),
        BumpWeakCooldown(0.15),
        SettleDuration(0.25),
    ));

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));
    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1, "should emit one BumpPerformed");
    assert_eq!(
        captured.0[0].grade,
        BumpGrade::Perfect,
        "timer 0.25 should be Perfect with widened window 0.30 (0.15 * 2.0)"
    );
}

#[test]
fn forward_grade_without_anchor_planted_does_not_widen() {
    // Edge case: AnchorActive present but AnchorPlanted absent -- same timer 0.25 is Early
    // because raw perfect window is 0.15 and 0.25 > 0.15.
    let mut app = grade_bump_test_app();

    app.world_mut().spawn((
        Breaker,
        // NO AnchorPlanted
        AnchorActive {
            bump_force_multiplier: 2.0,
            perfect_window_multiplier: 2.0,
            plant_delay: 0.3,
        },
        BumpState {
            active: true,
            timer: 0.25,
            ..Default::default()
        },
        BumpPerfectWindow(0.15),
        BumpEarlyWindow(0.15),
        BumpLateWindow(0.15),
        BumpPerfectCooldown(0.0),
        BumpWeakCooldown(0.15),
        SettleDuration(0.25),
    ));

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));
    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1, "should emit one BumpPerformed");
    assert_eq!(
        captured.0[0].grade,
        BumpGrade::Early,
        "timer 0.25 should be Early without AnchorPlanted (raw window 0.15)"
    );
}

// ── Behavior 2: Forward grade at widened boundary is Perfect ──

#[test]
fn forward_grade_at_widened_boundary_is_perfect() {
    // Timer exactly at widened boundary: timer 0.30 == effective perfect window 0.30.
    // Boundary is inclusive, so grade is Perfect.
    let mut app = grade_bump_test_app();

    app.world_mut().spawn((
        Breaker,
        AnchorPlanted,
        AnchorActive {
            bump_force_multiplier: 1.5,
            perfect_window_multiplier: 2.0,
            plant_delay: 0.3,
        },
        BumpState {
            active: true,
            timer: 0.30,
            ..Default::default()
        },
        BumpPerfectWindow(0.15),
        BumpEarlyWindow(0.15),
        BumpLateWindow(0.15),
        BumpPerfectCooldown(0.0),
        BumpWeakCooldown(0.15),
        SettleDuration(0.25),
    ));

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));
    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1, "should emit one BumpPerformed");
    assert_eq!(
        captured.0[0].grade,
        BumpGrade::Perfect,
        "timer 0.30 at widened boundary should be Perfect (inclusive)"
    );
}

#[test]
fn forward_grade_just_above_widened_boundary_is_early() {
    // Edge case: timer 0.301 is just above the widened boundary 0.30. Grade is Early.
    let mut app = grade_bump_test_app();

    app.world_mut().spawn((
        Breaker,
        AnchorPlanted,
        AnchorActive {
            bump_force_multiplier: 1.5,
            perfect_window_multiplier: 2.0,
            plant_delay: 0.3,
        },
        BumpState {
            active: true,
            timer: 0.301,
            ..Default::default()
        },
        BumpPerfectWindow(0.15),
        BumpEarlyWindow(0.15),
        BumpLateWindow(0.15),
        BumpPerfectCooldown(0.0),
        BumpWeakCooldown(0.15),
        SettleDuration(0.25),
    ));

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));
    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1, "should emit one BumpPerformed");
    assert_eq!(
        captured.0[0].grade,
        BumpGrade::Early,
        "timer 0.301 just above widened boundary 0.30 should be Early"
    );
}

// ── Behavior 3a: grade_bump sets widened post_hit_timer when planted ──

#[test]
fn grade_bump_sets_widened_post_hit_timer_when_planted() {
    // No active forward bump. Bolt hits while planted.
    // post_hit_timer = (perfect_window * multiplier) + late_window
    //                = (0.15 * 2.0) + 0.15 = 0.45
    let mut app = grade_bump_test_app();

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

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    let expected = 0.45; // (0.15 * 2.0) + 0.15
    assert!(
        (bump.post_hit_timer - expected).abs() < f32::EPSILON,
        "post_hit_timer should be {expected} when planted (widened), got {}",
        bump.post_hit_timer
    );
}

#[test]
fn grade_bump_sets_unwidened_post_hit_timer_without_anchor_planted() {
    // Edge case: Without AnchorPlanted, post_hit_timer = perfect_window + late_window = 0.30
    let mut app = grade_bump_test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            // NO AnchorPlanted
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

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    let expected = 0.30; // 0.15 + 0.15 (no widening)
    assert!(
        (bump.post_hit_timer - expected).abs() < f32::EPSILON,
        "post_hit_timer should be {expected} without AnchorPlanted, got {}",
        bump.post_hit_timer
    );
}

// ── Behavior 3b: Retroactive grade uses widened perfect window via pre-seeded post_hit_timer ──

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

// ── Behavior 4: Forward window total duration includes widened perfect window ──

#[test]
fn forward_window_duration_includes_widened_perfect_window() {
    // When planted, forward window timer = early_window + (perfect_window * multiplier)
    //                                    = 0.15 + (0.15 * 2.0) = 0.45
    // After one tick, timer ~= 0.45 - dt
    let mut app = update_bump_test_app();

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

    app.insert_resource(TestInputActive(true));
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(bump.active, "forward window should open");

    // Timer should be approximately 0.45 - dt (one tick elapsed)
    let expected = 0.45; // early_window(0.15) + perfect_window * multiplier(0.30)
    assert!(
        (bump.timer - expected).abs() < 0.02,
        "forward window timer should be near {expected} (widened), got {}",
        bump.timer
    );
}

#[test]
fn forward_window_duration_with_multiplier_one_is_unchanged() {
    // Edge case: perfect_window_multiplier = 1.0, so timer = early + perfect = 0.30
    let mut app = update_bump_test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            AnchorPlanted,
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.0,
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

    app.insert_resource(TestInputActive(true));
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(bump.active, "forward window should open");

    let expected = 0.30; // early_window(0.15) + perfect_window * 1.0(0.15)
    assert!(
        (bump.timer - expected).abs() < 0.02,
        "forward window timer with multiplier 1.0 should be near {expected}, got {}",
        bump.timer
    );
}

// ── Behavior 5: Retroactive post_hit_timer includes widened perfect window ──
// (Same as behavior 3a -- tested above. Included for spec coverage completeness.)

#[test]
fn retroactive_post_hit_timer_includes_widened_perfect_window() {
    // Identical to behavior 3a -- confirming the retroactive window path.
    // post_hit_timer = (perfect_window * multiplier) + late_window = 0.45
    let mut app = grade_bump_test_app();

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

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(
        (bump.post_hit_timer - 0.45).abs() < f32::EPSILON,
        "retroactive post_hit_timer should be 0.45 when planted, got {}",
        bump.post_hit_timer
    );
}

#[test]
fn retroactive_post_hit_timer_unwidened_without_anchor_planted() {
    // Edge case: no AnchorPlanted, post_hit_timer = 0.15 + 0.15 = 0.30
    let mut app = grade_bump_test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BumpState::default(),
            BumpPerfectWindow(0.15),
            BumpEarlyWindow(0.15),
            BumpLateWindow(0.15),
            BumpPerfectCooldown(0.0),
            BumpWeakCooldown(0.15),
            SettleDuration(0.25),
        ))
        .id();

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(
        (bump.post_hit_timer - 0.30).abs() < f32::EPSILON,
        "retroactive post_hit_timer should be 0.30 without AnchorPlanted, got {}",
        bump.post_hit_timer
    );
}

// ── Behavior 6: AnchorActive present but AnchorPlanted absent means no widening ──

#[test]
fn anchor_active_without_planted_does_not_widen_forward_grade() {
    // AnchorActive present, AnchorPlanted absent. Timer 0.20 > raw perfect 0.15. Grade: Early.
    let mut app = grade_bump_test_app();

    app.world_mut().spawn((
        Breaker,
        AnchorActive {
            bump_force_multiplier: 2.0,
            perfect_window_multiplier: 2.0,
            plant_delay: 0.3,
        },
        // NO AnchorPlanted
        BumpState {
            active: true,
            timer: 0.20,
            ..Default::default()
        },
        BumpPerfectWindow(0.15),
        BumpEarlyWindow(0.15),
        BumpLateWindow(0.15),
        BumpPerfectCooldown(0.0),
        BumpWeakCooldown(0.15),
        SettleDuration(0.25),
    ));

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));
    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1);
    assert_eq!(
        captured.0[0].grade,
        BumpGrade::Early,
        "timer 0.20 without AnchorPlanted should grade Early (raw window 0.15)"
    );
}

#[test]
fn anchor_active_without_planted_within_raw_window_is_perfect() {
    // Edge case: timer 0.10 <= raw perfect 0.15. Grade: Perfect (normal behavior confirmed).
    let mut app = grade_bump_test_app();

    app.world_mut().spawn((
        Breaker,
        AnchorActive {
            bump_force_multiplier: 2.0,
            perfect_window_multiplier: 2.0,
            plant_delay: 0.3,
        },
        BumpState {
            active: true,
            timer: 0.10,
            ..Default::default()
        },
        BumpPerfectWindow(0.15),
        BumpEarlyWindow(0.15),
        BumpLateWindow(0.15),
        BumpPerfectCooldown(0.0),
        BumpWeakCooldown(0.15),
        SettleDuration(0.25),
    ));

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));
    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1);
    assert_eq!(
        captured.0[0].grade,
        BumpGrade::Perfect,
        "timer 0.10 within raw window 0.15 should be Perfect regardless of AnchorActive"
    );
}

// ── Behavior 7: Neither AnchorActive nor AnchorPlanted means unchanged default behavior ──

#[test]
fn no_anchor_components_unchanged_default_behavior() {
    // Neither AnchorActive nor AnchorPlanted. Timer 0.10 <= raw perfect 0.15. Grade: Perfect.
    let mut app = grade_bump_test_app();

    app.world_mut().spawn((
        Breaker,
        BumpState {
            active: true,
            timer: 0.10,
            ..Default::default()
        },
        BumpPerfectWindow(0.15),
        BumpEarlyWindow(0.15),
        BumpLateWindow(0.15),
        BumpPerfectCooldown(0.0),
        BumpWeakCooldown(0.15),
        SettleDuration(0.25),
    ));

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));
    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1);
    assert_eq!(
        captured.0[0].grade,
        BumpGrade::Perfect,
        "timer 0.10 with no anchor components should be Perfect (raw window 0.15)"
    );
}

// ── Behavior 8: Un-plant before retroactive press still uses widened window ──

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
