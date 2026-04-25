use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::config_impl::*;
use crate::{
    chips::definition::Rarity,
    effect_v3::{
        effects::ramping_damage::components::RampingDamageAccumulator,
        stacking::EffectStack,
        traits::{Fireable, Reversible},
    },
    prelude::{SourceId, SourceIdExt},
};

// Builder-format `SourceId` fixtures — replace arbitrary string literals with
// canonical `chip:<template>:<rarity>` keys so tests demonstrate the shape
// production code uses.

fn test_source() -> SourceId {
    SourceId::chip("Test").rarity(Rarity::Common).build()
}

fn amp_source() -> SourceId {
    SourceId::chip("Amp").rarity(Rarity::Common).build()
}

fn feedback_loop_source() -> SourceId {
    SourceId::chip("FeedbackLoop")
        .rarity(Rarity::Common)
        .build()
}

#[test]
fn fire_creates_stack_and_pushes_entry() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let config = RampingDamageConfig {
        increment: OrderedFloat(0.5),
    };

    config.fire(entity, test_source().0.as_ref(), &mut world);

    let stack = world
        .get::<EffectStack<RampingDamageConfig>>(entity)
        .unwrap();
    assert_eq!(stack.len(), 1);

    // B55 — the entry's key is a typed `SourceId`, NOT a String. The
    // post-W5 EffectStack signature stores `(SourceId, T)`, and the source
    // string passed into `Fireable::fire` is wrapped via `SourceId::from(_)`
    // at the push boundary.
    let entry = stack.iter().next().expect("stack must contain one entry");
    assert_eq!(
        entry.0,
        test_source(),
        "stack entry key must be a SourceId built from the source string"
    );
}

#[test]
fn fire_multiple_times_stacks_entries() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let config = RampingDamageConfig {
        increment: OrderedFloat(0.5),
    };

    config.fire(entity, test_source().0.as_ref(), &mut world);
    config.fire(entity, test_source().0.as_ref(), &mut world);

    let stack = world
        .get::<EffectStack<RampingDamageConfig>>(entity)
        .unwrap();
    assert_eq!(stack.len(), 2);
    assert!((stack.aggregate() - 1.0).abs() < 1e-5);
}

#[test]
fn reverse_removes_matching_entry() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let config = RampingDamageConfig {
        increment: OrderedFloat(0.5),
    };

    config.fire(entity, test_source().0.as_ref(), &mut world);
    config.reverse(entity, test_source().0.as_ref(), &mut world);

    let stack = world
        .get::<EffectStack<RampingDamageConfig>>(entity)
        .unwrap();
    assert_eq!(stack.len(), 0);
}

#[test]
fn reverse_on_entity_without_stack_is_noop() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let config = RampingDamageConfig {
        increment: OrderedFloat(0.5),
    };

    config.reverse(entity, test_source().0.as_ref(), &mut world);
}

// ── reverse_all_by_source ─────────────────────────────────────────

#[test]
fn reverse_all_by_source_removes_all_entries_from_matching_source_leaves_others() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    RampingDamageConfig {
        increment: OrderedFloat(0.5),
    }
    .fire(entity, amp_source().0.as_ref(), &mut world);
    RampingDamageConfig {
        increment: OrderedFloat(0.25),
    }
    .fire(entity, feedback_loop_source().0.as_ref(), &mut world);
    RampingDamageConfig {
        increment: OrderedFloat(1.0),
    }
    .fire(entity, amp_source().0.as_ref(), &mut world);

    // Manually set accumulator to a non-zero value.
    world
        .entity_mut(entity)
        .insert(RampingDamageAccumulator(OrderedFloat(3.0)));

    RampingDamageConfig {
        increment: OrderedFloat(0.5),
    }
    .reverse_all_by_source(entity, amp_source().0.as_ref(), &mut world);

    let stack = world
        .get::<EffectStack<RampingDamageConfig>>(entity)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 0.25).abs() < 1e-5);
    // B55 — surviving entry's key must be a typed `SourceId`.
    let entry = stack.iter().next().expect("stack must contain one entry");
    assert_eq!(
        entry.0,
        feedback_loop_source(),
        "remaining entry's key must be the feedback-loop source"
    );
    // Accumulator should still be present because the stack is non-empty.
    assert!(
        world.get::<RampingDamageAccumulator>(entity).is_some(),
        "RampingDamageAccumulator should still be present when stack is non-empty"
    );
}

#[test]
fn reverse_all_by_source_removes_accumulator_when_stack_becomes_empty() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    RampingDamageConfig {
        increment: OrderedFloat(0.5),
    }
    .fire(entity, amp_source().0.as_ref(), &mut world);
    RampingDamageConfig {
        increment: OrderedFloat(1.0),
    }
    .fire(entity, amp_source().0.as_ref(), &mut world);

    world
        .entity_mut(entity)
        .insert(RampingDamageAccumulator(OrderedFloat(2.5)));

    RampingDamageConfig {
        increment: OrderedFloat(0.5),
    }
    .reverse_all_by_source(entity, amp_source().0.as_ref(), &mut world);

    let stack = world
        .get::<EffectStack<RampingDamageConfig>>(entity)
        .unwrap();
    assert!(stack.is_empty());
    assert!(
        world.get::<RampingDamageAccumulator>(entity).is_none(),
        "RampingDamageAccumulator should be removed when stack becomes empty"
    );
}

