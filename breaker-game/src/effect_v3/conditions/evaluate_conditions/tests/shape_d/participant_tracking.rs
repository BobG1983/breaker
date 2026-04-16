use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::super::{super::system::*, helpers::*};
use crate::{
    effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        storage::BoundEffects,
        types::{Trigger, TriggerContext},
    },
    state::types::NodeState,
};

// ================================================================
// Wave D — Shape D Reversal on Resolved Participants
// ================================================================

// ----------------------------------------------------------------
// Behavior 1: Armed On fire on a single non-owner participant
//             records the participant for later reversal.
// ----------------------------------------------------------------

#[test]
fn shape_d_armed_fire_on_non_owner_participant_goes_to_participant_stack() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let owner = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();
    let bolt = world.spawn_empty().id();

    // Precondition: owner has a single (unarmed) During entry
    assert_eq!(
        world.get::<BoundEffects>(owner).unwrap().0.len(),
        1,
        "Precondition: owner should have 1 entry in BoundEffects before arming"
    );

    // Arm
    evaluate_conditions(&mut world);

    // Fire with the bolt in context (non-owner participant)
    let context = TriggerContext::Bump {
        bolt:    Some(bolt),
        breaker: owner,
    };
    walk_entity_effects(&mut world, owner, &Trigger::Bumped, &context);

    // Effect lands on the bolt, NOT on the owner
    let bolt_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("Bolt should have EffectStack<SpeedBoostConfig>");
    assert_eq!(bolt_stack.len(), 1, "Bolt should have exactly 1 entry");
    let entry = bolt_stack.iter().next().unwrap();
    assert_eq!(entry.0, "chip_redirect#armed[0]");

    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(owner).is_none(),
        "Owner must NOT have an EffectStack — effect goes to participant"
    );

    // Armed entry still present in BoundEffects
    let bound = world.get::<BoundEffects>(owner).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should still have 2 entries (During + armed On)"
    );
    assert!(
        bound
            .0
            .iter()
            .any(|(name, _)| name == "chip_redirect#armed[0]"),
        "Armed entry must still be present after fire"
    );

    // DuringActive still contains the source
    let da = world.get::<DuringActive>(owner).unwrap();
    assert!(
        da.0.contains("chip_redirect"),
        "DuringActive should still contain 'chip_redirect' while armed"
    );
}

// ----------------------------------------------------------------
// Behavior 2: Armed On fire with `TriggerContext::Bump { bolt: None, .. }`
//             does not record any participant.
// ----------------------------------------------------------------

#[test]
fn shape_d_armed_fire_with_none_participant_does_not_record() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let owner = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();

    // Arm
    evaluate_conditions(&mut world);

    // Precondition: armed entry is installed
    assert_eq!(
        world.get::<BoundEffects>(owner).unwrap().0.len(),
        2,
        "Precondition: armed On entry must be installed"
    );

    // Fire with bolt = None
    let context = TriggerContext::Bump {
        bolt:    None,
        breaker: owner,
    };
    walk_entity_effects(&mut world, owner, &Trigger::Bumped, &context);

    // No stack on owner (participant was None, nothing resolved)
    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(owner).is_none(),
        "Owner should have no EffectStack when bolt is None"
    );

    // BoundEffects still has 2 entries
    assert_eq!(
        world.get::<BoundEffects>(owner).unwrap().0.len(),
        2,
        "BoundEffects should still have 2 entries"
    );

    // DuringActive still contains the source
    let da = world.get::<DuringActive>(owner).unwrap();
    assert!(
        da.0.contains("chip_redirect"),
        "DuringActive should still contain 'chip_redirect' (still armed)"
    );
}

// ----------------------------------------------------------------
// Behavior 3: Armed On fire on the same participant twice stacks
//             two entries on that participant.
// ----------------------------------------------------------------

#[test]
fn shape_d_armed_fire_twice_on_same_participant_stacks_two_entries() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let owner = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();
    let bolt = world.spawn_empty().id();

    // Arm
    evaluate_conditions(&mut world);

    let context = TriggerContext::Bump {
        bolt:    Some(bolt),
        breaker: owner,
    };

    // Fire twice on the same bolt
    walk_entity_effects(&mut world, owner, &Trigger::Bumped, &context);
    walk_entity_effects(&mut world, owner, &Trigger::Bumped, &context);

    let bolt_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("Bolt should have EffectStack<SpeedBoostConfig>");
    assert_eq!(
        bolt_stack.len(),
        2,
        "Bolt should have exactly 2 entries from two fires"
    );

    let expected_cfg = SpeedBoostConfig {
        multiplier: OrderedFloat(1.5),
    };
    for entry in bolt_stack.iter() {
        assert_eq!(entry.0, "chip_redirect#armed[0]");
        assert_eq!(entry.1, expected_cfg);
    }

    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(owner).is_none(),
        "Owner must NOT have EffectStack"
    );
}

// ----------------------------------------------------------------
// Behavior 4: Armed On fire on two different participants records both.
// ----------------------------------------------------------------

#[test]
fn shape_d_armed_fire_on_two_different_participants_records_both() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let owner = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();
    let bolt_a = world.spawn_empty().id();
    let bolt_b = world.spawn_empty().id();

    // Arm
    evaluate_conditions(&mut world);

    // Fire on bolt_a
    walk_entity_effects(
        &mut world,
        owner,
        &Trigger::Bumped,
        &TriggerContext::Bump {
            bolt:    Some(bolt_a),
            breaker: owner,
        },
    );

    // Fire on bolt_b
    walk_entity_effects(
        &mut world,
        owner,
        &Trigger::Bumped,
        &TriggerContext::Bump {
            bolt:    Some(bolt_b),
            breaker: owner,
        },
    );

    let a_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(bolt_a)
        .expect("bolt_a should have EffectStack");
    assert_eq!(a_stack.len(), 1);
    assert_eq!(a_stack.iter().next().unwrap().0, "chip_redirect#armed[0]");

    let b_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(bolt_b)
        .expect("bolt_b should have EffectStack");
    assert_eq!(b_stack.len(), 1);
    assert_eq!(b_stack.iter().next().unwrap().0, "chip_redirect#armed[0]");

    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(owner).is_none(),
        "Owner must NOT have EffectStack"
    );
}
