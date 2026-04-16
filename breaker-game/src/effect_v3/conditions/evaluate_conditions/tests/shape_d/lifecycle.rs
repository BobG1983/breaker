use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::super::{super::system::*, helpers::*};
use crate::{
    effect_v3::{
        effects::{DamageBoostConfig, SpeedBoostConfig},
        stacking::EffectStack,
        storage::{ArmedFiredParticipants, BoundEffects},
        types::{
            Condition, EntityKind, ImpactTarget, ParticipantTarget, ReversibleEffectType,
            ScopedTerminal, ScopedTree, Tree, Trigger, TriggerContext,
        },
    },
    state::types::NodeState,
};

// ----------------------------------------------------------------
// Behavior 14: Shape D full lifecycle with cross-participant fires.
// ----------------------------------------------------------------

#[test]
fn shape_d_full_lifecycle_with_cross_participant_fires() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let owner = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();
    let bolt_1 = world.spawn_empty().id();
    let bolt_2 = world.spawn_empty().id();
    let bolt_3 = world.spawn_empty().id();

    // Step 1: arm
    evaluate_conditions(&mut world);

    // Step 2: fire on bolt_1
    walk_entity_effects(
        &mut world,
        owner,
        &Trigger::Bumped,
        &TriggerContext::Bump {
            bolt:    Some(bolt_1),
            breaker: owner,
        },
    );

    // Step 3: fire on bolt_2
    walk_entity_effects(
        &mut world,
        owner,
        &Trigger::Bumped,
        &TriggerContext::Bump {
            bolt:    Some(bolt_2),
            breaker: owner,
        },
    );

    // Step 4: disarm
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Step 5: re-arm
    world.insert_resource(State::new(NodeState::Playing));
    evaluate_conditions(&mut world);

    // Step 6: fire on bolt_3
    walk_entity_effects(
        &mut world,
        owner,
        &Trigger::Bumped,
        &TriggerContext::Bump {
            bolt:    Some(bolt_3),
            breaker: owner,
        },
    );

    // Step 7: disarm
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Cumulative assertions:
    let s1 = world
        .get::<EffectStack<SpeedBoostConfig>>(bolt_1)
        .expect("bolt_1 stack should still exist");
    assert!(s1.is_empty(), "bolt_1 stack should be empty");

    let s2 = world
        .get::<EffectStack<SpeedBoostConfig>>(bolt_2)
        .expect("bolt_2 stack should still exist");
    assert!(s2.is_empty(), "bolt_2 stack should be empty");

    let s3 = world
        .get::<EffectStack<SpeedBoostConfig>>(bolt_3)
        .expect("bolt_3 stack should still exist");
    assert!(s3.is_empty(), "bolt_3 stack should be empty");

    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(owner).is_none(),
        "Owner should have no EffectStack"
    );

    let bound = world.get::<BoundEffects>(owner).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should have only the original During"
    );

    let da = world.get::<DuringActive>(owner).unwrap();
    assert!(!da.0.contains("chip_redirect"));
}

// ----------------------------------------------------------------
// Behavior 15: Multiple Shape D entries on the same entity track
//              participants independently by source.
// ----------------------------------------------------------------

