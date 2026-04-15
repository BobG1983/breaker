use bevy::prelude::*;

use super::{super::system::on_bumped, helpers::*};
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
// Behavior 14: on_bumped walks breaker entity on any grade
// ====================================================================

#[test]
fn on_bumped_walks_breaker_on_perfect_grade() {
    let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::Bumped,
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
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
        .expect("breaker should have EffectStack");
    assert_eq!(stack.len(), 1);
}

#[test]
fn on_bumped_walks_breaker_on_early_grade() {
    let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::Bumped,
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
        .expect("breaker should have EffectStack for Early grade");
    assert_eq!(stack.len(), 1);
}

// ====================================================================
// Behavior 15: on_bumped walks bolt entity when bolt is Some
// ====================================================================

#[test]
fn on_bumped_walks_bolt_entity_when_bolt_is_some() {
    let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

    let bolt_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::Bumped,
            2.0,
        )]))
        .id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Late,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
        .expect("bolt should have EffectStack");
    assert_eq!(stack.len(), 1);
}

#[test]
fn on_bumped_skips_bolt_without_bound_effects() {
    let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::Bumped,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    // Bolt should be silently skipped (no BoundEffects), no panic
    let bolt_stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_entity);
    assert!(
        bolt_stack.is_none(),
        "bolt without BoundEffects should not have EffectStack"
    );

    // Breaker still gets the effect
    let breaker_stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
        .expect("breaker should still get EffectStack");
    assert_eq!(breaker_stack.len(), 1);
}

// ====================================================================
// Behavior 16: on_bumped skips bolt when bolt is None
// ====================================================================

#[test]
fn on_bumped_skips_bolt_when_none() {
    let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::Bumped,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    None,
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
        .expect("breaker should have EffectStack despite None bolt");
    assert_eq!(stack.len(), 1);
}

// ====================================================================
// Behavior 17: on_bumped provides TriggerContext::Bump with participants
// ====================================================================

#[test]
fn on_bumped_provides_bump_context_on_resolves_to_bolt() {
    let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![on_target_tree(
            "chip_a",
            Trigger::Bumped,
            BumpTarget::Bolt,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let bolt_stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
        .expect("bolt_entity should gain EffectStack via On(Bump(Bolt))");
    assert_eq!(bolt_stack.len(), 1);

    let breaker_stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity);
    assert!(
        breaker_stack.is_none(),
        "breaker should NOT have EffectStack — effect redirected to bolt"
    );
}

#[test]
fn on_bumped_on_bump_breaker_resolves_to_breaker() {
    let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

    let bolt_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![on_target_tree(
            "chip_a",
            Trigger::Bumped,
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
}

// ====================================================================
// Behavior 18: on_bumped is no-op without messages
// ====================================================================

#[test]
fn on_bumped_is_noop_without_messages() {
    let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::Bumped,
            1.5,
        )]))
        .id();

    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(stack.is_none(), "no EffectStack without messages");
}

// ====================================================================
// Behavior 19: on_bumped does not walk third-party entities (local scope)
// ====================================================================

#[test]
fn on_bumped_does_not_walk_bystander_entities() {
    let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::Bumped,
            1.5,
        )]))
        .id();

    // Bystander Entity C — has BoundEffects but is not bolt or breaker
    let bystander = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_c",
            Trigger::Bumped,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    // Breaker gets the effect (is a participant)
    let breaker_stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
        .expect("breaker should have EffectStack");
    assert_eq!(breaker_stack.len(), 1);

    // Bolt has no BoundEffects, so no effect
    let bolt_stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_entity);
    assert!(bolt_stack.is_none(), "bolt without BoundEffects skipped");

    // Bystander should NOT be walked — local dispatch only
    let bystander_stack = app.world().get::<EffectStack<SpeedBoostConfig>>(bystander);
    assert!(
        bystander_stack.is_none(),
        "bystander entity should NOT be walked by local dispatch"
    );
}

// ====================================================================
// Behavior 20: on_bumped handles multiple messages in one frame
// ====================================================================

#[test]
fn on_bumped_handles_multiple_messages_per_frame() {
    let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

    let bolt_a = app.world_mut().spawn_empty().id();
    let bolt_b = app.world_mut().spawn_empty().id();
    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::Bumped,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![
        BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    Some(bolt_a),
            breaker: breaker_entity,
        },
        BumpPerformed {
            grade:   BumpGrade::Late,
            bolt:    Some(bolt_b),
            breaker: breaker_entity,
        },
    ]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
        .expect("breaker should have EffectStack from two messages");
    assert_eq!(
        stack.len(),
        2,
        "each BumpPerformed message should trigger a separate walk"
    );
}
