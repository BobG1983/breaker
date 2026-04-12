//! Anchor bump multiplier tests -- forward window and retroactive timer durations.

use bevy::prelude::*;

use crate::{
    bolt::messages::BoltImpactBreaker,
    breaker::{
        components::{
            Breaker, BumpEarlyWindow, BumpLateWindow, BumpPerfectCooldown, BumpPerfectWindow,
            BumpState, BumpWeakCooldown, SettleDuration,
        },
        systems::bump::tests::helpers::*,
    },
    effect_v3::effects::anchor::{AnchorActive, AnchorPlanted},
};

// -- Behavior 4: Forward window total duration includes widened perfect window --

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
                bump_force_multiplier:     2.0,
                perfect_window_multiplier: 2.0,
                plant_delay:               0.3,
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
                bump_force_multiplier:     2.0,
                perfect_window_multiplier: 1.0,
                plant_delay:               0.3,
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

// -- Behavior 5: Retroactive post_hit_timer includes widened perfect window --
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
                bump_force_multiplier:     2.0,
                perfect_window_multiplier: 2.0,
                plant_delay:               0.3,
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
        bolt:    Entity::PLACEHOLDER,
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
        bolt:    Entity::PLACEHOLDER,
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
