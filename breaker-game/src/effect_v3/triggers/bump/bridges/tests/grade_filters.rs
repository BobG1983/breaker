use bevy::prelude::*;

use super::{
    super::system::{on_early_bumped, on_late_bumped, on_perfect_bumped},
    helpers::*,
};
use crate::{
    breaker::messages::{BumpGrade, BumpPerformed},
    effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        storage::BoundEffects,
        types::{BumpTarget, Trigger},
    },
};

// ====================================================================
// Behavior 21: on_perfect_bumped fires only on Perfect
// ====================================================================

#[test]
fn on_perfect_bumped_fires_on_perfect_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_perfect_bumped),
        on_perfect_bumped,
    ));

    let bolt_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_b",
            Trigger::PerfectBumped,
            1.5,
        )]))
        .id();
    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::PerfectBumped,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let breaker_stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
        .expect("breaker should have EffectStack");
    assert_eq!(breaker_stack.len(), 1);

    // Bolt also gets walked (local dispatch)
    let bolt_stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
        .expect("bolt should also gain EffectStack from local dispatch");
    assert_eq!(bolt_stack.len(), 1);
}

// ====================================================================
// Behavior 22: on_perfect_bumped filters out Early
// ====================================================================

#[test]
fn on_perfect_bumped_filters_out_early_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_perfect_bumped),
        on_perfect_bumped,
    ));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::PerfectBumped,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Early,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity);
    assert!(
        stack.is_none(),
        "PerfectBumped should not fire on Early grade"
    );
}

// ====================================================================
// Behavior 23: on_perfect_bumped filters out Late
// ====================================================================

#[test]
fn on_perfect_bumped_filters_out_late_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_perfect_bumped),
        on_perfect_bumped,
    ));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::PerfectBumped,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Late,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity);
    assert!(
        stack.is_none(),
        "PerfectBumped should not fire on Late grade"
    );
}

// ====================================================================
// Behavior 24: on_perfect_bumped provides TriggerContext::Bump
// ====================================================================

#[test]
fn on_perfect_bumped_provides_bump_context_on_resolves_to_breaker() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_perfect_bumped),
        on_perfect_bumped,
    ));

    let bolt_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![on_target_tree(
            "chip_a",
            Trigger::PerfectBumped,
            BumpTarget::Breaker,
            1.5,
        )]))
        .id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let breaker_stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
        .expect("breaker_entity should gain EffectStack via On(Bump(Breaker))");
    assert_eq!(breaker_stack.len(), 1);

    let bolt_stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_entity);
    assert!(
        bolt_stack.is_none(),
        "bolt should NOT have EffectStack — effect redirected to breaker"
    );
}

// ====================================================================
// Behavior 25: on_early_bumped fires only on Early
// ====================================================================

#[test]
fn on_early_bumped_fires_on_early_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_early_bumped),
        on_early_bumped,
    ));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::EarlyBumped,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Early,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
        .expect("breaker should have EffectStack for Early");
    assert_eq!(stack.len(), 1);
}

// ====================================================================
// Behavior 26: on_early_bumped filters out Perfect
// ====================================================================

#[test]
fn on_early_bumped_filters_out_perfect_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_early_bumped),
        on_early_bumped,
    ));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::EarlyBumped,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity);
    assert!(
        stack.is_none(),
        "EarlyBumped should not fire on Perfect grade"
    );
}

// ====================================================================
// Behavior 27: on_early_bumped filters out Late
// ====================================================================

#[test]
fn on_early_bumped_filters_out_late_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_early_bumped),
        on_early_bumped,
    ));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::EarlyBumped,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Late,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity);
    assert!(stack.is_none(), "EarlyBumped should not fire on Late grade");
}

// ====================================================================
// Behavior 28: on_late_bumped fires only on Late
// ====================================================================

#[test]
fn on_late_bumped_fires_on_late_grade() {
    let mut app =
        bump_performed_test_app((inject_bump_performed.before(on_late_bumped), on_late_bumped));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::LateBumped,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Late,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
        .expect("breaker should have EffectStack for Late");
    assert_eq!(stack.len(), 1);
}

// ====================================================================
// Behavior 29: on_late_bumped filters out Perfect
// ====================================================================

#[test]
fn on_late_bumped_filters_out_perfect_grade() {
    let mut app =
        bump_performed_test_app((inject_bump_performed.before(on_late_bumped), on_late_bumped));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::LateBumped,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity);
    assert!(
        stack.is_none(),
        "LateBumped should not fire on Perfect grade"
    );
}

// ====================================================================
// Behavior 30: on_late_bumped filters out Early
// ====================================================================

#[test]
fn on_late_bumped_filters_out_early_grade() {
    let mut app =
        bump_performed_test_app((inject_bump_performed.before(on_late_bumped), on_late_bumped));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::LateBumped,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Early,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity);
    assert!(stack.is_none(), "LateBumped should not fire on Early grade");
}