#[test]
fn reverse_all_by_source_on_entity_without_stack_is_noop() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    RampingDamageConfig {
        increment: OrderedFloat(0.5),
    }
    .reverse_all_by_source(entity, amp_source().0.as_ref(), &mut world);
    // No panic.
}

// ── fire inserts accumulator ──────────────────────────────────────

#[test]
fn fire_inserts_accumulator_if_absent() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    RampingDamageConfig {
        increment: OrderedFloat(0.5),
    }
    .fire(entity, amp_source().0.as_ref(), &mut world);

    let acc = world.get::<RampingDamageAccumulator>(entity);
    assert!(acc.is_some(), "fire should insert RampingDamageAccumulator");
    assert_eq!(
        acc.unwrap().0,
        OrderedFloat(0.0),
        "newly inserted accumulator should be 0.0",
    );
}

#[test]
fn fire_does_not_overwrite_existing_accumulator() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    // Fire once to create the stack + accumulator.
    RampingDamageConfig {
        increment: OrderedFloat(0.5),
    }
    .fire(entity, amp_source().0.as_ref(), &mut world);

    // Manually set accumulator to non-zero to simulate gameplay usage.
    world
        .entity_mut(entity)
        .insert(RampingDamageAccumulator(OrderedFloat(3.5)));

    // Fire again from a different source.
    RampingDamageConfig {
        increment: OrderedFloat(0.5),
    }
    .fire(entity, feedback_loop_source().0.as_ref(), &mut world);

    let acc = world.get::<RampingDamageAccumulator>(entity).unwrap();
    assert_eq!(
        acc.0,
        OrderedFloat(3.5),
        "fire should not overwrite existing accumulator value",
    );

    let stack = world
        .get::<EffectStack<RampingDamageConfig>>(entity)
        .unwrap();
    assert_eq!(
        stack.len(),
        2,
        "stack should have 2 entries after second fire"
    );
}

// ── reverse removes accumulator when stack becomes empty ──────────

#[test]
fn reverse_with_single_entry_removes_accumulator() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    RampingDamageConfig {
        increment: OrderedFloat(0.5),
    }
    .fire(entity, amp_source().0.as_ref(), &mut world);

    // Manually set accumulator to non-zero.
    world
        .entity_mut(entity)
        .insert(RampingDamageAccumulator(OrderedFloat(2.0)));

    RampingDamageConfig {
        increment: OrderedFloat(0.5),
    }
    .reverse(entity, amp_source().0.as_ref(), &mut world);

    let stack = world
        .get::<EffectStack<RampingDamageConfig>>(entity)
        .unwrap();
    assert!(stack.is_empty(), "stack should be empty after reverse");
    assert!(
        world.get::<RampingDamageAccumulator>(entity).is_none(),
        "accumulator should be removed when stack becomes empty",
    );
}

#[test]
fn reverse_on_entity_without_accumulator_does_not_panic() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    // Manually insert stack with one entry but no accumulator.
    world
        .entity_mut(entity)
        .insert(EffectStack::<RampingDamageConfig>::default());
    world
        .get_mut::<EffectStack<RampingDamageConfig>>(entity)
        .unwrap()
        .push(
            amp_source(),
            RampingDamageConfig {
                increment: OrderedFloat(0.5),
            },
        );

    // Should not panic even without accumulator component.
    RampingDamageConfig {
        increment: OrderedFloat(0.5),
    }
    .reverse(entity, amp_source().0.as_ref(), &mut world);
}

// ── reverse with non-empty stack keeps accumulator ────────────────

#[test]
fn reverse_with_non_empty_stack_keeps_accumulator() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    RampingDamageConfig {
        increment: OrderedFloat(0.5),
    }
    .fire(entity, amp_source().0.as_ref(), &mut world);
    RampingDamageConfig {
        increment: OrderedFloat(0.25),
    }
    .fire(entity, feedback_loop_source().0.as_ref(), &mut world);

    // Set accumulator to 1.5.
    world
        .entity_mut(entity)
        .insert(RampingDamageAccumulator(OrderedFloat(1.5)));

    RampingDamageConfig {
        increment: OrderedFloat(0.5),
    }
    .reverse(entity, amp_source().0.as_ref(), &mut world);

    let stack = world
        .get::<EffectStack<RampingDamageConfig>>(entity)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 0.25).abs() < 1e-5);

    let acc = world.get::<RampingDamageAccumulator>(entity);
    assert!(
        acc.is_some(),
        "accumulator should remain when stack is non-empty",
    );
    assert_eq!(
        acc.unwrap().0,
        OrderedFloat(1.5),
        "accumulator value should be preserved",
    );
}

#[test]
fn reverse_with_non_empty_stack_keeps_zero_accumulator() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    RampingDamageConfig {
        increment: OrderedFloat(0.5),
    }
    .fire(entity, amp_source().0.as_ref(), &mut world);
    RampingDamageConfig {
        increment: OrderedFloat(0.25),
    }
    .fire(entity, feedback_loop_source().0.as_ref(), &mut world);

    // Accumulator is already 0.0 from fire.
    RampingDamageConfig {
        increment: OrderedFloat(0.5),
    }
    .reverse(entity, amp_source().0.as_ref(), &mut world);

    let acc = world.get::<RampingDamageAccumulator>(entity);
    assert!(
        acc.is_some(),
        "zero-valued accumulator should still be kept when stack is non-empty",
    );
    assert_eq!(acc.unwrap().0, OrderedFloat(0.0));
}
