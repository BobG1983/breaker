//! Anchor bump multiplier tests -- forward grading with `AnchorPlanted`.

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
    effect::{AnchorActive, AnchorPlanted},
};

// -- Behavior 1: Forward grade uses widened perfect window when planted --

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

// -- Behavior 2: Forward grade at widened boundary is Perfect --

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

// -- Behavior 3a: grade_bump sets widened post_hit_timer when planted --

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

// -- Behavior 6: AnchorActive present but AnchorPlanted absent means no widening --

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

// -- Behavior 7: Neither AnchorActive nor AnchorPlanted means unchanged default behavior --

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
