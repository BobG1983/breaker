use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{super::system::*, helpers::*};
use crate::{
    effect_v3::{
        effects::{DamageBoostConfig, SpeedBoostConfig},
        stacking::EffectStack,
        storage::{ArmedFiredParticipants, BoundEffects},
        traits::Fireable,
        types::{
            BumpTarget, Condition, EffectType, EntityKind, ImpactTarget, ParticipantTarget,
            ReversibleEffectType, ScopedTerminal, ScopedTree, Terminal, Tree, Trigger,
            TriggerContext,
        },
    },
    state::types::NodeState,
};

// ================================================================
// Shape D: During(Cond, On(Participant, Fire(reversible)))
// ================================================================

// ----------------------------------------------------------------
// Behavior 13: Cond entering true installs a Tree::On armed entry
//              into BoundEffects with #armed[0] key
// ----------------------------------------------------------------

#[test]
fn shape_d_cond_entering_true_installs_armed_on_entry() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();

    evaluate_conditions(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should contain 2 entries: original During + armed On"
    );

    // Original During entry preserved
    assert_eq!(bound.0[0].0, "chip_redirect");
    assert!(
        matches!(bound.0[0].1, Tree::During(..)),
        "First entry should still be the original During tree"
    );

    // Armed On entry installed
    let armed = bound
        .0
        .iter()
        .find(|(name, _)| name == "chip_redirect#armed[0]");
    assert!(
        armed.is_some(),
        "Should find armed On with key 'chip_redirect#armed[0]'"
    );
    let (_, armed_tree) = armed.unwrap();
    // Verify it is Tree::On with widened Terminal::Fire(EffectType::...), not ScopedTerminal
    assert!(
        matches!(
            armed_tree,
            Tree::On(
                ParticipantTarget::Bump(BumpTarget::Bolt),
                Terminal::Fire(EffectType::SpeedBoost(..))
            )
        ),
        "Armed entry should be Tree::On(Bump(Bolt), Terminal::Fire(EffectType::SpeedBoost(...)))"
    );

    let da = world
        .get::<DuringActive>(entity)
        .expect("DuringActive should exist");
    assert!(
        da.0.contains("chip_redirect"),
        "DuringActive should contain 'chip_redirect'"
    );
}

// ----------------------------------------------------------------
// Behavior 14: Cond staying true does not re-install the armed On entry
// ----------------------------------------------------------------

#[test]
fn shape_d_cond_staying_true_does_not_reinstall_armed_on_entry() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();

    evaluate_conditions(&mut world);
    evaluate_conditions(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should contain exactly 2 entries after two evaluations"
    );
}

// ----------------------------------------------------------------
// Behavior 15: Firing trigger with matching participant context
//              redirects effect to participant entity
// ----------------------------------------------------------------

#[test]
fn shape_d_trigger_with_matching_context_redirects_to_participant() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity_a = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();
    let entity_b = world.spawn_empty().id();

    // Arm
    evaluate_conditions(&mut world);

    // Fire with matching context
    let context = TriggerContext::Bump {
        bolt:    Some(entity_b),
        breaker: entity_a,
    };
    walk_entity_effects(&mut world, entity_a, &Trigger::Bumped, &context);

    // Effect should be on entity_b (the bolt), not entity_a (the owner)
    let bolt_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity_b)
        .expect("EffectStack should exist on bolt entity (entity_b)");
    assert_eq!(bolt_stack.len(), 1);

    let entry = bolt_stack.iter().next().unwrap();
    assert_eq!(
        entry.0, "chip_redirect#armed[0]",
        "Source on bolt's stack must be the armed key"
    );

    assert!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(entity_a)
            .is_none(),
        "Owner (entity_a) should have no EffectStack — effect goes to participant"
    );
}

// ----------------------------------------------------------------
// Behavior 16: Participant filter correctness — no bolt in context
// ----------------------------------------------------------------

#[test]
fn shape_d_no_bolt_in_context_does_not_fire() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity_a = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();

    // Arm
    evaluate_conditions(&mut world);

    // Precondition: armed entry must be installed
    let bound = world.get::<BoundEffects>(entity_a).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "Precondition: armed On entry must be installed before testing participant filter"
    );

    // Context with bolt = None
    let context = TriggerContext::Bump {
        bolt:    None,
        breaker: entity_a,
    };
    walk_entity_effects(&mut world, entity_a, &Trigger::Bumped, &context);

    assert!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(entity_a)
            .is_none(),
        "No EffectStack should exist when bolt is None in context"
    );
}

