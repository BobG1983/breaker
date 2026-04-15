use ordered_float::OrderedFloat;

use super::super::component::*;
use crate::effect_v3::effects::{DamageBoostConfig, PiercingConfig, SpeedBoostConfig};

fn assert_f32_eq(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-5,
        "expected {expected}, got {actual}"
    );
}

// ---------------------------------------------------------------
// Mixed-source integration tests (behaviors 32-33)
// ---------------------------------------------------------------

#[test]
fn multiplicative_aggregate_is_source_agnostic() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    stack.push(
        "overclock".into(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    stack.push(
        "chrono_passive".into(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.2),
        },
    );
    stack.push(
        "feedback_loop".into(),
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
        "overclock".into(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    stack.push(
        "feedback_loop".into(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );
    stack.push(
        "overclock".into(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.3),
        },
    );

    stack.retain_by_source("overclock");

    assert_eq!(stack.len(), 1);
    let entries: Vec<_> = stack.iter().collect();
    assert_eq!(entries[0].0, "feedback_loop");
    assert_f32_eq(stack.aggregate(), 2.0);
}

#[test]
fn retain_by_source_with_no_matching_source_is_noop() {
    let mut stack = EffectStack::<DamageBoostConfig>::default();
    stack.push(
        "amp".into(),
        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );
    stack.push(
        "feedback_loop".into(),
        DamageBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );

    stack.retain_by_source("nonexistent");

    assert_eq!(stack.len(), 2);
    assert_f32_eq(stack.aggregate(), 3.0);
}

#[test]
fn retain_by_source_on_empty_stack_is_noop() {
    let mut stack = EffectStack::<PiercingConfig>::default();

    stack.retain_by_source("anything");

    assert!(stack.is_empty());
    assert_eq!(stack.len(), 0);
}

#[test]
fn retain_by_source_removes_all_entries_when_all_share_same_source() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    stack.push(
        "overclock".into(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    stack.push(
        "overclock".into(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );
    stack.push(
        "overclock".into(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.3),
        },
    );

    stack.retain_by_source("overclock");

    assert!(stack.is_empty());
    assert_eq!(stack.len(), 0);
    assert_f32_eq(stack.aggregate(), 1.0);
}

#[test]
fn retain_by_source_preserves_insertion_order_of_surviving_entries() {
    let mut stack = EffectStack::<DamageBoostConfig>::default();
    stack.push(
        "amp".into(),
        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );
    stack.push(
        "overclock".into(),
        DamageBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    stack.push(
        "amp".into(),
        DamageBoostConfig {
            multiplier: OrderedFloat(3.0),
        },
    );
    stack.push(
        "feedback_loop".into(),
        DamageBoostConfig {
            multiplier: OrderedFloat(1.2),
        },
    );

    stack.retain_by_source("amp");

    assert_eq!(stack.len(), 2);
    let entries: Vec<_> = stack.iter().collect();
    assert_eq!(entries[0].0, "overclock");
    assert_eq!(entries[0].1.multiplier, OrderedFloat(1.5));
    assert_eq!(entries[1].0, "feedback_loop");
    assert_eq!(entries[1].1.multiplier, OrderedFloat(1.2));
}

#[test]
fn retain_by_source_with_empty_string_matches_empty_string_entries() {
    let mut stack = EffectStack::<PiercingConfig>::default();
    stack.push(String::new(), PiercingConfig { charges: 1 });
    stack.push("splinter".into(), PiercingConfig { charges: 3 });

    stack.retain_by_source("");

    assert_eq!(stack.len(), 1);
    let entries: Vec<_> = stack.iter().collect();
    assert_eq!(entries[0].0, "splinter");
    assert_eq!(entries[0].1.charges, 3);
}

// ---------------------------------------------------------------
// Mixed-source integration tests (behaviors 32-33)
// ---------------------------------------------------------------

#[test]
fn additive_aggregate_is_source_agnostic() {
    let mut stack = EffectStack::<PiercingConfig>::default();
    stack.push("splinter".into(), PiercingConfig { charges: 2 });
    stack.push("piercing_bolt".into(), PiercingConfig { charges: 5 });
    assert_f32_eq(stack.aggregate(), 7.0);
}
