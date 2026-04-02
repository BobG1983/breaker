use super::helpers::*;
use crate::breaker::{
    components::{Breaker, BumpState},
    definition::BreakerDefinition,
    messages::BumpGrade,
};

#[test]
fn retroactive_perfect_grades_and_sets_zero_cooldown() {
    let mut app = update_bump_test_app();
    let config = BreakerDefinition::default();

    // post_hit_timer at max — just hit, pressing immediately
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BumpState {
                post_hit_timer: config.perfect_window + config.late_window,
                ..Default::default()
            },
            bump_param_bundle(&config),
        ))
        .id();

    app.insert_resource(TestInputActive(true));
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(
        (bump.cooldown - config.perfect_bump_cooldown).abs() < f32::EPSILON,
        "perfect retroactive should set perfect_bump_cooldown ({}), got {}",
        config.perfect_bump_cooldown,
        bump.cooldown
    );
    assert!(bump.post_hit_timer <= 0.0, "should clear post_hit_timer");

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1);
    assert_eq!(captured.0[0].grade, BumpGrade::Perfect);
}

#[test]
fn retroactive_late_grades_correctly() {
    let mut app = update_bump_test_app();
    let config = BreakerDefinition::default();

    // post_hit_timer low — hit happened a while ago, pressing late
    let remaining = config.late_window * 0.5; // some time left but past perfect
    app.world_mut().spawn((
        Breaker,
        BumpState {
            post_hit_timer: remaining,
            ..Default::default()
        },
        bump_param_bundle(&config),
    ));

    app.insert_resource(TestInputActive(true));
    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1);
    assert_eq!(captured.0[0].grade, BumpGrade::Late);
}

#[test]
fn update_bump_retroactive_uses_last_hit_bolt() {
    // Given: BumpState.last_hit_bolt is set to a specific entity, post_hit_timer is active
    // When: update_bump runs with Bump input (retroactive path)
    // Then: BumpPerformed.bolt matches last_hit_bolt
    let mut app = update_bump_test_app();
    let config = BreakerDefinition::default();

    let bolt_entity = app.world_mut().spawn_empty().id();

    app.world_mut().spawn((
        Breaker,
        BumpState {
            post_hit_timer: config.perfect_window + config.late_window,
            last_hit_bolt: Some(bolt_entity),
            ..Default::default()
        },
        bump_param_bundle(&config),
    ));

    app.insert_resource(TestInputActive(true));
    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1, "should emit one BumpPerformed");
    assert_eq!(
        captured.0[0].bolt,
        Some(bolt_entity),
        "BumpPerformed.bolt in retroactive path should match BumpState.last_hit_bolt"
    );
}
