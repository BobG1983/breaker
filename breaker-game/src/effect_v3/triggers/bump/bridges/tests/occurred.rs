use bevy::prelude::*;

use super::{
    super::system::{
        on_bump_occurred, on_early_bump_occurred, on_late_bump_occurred, on_perfect_bump_occurred,
    },
    helpers::*,
};
use crate::{
    breaker::messages::{BumpGrade, BumpPerformed, BumpWhiffed},
    effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        storage::BoundEffects,
        types::{BumpTarget, Trigger},
    },
};

// ====================================================================
// Behavior 31: on_bump_occurred walks ALL entities on any grade
// ====================================================================

#[test]
fn on_bump_occurred_walks_all_entities_with_bound_effects_any_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_bump_occurred),
        on_bump_occurred,
    ));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let entity_a = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::BumpOccurred,
            1.5,
        )]))
        .id();

    let entity_b = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_b",
            Trigger::BumpOccurred,
            2.0,
        )]))
        .id();

    // entity_c has no BoundEffects
    let entity_c = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Early,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack_a = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity_a)
        .expect("entity_a should have EffectStack");
    assert_eq!(stack_a.len(), 1);

    let stack_b = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity_b)
        .expect("entity_b should have EffectStack");
    assert_eq!(stack_b.len(), 1);

    let stack_c = app.world().get::<EffectStack<SpeedBoostConfig>>(entity_c);
    assert!(stack_c.is_none(), "entity_c without BoundEffects: no stack");
}

// ====================================================================
// Behavior 32: on_bump_occurred provides TriggerContext::Bump
// ====================================================================

#[test]
fn on_bump_occurred_provides_bump_context_on_resolves_to_bolt() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_bump_occurred),
        on_bump_occurred,
    ));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let owner = app
        .world_mut()
        .spawn(BoundEffects(vec![on_target_tree(
            "chip_a",
            Trigger::BumpOccurred,
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

    let owner_stack = app.world().get::<EffectStack<SpeedBoostConfig>>(owner);
    assert!(
        owner_stack.is_none(),
        "owner should NOT have EffectStack — redirected to bolt"
    );
}

#[test]
fn on_bump_occurred_on_bump_breaker_resolves_to_breaker() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_bump_occurred),
        on_bump_occurred,
    ));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let _owner = app
        .world_mut()
        .spawn(BoundEffects(vec![on_target_tree(
            "chip_a",
            Trigger::BumpOccurred,
            BumpTarget::Breaker,
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
        .expect("breaker_entity should gain EffectStack via On(Bump(Breaker))");
    assert_eq!(breaker_stack.len(), 1);
}

// ====================================================================
// Behavior 33: on_bump_occurred is no-op without messages
// ====================================================================

#[test]
fn on_bump_occurred_is_noop_without_messages() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_bump_occurred),
        on_bump_occurred,
    ));

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::BumpOccurred,
            1.5,
        )]))
        .id();

    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(stack.is_none(), "no EffectStack without messages");
}

// ====================================================================
// Behavior 34: on_bump_occurred handles multiple messages in one frame
// ====================================================================

#[test]
fn on_bump_occurred_handles_multiple_messages_per_frame() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_bump_occurred),
        on_bump_occurred,
    ));

    let bolt_a = app.world_mut().spawn_empty().id();
    let breaker_a = app.world_mut().spawn_empty().id();
    let bolt_b = app.world_mut().spawn_empty().id();
    let breaker_b = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::BumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![
        BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    Some(bolt_a),
            breaker: breaker_a,
        },
        BumpPerformed {
            grade:   BumpGrade::Late,
            bolt:    Some(bolt_b),
            breaker: breaker_b,
        },
    ]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist after two messages");
    assert_eq!(stack.len(), 2);
}

// ====================================================================
// Behavior 35: on_perfect_bump_occurred fires on all entities for Perfect
// ====================================================================

#[test]
fn on_perfect_bump_occurred_fires_on_all_entities_for_perfect_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_perfect_bump_occurred),
        on_perfect_bump_occurred,
    ));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let entity_a = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::PerfectBumpOccurred,
            1.5,
        )]))
        .id();

    let entity_b = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_b",
            Trigger::PerfectBumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack_a = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity_a)
        .expect("entity_a should have EffectStack");
    assert_eq!(stack_a.len(), 1);

    let stack_b = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity_b)
        .expect("entity_b should have EffectStack");
    assert_eq!(stack_b.len(), 1);
}

// ====================================================================
// Behavior 36: on_perfect_bump_occurred filters out Early and Late
// ====================================================================

#[test]
fn on_perfect_bump_occurred_filters_out_early_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_perfect_bump_occurred),
        on_perfect_bump_occurred,
    ));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::PerfectBumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Early,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "PerfectBumpOccurred should not fire on Early grade"
    );
}

#[test]
fn on_perfect_bump_occurred_filters_out_late_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_perfect_bump_occurred),
        on_perfect_bump_occurred,
    ));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::PerfectBumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Late,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "PerfectBumpOccurred should not fire on Late grade"
    );
}

// ====================================================================
// Behavior 37: on_perfect_bump_occurred provides TriggerContext::Bump
// ====================================================================