#[test]
fn shape_d_multiple_entries_track_participants_independently_by_source() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let owner = world
        .spawn(BoundEffects(vec![
            during_on_bump_bolt_speed_boost(),
            // chip_reflect with Impact(Impactee), DamageBoost(2.0)
            (
                "chip_reflect".to_string(),
                Tree::During(
                    Condition::NodeActive,
                    Box::new(ScopedTree::On(
                        ParticipantTarget::Impact(ImpactTarget::Impactee),
                        ScopedTerminal::Fire(ReversibleEffectType::DamageBoost(
                            DamageBoostConfig {
                                multiplier: OrderedFloat(2.0),
                            },
                        )),
                    )),
                ),
            ),
        ]))
        .id();
    let bolt = world.spawn_empty().id();
    let impactee = world.spawn_empty().id();

    // Arm — both entries arm
    evaluate_conditions(&mut world);

    let bound = world.get::<BoundEffects>(owner).unwrap();
    assert_eq!(
        bound.0.len(),
        4,
        "BoundEffects should have 4 entries after arming both (2 originals + 2 armed)"
    );

    // Fire with Bump context — only chip_redirect#armed[0] matches
    walk_entity_effects(
        &mut world,
        owner,
        &Trigger::Bumped,
        &TriggerContext::Bump {
            bolt:    Some(bolt),
            breaker: owner,
        },
    );

    // Fire with Impact context — only chip_reflect#armed[0] matches
    walk_entity_effects(
        &mut world,
        owner,
        &Trigger::Impacted(EntityKind::Cell),
        &TriggerContext::Impact {
            impactor: owner,
            impactee,
        },
    );

    // Preconditions: bolt has 1 SpeedBoost, impactee has 1 DamageBoost
    assert_eq!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        world
            .get::<EffectStack<DamageBoostConfig>>(impactee)
            .unwrap()
            .len(),
        1
    );

    // Disarm — both entries disarm
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Both participant stacks empty
    let bolt_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("bolt stack should still exist");
    assert!(
        bolt_stack.is_empty(),
        "bolt's SpeedBoost stack should be empty after disarm"
    );

    let impactee_stack = world
        .get::<EffectStack<DamageBoostConfig>>(impactee)
        .expect("impactee stack should still exist");
    assert!(
        impactee_stack.is_empty(),
        "impactee's DamageBoost stack should be empty after disarm"
    );

    // BoundEffects has exactly 2 entries (the two Durings)
    let bound = world.get::<BoundEffects>(owner).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should have 2 During entries after disarm (both armed entries removed)"
    );
    for (_, tree) in &bound.0 {
        assert!(
            matches!(tree, Tree::During(..)),
            "Remaining entries should all be During trees"
        );
    }

    // DuringActive contains neither
    let da = world.get::<DuringActive>(owner).unwrap();
    assert!(!da.0.contains("chip_redirect"));
    assert!(!da.0.contains("chip_reflect"));
}

// ----------------------------------------------------------------
// Behavior 16: Fired-participant tracking persists across frames
//              until disarm.
// ----------------------------------------------------------------

#[test]
fn shape_d_fired_participant_tracking_persists_across_frames_until_disarm() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let owner = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();
    let bolt = world.spawn_empty().id();

    // Arm
    evaluate_conditions(&mut world);

    // Fire once
    walk_entity_effects(
        &mut world,
        owner,
        &Trigger::Bumped,
        &TriggerContext::Bump {
            bolt:    Some(bolt),
            breaker: owner,
        },
    );

    // Precondition: bolt has 1 entry
    assert_eq!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .unwrap()
            .len(),
        1
    );

    // Second evaluation while condition stays true (still Playing) —
    // no-op: not re-armed, not disarmed, tracking untouched
    evaluate_conditions(&mut world);

    // bolt still has 1 entry — tracking must survive the no-op evaluation
    let bolt_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("Bolt stack should still exist");
    assert_eq!(
        bolt_stack.len(),
        1,
        "Bolt stack should still have 1 entry after no-op evaluation"
    );
    assert_eq!(
        world.get::<BoundEffects>(owner).unwrap().0.len(),
        2,
        "BoundEffects should still have 2 entries"
    );

    // Now disarm — tracking is consulted and bolt's stack is cleared
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    let bolt_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("Bolt stack should still exist");
    assert!(
        bolt_stack.is_empty(),
        "Bolt stack should be empty after disarm — tracking persisted across frames"
    );
}

// ----------------------------------------------------------------
// Behavior 17: Disarming the same source twice in a row does not
//              panic (idempotent reversal).
// ----------------------------------------------------------------

#[test]
fn shape_d_disarming_same_source_twice_does_not_panic() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let owner = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();
    let bolt = world.spawn_empty().id();

    // Arm
    evaluate_conditions(&mut world);

    // Fire once
    walk_entity_effects(
        &mut world,
        owner,
        &Trigger::Bumped,
        &TriggerContext::Bump {
            bolt:    Some(bolt),
            breaker: owner,
        },
    );

    // First disarm
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Second disarm, still in Loading — was_active is false, must short-circuit
    evaluate_conditions(&mut world);

    // Bolt stack still empty
    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("Bolt stack should still exist");
    assert!(stack.is_empty(), "Bolt stack should still be empty");

    // BoundEffects still has only the original During
    assert_eq!(world.get::<BoundEffects>(owner).unwrap().0.len(), 1);

    // DuringActive still does not contain the source
    let da = world.get::<DuringActive>(owner).unwrap();
    assert!(!da.0.contains("chip_redirect"));
}

// ----------------------------------------------------------------
// Sanity: `ArmedFiredParticipants` type is in scope.
// ----------------------------------------------------------------

#[test]
fn shape_d_armed_fired_participants_component_is_in_scope() {
    let mut world = World::new();
    let owner = world.spawn(ArmedFiredParticipants::default()).id();
    assert!(world.get::<ArmedFiredParticipants>(owner).is_some());
}
