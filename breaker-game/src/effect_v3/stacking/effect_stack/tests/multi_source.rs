use ordered_float::OrderedFloat;

use super::super::component::*;
use crate::{
    chips::definition::Rarity,
    effect_v3::effects::{PiercingConfig, SpeedBoostConfig},
    prelude::{SourceId, SourceIdExt},
};

fn assert_f32_eq(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-5,
        "expected {expected}, got {actual}"
    );
}

// Builder-format `SourceId` fixtures — replace arbitrary string literals with
// canonical `chip:<template>:<rarity>` keys so tests demonstrate the shape
// production code uses.

fn overclock_source() -> SourceId {
    SourceId::chip("Overclock").rarity(Rarity::Common).build()
}

fn chrono_passive_source() -> SourceId {
    SourceId::chip("ChronoPassive")
        .rarity(Rarity::Common)
        .build()
}

fn feedback_loop_source() -> SourceId {
    SourceId::chip("FeedbackLoop")
        .rarity(Rarity::Common)
        .build()
}

fn amp_source() -> SourceId {
    SourceId::chip("Amp").rarity(Rarity::Common).build()
}

fn nonexistent_source() -> SourceId {
    SourceId::chip("Nonexistent").rarity(Rarity::Common).build()
}

fn anything_source() -> SourceId {
    SourceId::chip("Anything").rarity(Rarity::Common).build()
}

fn splinter_source() -> SourceId {
    SourceId::chip("Splinter").rarity(Rarity::Common).build()
}

fn piercing_bolt_source() -> SourceId {
    SourceId::chip("PiercingBolt")
        .rarity(Rarity::Common)
        .build()
}

// ---------------------------------------------------------------
// Mixed-source integration tests (behaviors 32-33)
// ---------------------------------------------------------------

#[test]
fn multiplicative_aggregate_is_source_agnostic() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    stack.push(
        overclock_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    stack.push(
        chrono_passive_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.2),
        },
    );
    stack.push(
        feedback_loop_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );
    assert_f32_eq(stack.aggregate(), 3.6);
}

// ---------------------------------------------------------------
// retain_by_source tests (behaviors 1-6)
// ---------------------------------------------------------------

#[test]
fn retain_by_source_removes_all_entries_matching_source() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    stack.push(
        overclock_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    stack.push(
        feedback_loop_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );
    stack.push(
        overclock_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.3),
        },
    );

    stack.retain_by_source(&overclock_source());

    assert_eq!(stack.len(), 1);
    let entries: Vec<_> = stack.iter().collect();
    assert_eq!(entries[0].0, feedback_loop_source());
    assert_f32_eq(stack.aggregate(), 2.0);
}

#[test]
fn retain_by_source_with_no_matching_source_is_noop() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    stack.push(
        amp_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );
    stack.push(
        feedback_loop_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );

    stack.retain_by_source(&nonexistent_source());

    assert_eq!(stack.len(), 2);
    assert_f32_eq(stack.aggregate(), 3.0);
}

#[test]
fn retain_by_source_on_empty_stack_is_noop() {
    let mut stack = EffectStack::<PiercingConfig>::default();

    stack.retain_by_source(&anything_source());

    assert!(stack.is_empty());
    assert_eq!(stack.len(), 0);
}

#[test]
fn retain_by_source_removes_all_entries_when_all_share_same_source() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    stack.push(
        overclock_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    stack.push(
        overclock_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );
    stack.push(
        overclock_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.3),
        },
    );

    stack.retain_by_source(&overclock_source());

    assert!(stack.is_empty());
    assert_eq!(stack.len(), 0);
    assert_f32_eq(stack.aggregate(), 1.0);
}

#[test]
fn retain_by_source_preserves_insertion_order_of_surviving_entries() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    stack.push(
        amp_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );
    stack.push(
        overclock_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    stack.push(
        amp_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(3.0),
        },
    );
    stack.push(
        feedback_loop_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.2),
        },
    );

    stack.retain_by_source(&amp_source());

    assert_eq!(stack.len(), 2);
    let entries: Vec<_> = stack.iter().collect();
    assert_eq!(entries[0].0, overclock_source());
    assert_eq!(entries[0].1.multiplier, OrderedFloat(1.5));
    assert_eq!(entries[1].0, feedback_loop_source());
    assert_eq!(entries[1].1.multiplier, OrderedFloat(1.2));
}

#[test]
fn retain_by_source_with_empty_string_matches_empty_string_entries() {
    // The empty-source case stays as `SourceId::from("")` because the builder
    // does not produce empty `SourceId` strings — this test pins the
    // mechanics edge case (empty-source entries can be retained-out).
    let mut stack = EffectStack::<PiercingConfig>::default();
    stack.push(SourceId::from(""), PiercingConfig { charges: 1 });
    stack.push(splinter_source(), PiercingConfig { charges: 3 });

    stack.retain_by_source(&SourceId::from(""));

    assert_eq!(stack.len(), 1);
    let entries: Vec<_> = stack.iter().collect();
    assert_eq!(entries[0].0, splinter_source());
    assert_eq!(entries[0].1.charges, 3);
}

// ---------------------------------------------------------------
// Mixed-source integration tests (behaviors 32-33)
// ---------------------------------------------------------------

#[test]
fn additive_aggregate_is_source_agnostic() {
    let mut stack = EffectStack::<PiercingConfig>::default();
    stack.push(splinter_source(), PiercingConfig { charges: 2 });
    stack.push(piercing_bolt_source(), PiercingConfig { charges: 5 });
    assert_f32_eq(stack.aggregate(), 7.0);
}
