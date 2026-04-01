use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::messages::BoltImpactBreaker,
    breaker::{
        components::{Breaker, BumpState, DashState, DashStateTimer, SettleDuration},
        messages::{BumpGrade, BumpPerformed},
        resources::BreakerConfig,
    },
};

#[test]
fn same_frame_hit_and_expiry_grades_not_whiffs() {
    let mut app = combined_bump_test_app();
    let config = app.world().resource::<BreakerConfig>().clone();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BumpState {
                active: true,
                timer: 0.001, // about to expire this tick
                ..Default::default()
            },
            bump_param_bundle(&config),
        ))
        .id();

    // Bolt hits the same frame the window would expire
    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(!bump.active, "should deactivate");

    // Should be graded as a forward bump (perfect — timer near 0 is within perfect_window)
    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(
        captured.0.len(),
        1,
        "should grade the hit, not whiff — got {} bumps",
        captured.0.len()
    );
    assert_eq!(captured.0[0].grade, BumpGrade::Perfect);

    // Should NOT whiff
    let whiffs = app.world().resource::<CapturedWhiffs>();
    assert_eq!(whiffs.0, 0, "should not whiff when hit arrives same frame");

    // Cooldown should match grade, not whiff
    assert!(
        (bump.cooldown - config.perfect_bump_cooldown).abs() < f32::EPSILON,
        "cooldown should be perfect_bump_cooldown ({}), got {}",
        config.perfect_bump_cooldown,
        bump.cooldown
    );
}

// ── perfect_bump_dash_cancel tests ───────────────────────────────

#[test]
fn perfect_bump_cancels_dash() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<BreakerConfig>()
        .add_message::<BumpPerformed>();
    let config = app.world().resource::<BreakerConfig>().clone();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            DashState::Dashing,
            DashStateTimer { remaining: 0.1 },
            SettleDuration(config.settle_duration),
        ))
        .id();

    app.insert_resource(TestBumpMessage(Some(BumpPerformed {
        grade: BumpGrade::Perfect,
        bolt: None,
    })));

    app.add_systems(
        FixedUpdate,
        (
            enqueue_bump.before(super::super::perfect_bump_dash_cancel),
            super::super::perfect_bump_dash_cancel,
        ),
    );
    tick(&mut app);

    let state = app.world().get::<DashState>(entity).unwrap();
    assert_eq!(
        *state,
        DashState::Settling,
        "perfect bump during dash should transition to settling"
    );

    let timer = app.world().get::<DashStateTimer>(entity).unwrap();
    assert!(
        (timer.remaining - config.settle_duration).abs() < f32::EPSILON,
        "settle timer should be set to config.settle_duration"
    );
}