#[test]
fn on_perfect_bump_occurred_provides_bump_context_resolves_to_breaker() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_perfect_bump_occurred),
        on_perfect_bump_occurred,
    ));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let _entity = app
        .world_mut()
        .spawn(BoundEffects(vec![on_target_tree(
            "chip_a",
            Trigger::PerfectBumpOccurred,
            BumpTarget::Breaker,
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
        .expect("breaker_entity should gain EffectStack via On(Bump(Breaker))");
    assert_eq!(breaker_stack.len(), 1);
}

// ====================================================================
// Behavior 38: on_early_bump_occurred fires only for Early
// ====================================================================

#[test]
fn on_early_bump_occurred_fires_on_early_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_early_bump_occurred),
        on_early_bump_occurred,
    ));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::EarlyBumpOccurred,
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
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack for Early");
    assert_eq!(stack.len(), 1);
}

// ====================================================================
// Behavior 39: on_early_bump_occurred filters out Perfect and Late
// ====================================================================

#[test]
fn on_early_bump_occurred_filters_out_perfect_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_early_bump_occurred),
        on_early_bump_occurred,
    ));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::EarlyBumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "EarlyBumpOccurred should not fire on Perfect grade"
    );
}

#[test]
fn on_early_bump_occurred_filters_out_late_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_early_bump_occurred),
        on_early_bump_occurred,
    ));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::EarlyBumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Late,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "EarlyBumpOccurred should not fire on Late grade"
    );
}

// ====================================================================
// Behavior 40: on_late_bump_occurred fires only for Late
// ====================================================================

#[test]
fn on_late_bump_occurred_fires_on_late_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_late_bump_occurred),
        on_late_bump_occurred,
    ));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::LateBumpOccurred,
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
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack for Late");
    assert_eq!(stack.len(), 1);
}

// ====================================================================
// Behavior 41: on_late_bump_occurred filters out Perfect and Early
// ====================================================================

#[test]
fn on_late_bump_occurred_filters_out_perfect_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_late_bump_occurred),
        on_late_bump_occurred,
    ));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::LateBumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "LateBumpOccurred should not fire on Perfect grade"
    );
}

#[test]
fn on_late_bump_occurred_filters_out_early_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_late_bump_occurred),
        on_late_bump_occurred,
    ));

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::LateBumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Early,
        bolt:    Some(bolt_entity),
        breaker: breaker_entity,
    }]));

    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "LateBumpOccurred should not fire on Early grade"
    );
}

// ====================================================================
// Behavior 42: on_bump_whiff_occurred walks all entities
// ====================================================================

#[test]
fn on_bump_whiff_occurred_walks_all_entities_with_bound_effects() {
    let mut app = bump_whiff_occurred_test_app();

    let entity_a = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::BumpWhiffOccurred,
            1.5,
        )]))
        .id();

    let entity_b = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_b",
            Trigger::BumpWhiffOccurred,
            1.5,
        )]))
        .id();

    // Entity without BoundEffects
    let entity_c = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBumpWhiffedMessages(vec![BumpWhiffed]));

    tick(&mut app);

    let stack_a = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity_a)
        .expect("entity_a should have EffectStack");
    assert_eq!(stack_a.len(), 1);

    let stack_b = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity_b)
        .expect("entity_b should have EffectStack");
    assert_eq!(stack_b.len(), 1);

    let stack_c = app.world().get::<EffectStack<SpeedBoostConfig>>(entity_c);
    assert!(stack_c.is_none(), "entity_c without BoundEffects: no stack");
}

// ====================================================================
// Behavior 43: on_bump_whiff_occurred uses TriggerContext::None
// ====================================================================

#[test]
fn on_bump_whiff_occurred_uses_trigger_context_none_on_bump_cannot_resolve() {
    let mut app = bump_whiff_occurred_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![on_target_tree(
            "chip_a",
            Trigger::BumpWhiffOccurred,
            BumpTarget::Bolt,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpWhiffedMessages(vec![BumpWhiffed]));

    tick(&mut app);

    let bolt_stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_entity);
    assert!(
        bolt_stack.is_none(),
        "On(Bump(Bolt)) should not resolve with TriggerContext::None"
    );

    let owner_stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        owner_stack.is_none(),
        "effect should not fire on owner when On cannot resolve"
    );
}

// ====================================================================
// Behavior 44: on_bump_whiff_occurred is no-op without messages
// ====================================================================

#[test]
fn on_bump_whiff_occurred_is_noop_without_messages() {
    let mut app = bump_whiff_occurred_test_app();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::BumpWhiffOccurred,
            1.5,
        )]))
        .id();

    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(stack.is_none(), "no EffectStack without messages");
}

// ====================================================================
// Behavior 45: on_bump_whiff_occurred handles multiple whiff messages
// ====================================================================

#[test]
fn on_bump_whiff_occurred_handles_multiple_messages_per_frame() {
    let mut app = bump_whiff_occurred_test_app();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip_a",
            Trigger::BumpWhiffOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpWhiffedMessages(vec![BumpWhiffed, BumpWhiffed]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist after two whiff messages");
    assert_eq!(stack.len(), 2);
}
