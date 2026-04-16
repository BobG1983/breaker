use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::super::{super::system::*, helpers::*};
use crate::{
    effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        storage::BoundEffects,
        traits::Fireable,
        types::{Tree, Trigger, TriggerContext},
    },
    state::types::NodeState,
};

// ----------------------------------------------------------------
// Behavior 5: Disarm clears the fired participant's stack when a
//             single participant was recorded.
// ----------------------------------------------------------------

#[test]
fn shape_d_disarm_clears_single_fired_participant_stack() {
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

    // Precondition: bolt has 1 entry, owner has none
    assert_eq!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .unwrap()
            .len(),
        1
    );
    assert!(world.get::<EffectStack<SpeedBoostConfig>>(owner).is_none());

    // Disarm
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Armed entry removed
    let bound = world.get::<BoundEffects>(owner).unwrap();
    assert_eq!(bound.0.len(), 1, "Only the original During entry remains");
    assert!(
        matches!(bound.0[0].1, Tree::During(..)),
        "Remaining entry should be the During"
    );

    // Bolt's stack cleared via reverse_effect on the participant
    let bolt_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("Bolt stack should still exist as a component");
    assert!(
        bolt_stack.is_empty(),
        "Bolt's stack should be empty after disarm (reversed on participant)"
    );

    // Owner still has no stack
    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(owner).is_none(),
        "Owner should still have no EffectStack after disarm"
    );

    // DuringActive cleared
    let da = world.get::<DuringActive>(owner).unwrap();
    assert!(!da.0.contains("chip_redirect"));
}

// ----------------------------------------------------------------
// Behavior 6: Disarm clears both entries when the same participant
//             was fired twice.
// ----------------------------------------------------------------

#[test]
fn shape_d_disarm_clears_both_entries_when_same_participant_fired_twice() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let owner = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();
    let bolt = world.spawn_empty().id();

    // Arm
    evaluate_conditions(&mut world);

    // Fire twice on the same bolt
    let context = TriggerContext::Bump {
        bolt:    Some(bolt),
        breaker: owner,
    };
    walk_entity_effects(&mut world, owner, &Trigger::Bumped, &context);
    walk_entity_effects(&mut world, owner, &Trigger::Bumped, &context);

    // Precondition: bolt has 2 stack entries
    assert_eq!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .unwrap()
            .len(),
        2,
        "Precondition: bolt should have 2 entries from 2 fires"
    );

    // Disarm
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Bolt's stack empty
    let bolt_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("Bolt stack should still exist as a component");
    assert!(
        bolt_stack.is_empty(),
        "Bolt's stack should be empty after both entries are reversed"
    );
}

// ----------------------------------------------------------------
// Behavior 7: Disarm clears stacks on every distinct fired participant.
// ----------------------------------------------------------------

#[test]
fn shape_d_disarm_clears_stacks_on_every_distinct_fired_participant() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let owner = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();
    let bolt_a = world.spawn_empty().id();
    let bolt_b = world.spawn_empty().id();

    // Arm
    evaluate_conditions(&mut world);

    // Fire on bolt_a and bolt_b
    walk_entity_effects(
        &mut world,
        owner,
        &Trigger::Bumped,
        &TriggerContext::Bump {
            bolt:    Some(bolt_a),
            breaker: owner,
        },
    );
    walk_entity_effects(
        &mut world,
        owner,
        &Trigger::Bumped,
        &TriggerContext::Bump {
            bolt:    Some(bolt_b),
            breaker: owner,
        },
    );

    // Precondition: each bolt has 1 entry
    assert_eq!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(bolt_a)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(bolt_b)
            .unwrap()
            .len(),
        1
    );

    // Disarm
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Both stacks empty (order-agnostic)
    let a_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(bolt_a)
        .expect("bolt_a stack should still exist");
    assert!(a_stack.is_empty(), "bolt_a stack should be empty");

    let b_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(bolt_b)
        .expect("bolt_b stack should still exist");
    assert!(b_stack.is_empty(), "bolt_b stack should be empty");

    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(owner).is_none(),
        "Owner must still have no EffectStack"
    );

    let bound = world.get::<BoundEffects>(owner).unwrap();
    assert_eq!(bound.0.len(), 1);

    let da = world.get::<DuringActive>(owner).unwrap();
    assert!(!da.0.contains("chip_redirect"));
}

// ----------------------------------------------------------------
// Behavior 8: Disarm with no fires during the armed lifetime does
//             not panic and reverses nothing.
// ----------------------------------------------------------------

#[test]
fn shape_d_disarm_with_no_fires_does_not_panic() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let owner = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();

    // Arm
    evaluate_conditions(&mut world);

    // Precondition: armed entry installed, DuringActive contains source
    assert_eq!(world.get::<BoundEffects>(owner).unwrap().0.len(), 2);
    assert!(
        world
            .get::<DuringActive>(owner)
            .unwrap()
            .0
            .contains("chip_redirect")
    );

    // No walk_entity_effects calls.

    // Disarm — must not panic, empty participant set path
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Armed entry removed
    assert_eq!(world.get::<BoundEffects>(owner).unwrap().0.len(), 1);

    // No stack was ever created
    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(owner).is_none(),
        "Owner should have no EffectStack (never created)"
    );

    // DuringActive cleared
    let da = world.get::<DuringActive>(owner).unwrap();
    assert!(!da.0.contains("chip_redirect"));
}