// Edge case: TriggerContext::None also produces no stack entries
#[test]
fn shape_d_context_none_does_not_fire() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity_a = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();

    evaluate_conditions(&mut world);

    // Precondition: armed entry must be installed
    let bound = world.get::<BoundEffects>(entity_a).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "Precondition: armed On entry must be installed before testing TriggerContext::None"
    );

    walk_entity_effects(
        &mut world,
        entity_a,
        &Trigger::Bumped,
        &TriggerContext::None,
    );

    assert!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(entity_a)
            .is_none(),
        "No EffectStack should exist when context is None"
    );
}

// ----------------------------------------------------------------
// Behavior 17: Participant filter — Impact context does not match
//              Bump(Bolt) target
// ----------------------------------------------------------------

#[test]
fn shape_d_impact_context_does_not_match_bump_bolt_target() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity_a = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();
    let entity_b = world.spawn_empty().id();
    let entity_c = world.spawn_empty().id();

    // Arm
    evaluate_conditions(&mut world);

    // Precondition: armed entry must be installed
    let bound = world.get::<BoundEffects>(entity_a).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "Precondition: armed On entry must be installed before testing mismatched context"
    );

    // Fire with Impact context (wrong context type for Bump(Bolt))
    let context = TriggerContext::Impact {
        impactor: entity_b,
        impactee: entity_c,
    };
    walk_entity_effects(&mut world, entity_a, &Trigger::Bumped, &context);

    assert!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(entity_a)
            .is_none(),
        "No EffectStack on owner"
    );
    assert!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(entity_b)
            .is_none(),
        "No EffectStack on impactor"
    );
    assert!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(entity_c)
            .is_none(),
        "No EffectStack on impactee"
    );
}

// ----------------------------------------------------------------
// Behavior 18: Firing multiple times with matching context stacks
//              effects on participant entity
// ----------------------------------------------------------------

#[test]
fn shape_d_multiple_fires_stack_on_participant() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity_a = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();
    let entity_b = world.spawn_empty().id();

    // Arm
    evaluate_conditions(&mut world);

    let context = TriggerContext::Bump {
        bolt:    Some(entity_b),
        breaker: entity_a,
    };

    // Fire 3 times
    for _ in 0..3 {
        walk_entity_effects(&mut world, entity_a, &Trigger::Bumped, &context);
    }

    let bolt_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity_b)
        .expect("EffectStack should exist on bolt entity");
    assert_eq!(
        bolt_stack.len(),
        3,
        "Bolt entity should have 3 stack entries from 3 fires"
    );

    for entry in bolt_stack.iter() {
        assert_eq!(entry.0, "chip_redirect#armed[0]");
    }
}

// ----------------------------------------------------------------
// Behavior 19 (Wave 7c -> Wave D rewrite): Shape D disarm reverses
// effects on the OWNER entity when the owner was itself the
// fired participant (degenerate bolt = owner case).
// ----------------------------------------------------------------

#[test]
fn shape_d_disarm_reverses_when_owner_is_participant() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let owner = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();

    // Arm
    evaluate_conditions(&mut world);

    // Degenerate context: bolt == owner (valid — same entity can be both)
    let context = TriggerContext::Bump {
        bolt:    Some(owner),
        breaker: owner,
    };

    // Fire 2x — effects go to owner (since bolt resolves to owner)
    for _ in 0..2 {
        walk_entity_effects(&mut world, owner, &Trigger::Bumped, &context);
    }

    // Precondition: owner has 2 stack entries because the owner IS
    // the resolved participant in this degenerate context
    assert_eq!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(owner)
            .unwrap()
            .len(),
        2,
        "Precondition: owner should have 2 stack entries (owner == participant)"
    );

    // Disarm
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Armed entry removed
    let bound = world.get::<BoundEffects>(owner).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should have 1 entry (original During only)"
    );
    assert_eq!(bound.0[0].0, "chip_redirect");

    // Stack cleared on owner — because owner was a tracked fired
    // participant, not because reversal runs on the owner blindly
    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(owner)
        .expect("Stack should still exist");
    assert!(
        stack.is_empty(),
        "Stack on owner should be empty after participant-targeted reversal"
    );

    // DuringActive cleared
    let da = world.get::<DuringActive>(owner).unwrap();
    assert!(
        !da.0.contains("chip_redirect"),
        "DuringActive should not contain 'chip_redirect'"
    );
}

// ----------------------------------------------------------------
// Behavior 20: Firing the trigger after disarm is a no-op
// ----------------------------------------------------------------

