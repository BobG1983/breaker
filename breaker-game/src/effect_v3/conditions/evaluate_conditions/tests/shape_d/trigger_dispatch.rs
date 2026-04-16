use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::super::{super::system::*, helpers::*};
use crate::{
    effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        storage::BoundEffects,
        types::{
            BumpTarget, Condition, EffectType, EntityKind, ImpactTarget, ParticipantTarget,
            ReversibleEffectType, ScopedTerminal, ScopedTree, Terminal, Tree, Trigger,
            TriggerContext,
        },
    },
    state::types::NodeState,
};

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