// ----------------------------------------------------------------
// Behavior 9: Re-arming after disarm starts with an empty fired-
//             participant set.
// ----------------------------------------------------------------

#[test]
fn shape_d_re_arming_after_disarm_starts_with_empty_fired_participant_set() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let owner = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();
    let bolt_1 = world.spawn_empty().id();

    // Arm
    evaluate_conditions(&mut world);

    // Fire once
    walk_entity_effects(
        &mut world,
        owner,
        &Trigger::Bumped,
        &TriggerContext::Bump {
            bolt:    Some(bolt_1),
            breaker: owner,
        },
    );

    // Disarm — bolt_1's stack cleared
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Re-arm
    world.insert_resource(State::new(NodeState::Playing));
    evaluate_conditions(&mut world);

    // Second disarm with no fires between re-arm and disarm —
    // must not panic (tracking was cleared on re-arm / previous disarm)
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // bolt_1 stack still empty — not re-reversed, not re-fired
    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(bolt_1)
        .expect("bolt_1 stack should still exist");
    assert!(
        stack.is_empty(),
        "bolt_1 stack should still be empty after second disarm"
    );

    // BoundEffects has only the original During
    let bound = world.get::<BoundEffects>(owner).unwrap();
    assert_eq!(bound.0.len(), 1);

    // DuringActive does not contain the source
    let da = world.get::<DuringActive>(owner).unwrap();
    assert!(!da.0.contains("chip_redirect"));
}

// ----------------------------------------------------------------
// Behavior 10: Disarm does not reverse on the owner when the owner
//              is not a fired participant.
// ----------------------------------------------------------------

#[test]
fn shape_d_disarm_does_not_reverse_owner_when_owner_is_not_participant() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let owner = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();
    let bolt = world.spawn_empty().id();

    // Arm
    evaluate_conditions(&mut world);

    // Fire on bolt
    walk_entity_effects(
        &mut world,
        owner,
        &Trigger::Bumped,
        &TriggerContext::Bump {
            bolt:    Some(bolt),
            breaker: owner,
        },
    );

    // Manually plant a stray entry on the owner keyed by the armed source,
    // simulating an unrelated code path that happened to push an entry
    // with this source name. The owner was NEVER a fired participant,
    // so this must survive disarm.
    SpeedBoostConfig {
        multiplier: OrderedFloat(1.5),
    }
    .fire(owner, "chip_redirect#armed[0]", &mut world);

    // Precondition: owner has 1 entry, bolt has 1 entry
    assert_eq!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(owner)
            .unwrap()
            .len(),
        1,
        "Precondition: owner should have 1 entry (manually planted)"
    );
    assert_eq!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .unwrap()
            .len(),
        1,
        "Precondition: bolt should have 1 entry from fire"
    );

    // Disarm
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Bolt cleared (participant reversal)
    let bolt_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("Bolt stack should still exist");
    assert!(
        bolt_stack.is_empty(),
        "Bolt stack should be empty after participant-targeted reversal"
    );

    // Owner's stray entry NOT removed — owner was never a fired participant
    let owner_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(owner)
        .expect("Owner stack should still exist");
    assert_eq!(
        owner_stack.len(),
        1,
        "Owner stack should still have 1 entry — reversal must not over-reach"
    );
}

// ----------------------------------------------------------------
// Behavior 12: Despawned participant entity during disarm does not
//              panic.
// ----------------------------------------------------------------

#[test]
fn shape_d_despawned_participant_during_disarm_does_not_panic() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let owner = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();
    let bolt = world.spawn_empty().id();

    // Arm
    evaluate_conditions(&mut world);

    // Fire on bolt
    walk_entity_effects(
        &mut world,
        owner,
        &Trigger::Bumped,
        &TriggerContext::Bump {
            bolt:    Some(bolt),
            breaker: owner,
        },
    );

    // Despawn bolt — its id is still in the tracking (if any)
    world.despawn(bolt);

    // Disarm — must not panic
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Armed entry removed, DuringActive cleared
    let bound = world.get::<BoundEffects>(owner).unwrap();
    assert_eq!(bound.0.len(), 1);

    let da = world.get::<DuringActive>(owner).unwrap();
    assert!(!da.0.contains("chip_redirect"));
}

// ----------------------------------------------------------------
// Behavior 13: Despawned owner entity during disarm does not panic.
// ----------------------------------------------------------------

#[test]
fn shape_d_despawned_owner_during_disarm_does_not_panic() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let owner = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();
    let bolt = world.spawn_empty().id();

    // Arm
    evaluate_conditions(&mut world);

    // Fire on bolt
    walk_entity_effects(
        &mut world,
        owner,
        &Trigger::Bumped,
        &TriggerContext::Bump {
            bolt:    Some(bolt),
            breaker: owner,
        },
    );

    // Despawn owner
    world.despawn(owner);

    // Disarm — must not panic. evaluate_conditions must tolerate
    // the owner being gone.
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);
    // No assert: passing is "did not panic".
}