#[test]
fn shape_d_trigger_after_disarm_is_noop() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity_a = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();

    // Arm
    evaluate_conditions(&mut world);

    // Fire once with degenerate context
    let context = TriggerContext::Bump {
        bolt:    Some(entity_a),
        breaker: entity_a,
    };
    walk_entity_effects(&mut world, entity_a, &Trigger::Bumped, &context);

    // Disarm
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Fire again after disarm
    walk_entity_effects(&mut world, entity_a, &Trigger::Bumped, &context);

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity_a)
        .expect("Stack should still exist");
    assert!(
        stack.is_empty(),
        "Stack should remain empty after trigger fires post-disarm"
    );

    let bound = world.get::<BoundEffects>(entity_a).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should have only 1 entry after disarm"
    );
}

// ----------------------------------------------------------------
// Behavior 21: Re-entering Cond true re-arms with a fresh On entry
// ----------------------------------------------------------------

#[test]
fn shape_d_re_entering_true_rearms_with_fresh_on_entry() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity_a = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();

    // Arm
    evaluate_conditions(&mut world);

    // Fire 2x with degenerate context
    let context = TriggerContext::Bump {
        bolt:    Some(entity_a),
        breaker: entity_a,
    };
    for _ in 0..2 {
        walk_entity_effects(&mut world, entity_a, &Trigger::Bumped, &context);
    }

    // Disarm
    world.insert_resource(State::new(NodeState::Loading));
    evaluate_conditions(&mut world);

    // Re-arm
    world.insert_resource(State::new(NodeState::Playing));
    evaluate_conditions(&mut world);

    let bound = world.get::<BoundEffects>(entity_a).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should have 2 entries (original During + fresh armed On)"
    );
}

// ----------------------------------------------------------------
// Behavior 22: Shape D On(Impact(Impactee)) with Impact context
//              redirects to impactee
// ----------------------------------------------------------------

#[test]
fn shape_d_on_impact_impactee_redirects_to_impactee() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity_owner = world
        .spawn(BoundEffects(vec![(
            "chip_reflect".to_string(),
            Tree::During(
                Condition::NodeActive,
                Box::new(ScopedTree::On(
                    ParticipantTarget::Impact(ImpactTarget::Impactee),
                    ScopedTerminal::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(2.0),
                    })),
                )),
            ),
        )]))
        .id();
    let entity_cell = world.spawn_empty().id();

    // Arm
    evaluate_conditions(&mut world);

    // Fire with Impact context
    let context = TriggerContext::Impact {
        impactor: entity_owner,
        impactee: entity_cell,
    };
    walk_entity_effects(
        &mut world,
        entity_owner,
        &Trigger::Impacted(EntityKind::Cell),
        &context,
    );

    let cell_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity_cell)
        .expect("EffectStack should exist on impactee (cell) entity");
    assert_eq!(cell_stack.len(), 1);

    let entry = cell_stack.iter().next().unwrap();
    assert_eq!(
        entry.0, "chip_reflect#armed[0]",
        "Source on impactee's stack must be the armed key"
    );
}

// ----------------------------------------------------------------
// Behavior 23: Shape D armed entry with Bump(Breaker) redirects
//              to breaker entity
// ----------------------------------------------------------------

#[test]
fn shape_d_on_bump_breaker_redirects_to_breaker_entity() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    let entity_a = world
        .spawn(BoundEffects(vec![(
            "chip_breaker_buff".to_string(),
            Tree::During(
                Condition::NodeActive,
                Box::new(ScopedTree::On(
                    ParticipantTarget::Bump(BumpTarget::Breaker),
                    ScopedTerminal::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    })),
                )),
            ),
        )]))
        .id();
    let entity_bolt = world.spawn_empty().id();
    let entity_breaker = world.spawn_empty().id();

    // Arm
    evaluate_conditions(&mut world);

    // Fire with Bump context
    let context = TriggerContext::Bump {
        bolt:    Some(entity_bolt),
        breaker: entity_breaker,
    };
    walk_entity_effects(&mut world, entity_a, &Trigger::Bumped, &context);

    let breaker_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity_breaker)
        .expect("EffectStack should exist on breaker entity");
    assert_eq!(breaker_stack.len(), 1);

    assert!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(entity_bolt)
            .is_none(),
        "Bolt entity should have no EffectStack"
    );
    assert!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(entity_a)
            .is_none(),
        "Owner entity should have no EffectStack"
    );
}

// ================================================================
// Behavior 27: ScopedTerminal::Fire(reversible) converts to
//              Terminal::Fire(EffectType) for armed On entry
// ================================================================

#[test]
fn scoped_terminal_fire_converts_to_terminal_fire_with_widened_type() {
    let scoped = ScopedTerminal::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
        multiplier: OrderedFloat(1.5),
    }));
    let terminal = Terminal::from(scoped);
    assert_eq!(
        terminal,
        Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        })),
        "ScopedTerminal::Fire should convert to Terminal::Fire with widened EffectType"
    );
}

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
