use bevy::prelude::*;

use super::{super::data::*, helpers::*};
use crate::{
    breaker::components::{
        Breaker, BumpEarlyWindow, BumpLateWindow, BumpPerfectCooldown, BumpPerfectWindow,
        BumpState, BumpWeakCooldown,
    },
    effect_v3::effects::anchor::{AnchorActive, AnchorPlanted},
};

// ── Part E: BreakerBumpTimingData (mutable) ─────────────────────

// Behavior 9: BreakerBumpTimingData all bump timing fields
#[test]
fn breaker_bump_timing_data_all_fields_accessible() {
    let mut app = test_app();
    app.world_mut().spawn((
        Breaker,
        BumpState::default(),
        BumpPerfectWindow(0.05),
        BumpEarlyWindow(0.15),
        BumpLateWindow(0.1),
        BumpPerfectCooldown(0.5),
        BumpWeakCooldown(0.2),
    ));

    app.add_systems(
        FixedUpdate,
        |query: Query<BreakerBumpTimingDataReadOnly, With<Breaker>>,
         mut matched: ResMut<QueryMatched>| {
            for data in &query {
                matched.0 = true;
                assert!((data.perfect_window.0 - 0.05).abs() < f32::EPSILON);
                assert!((data.early_window.0 - 0.15).abs() < f32::EPSILON);
                assert!((data.late_window.0 - 0.1).abs() < f32::EPSILON);
                assert!((data.perfect_cooldown.0 - 0.5).abs() < f32::EPSILON);
                assert!((data.weak_cooldown.0 - 0.2).abs() < f32::EPSILON);
                assert!(data.anchor_planted.is_none());
                assert!(data.anchor_active.is_none());
            }
        },
    );
    tick(&mut app);
    assert_query_matched(&app);
}

// Behavior 9 edge case: anchors present
#[test]
fn breaker_bump_timing_data_with_anchors_present() {
    let mut app = test_app();
    app.world_mut().spawn((
        Breaker,
        BumpState::default(),
        BumpPerfectWindow(0.05),
        BumpEarlyWindow(0.15),
        BumpLateWindow(0.1),
        BumpPerfectCooldown(0.5),
        BumpWeakCooldown(0.2),
        AnchorPlanted,
        AnchorActive {
            bump_force_multiplier:     1.5,
            perfect_window_multiplier: 2.0,
            plant_delay:               0.5,
        },
    ));

    app.add_systems(
        FixedUpdate,
        |query: Query<BreakerBumpTimingDataReadOnly, With<Breaker>>,
         mut matched: ResMut<QueryMatched>| {
            for data in &query {
                matched.0 = true;
                assert!(
                    data.anchor_planted.is_some(),
                    "AnchorPlanted should be Some"
                );
                assert!(data.anchor_active.is_some(), "AnchorActive should be Some");
            }
        },
    );
    tick(&mut app);
    assert_query_matched(&app);
}

// Behavior 10: BumpState mutation through BreakerBumpTimingData
#[test]
fn breaker_bump_timing_data_bump_state_mutation() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BumpState::default(),
            BumpPerfectWindow(0.05),
            BumpEarlyWindow(0.15),
            BumpLateWindow(0.1),
            BumpPerfectCooldown(0.5),
            BumpWeakCooldown(0.2),
        ))
        .id();

    app.add_systems(
        FixedUpdate,
        |mut query: Query<BreakerBumpTimingData, With<Breaker>>| {
            for mut data in &mut query {
                data.bump.active = true;
            }
        },
    );
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(
        bump.active,
        "BumpState.active should be true after mutation"
    );
}

// ── Part F: BreakerBumpGradingData (mutable) ────────────────────

// Behavior 11: BreakerBumpGradingData fields (no early_window)
#[test]
fn breaker_bump_grading_data_fields_accessible_no_early_window() {
    let mut app = test_app();
    app.world_mut().spawn((
        Breaker,
        BumpState::default(),
        BumpPerfectWindow(0.05),
        BumpLateWindow(0.1),
        BumpPerfectCooldown(0.5),
        BumpWeakCooldown(0.2),
    ));

    app.add_systems(
        FixedUpdate,
        |query: Query<BreakerBumpGradingDataReadOnly, With<Breaker>>,
         mut matched: ResMut<QueryMatched>| {
            for data in &query {
                matched.0 = true;
                assert!((data.perfect_window.0 - 0.05).abs() < f32::EPSILON);
                assert!((data.late_window.0 - 0.1).abs() < f32::EPSILON);
                assert!((data.perfect_cooldown.0 - 0.5).abs() < f32::EPSILON);
                assert!((data.weak_cooldown.0 - 0.2).abs() < f32::EPSILON);
                assert!(data.anchor_planted.is_none());
                assert!(data.anchor_active.is_none());
            }
        },
    );
    tick(&mut app);
    assert_query_matched(&app);
}

// Behavior 12: BumpState mutation through BreakerBumpGradingData
#[test]
fn breaker_bump_grading_data_bump_state_mutation() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BumpState::default(),
            BumpPerfectWindow(0.05),
            BumpLateWindow(0.1),
            BumpPerfectCooldown(0.5),
            BumpWeakCooldown(0.2),
        ))
        .id();

    app.add_systems(
        FixedUpdate,
        |mut query: Query<BreakerBumpGradingData, With<Breaker>>| {
            for mut data in &mut query {
                data.bump.cooldown = 0.5;
            }
        },
    );
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(
        (bump.cooldown - 0.5).abs() < f32::EPSILON,
        "BumpState.cooldown should be 0.5 after mutation"
    );
}

// Behavior 13: BreakerBumpGradingData with anchors present
#[test]
fn breaker_bump_grading_data_with_anchors() {
    let mut app = test_app();
    app.world_mut().spawn((
        Breaker,
        BumpState::default(),
        BumpPerfectWindow(0.05),
        BumpLateWindow(0.1),
        BumpPerfectCooldown(0.5),
        BumpWeakCooldown(0.2),
        AnchorPlanted,
        AnchorActive {
            bump_force_multiplier:     1.5,
            perfect_window_multiplier: 2.0,
            plant_delay:               0.5,
        },
    ));

    app.add_systems(
        FixedUpdate,
        |query: Query<BreakerBumpGradingDataReadOnly, With<Breaker>>,
         mut matched: ResMut<QueryMatched>| {
            for data in &query {
                matched.0 = true;
                assert!(
                    data.anchor_planted.is_some(),
                    "AnchorPlanted should be Some"
                );
                assert!(data.anchor_active.is_some(), "AnchorActive should be Some");
            }
        },
    );
    tick(&mut app);
    assert_query_matched(&app);
}
