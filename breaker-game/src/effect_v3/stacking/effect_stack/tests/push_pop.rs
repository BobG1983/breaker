use ordered_float::OrderedFloat;

use super::super::component::*;
use crate::effect_v3::effects::{DamageBoostConfig, PiercingConfig, SpeedBoostConfig};

// ---------------------------------------------------------------
// Push tests (behaviors 1-3)
// ---------------------------------------------------------------

#[test]
fn push_adds_single_entry_to_empty_stack() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    stack.push(
        "overclock".to_string(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    assert_eq!(stack.len(), 1);
    assert!(!stack.is_empty());
}

#[test]
fn push_succeeds_with_empty_source_name() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    stack.push(
        String::new(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    assert_eq!(stack.len(), 1);
}

#[test]
fn push_appends_multiple_entries_preserving_insertion_order() {
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
    stack.push(
        "amp".into(),
        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );
    assert_eq!(stack.len(), 3);

    let entries: Vec<_> = stack.iter().collect();
    assert_eq!(entries[0].0, "amp");
    assert_eq!(entries[0].1.multiplier, OrderedFloat(2.0));
    assert_eq!(entries[1].0, "feedback_loop");
    assert_eq!(entries[1].1.multiplier, OrderedFloat(1.5));
    assert_eq!(entries[2].0, "amp");
    assert_eq!(entries[2].1.multiplier, OrderedFloat(2.0));
}

#[test]
fn push_allows_duplicate_source_config_pairs() {
    let mut stack = EffectStack::<DamageBoostConfig>::default();
    let config = DamageBoostConfig {
        multiplier: OrderedFloat(2.0),
    };
    stack.push("amp".into(), config.clone());
    stack.push("amp".into(), config);
    assert_eq!(stack.len(), 2);
}

#[test]
fn push_allows_same_source_with_different_configs() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    stack.push(
        "overclock".into(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.2),
        },
    );
    stack.push(
        "overclock".into(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    assert_eq!(stack.len(), 2);

    let entries: Vec<_> = stack.iter().collect();
    assert_eq!(entries[0].1.multiplier, OrderedFloat(1.2));
    assert_eq!(entries[1].1.multiplier, OrderedFloat(1.5));
}

// ---------------------------------------------------------------
// Remove tests (behaviors 4-8)
// ---------------------------------------------------------------

#[test]
fn remove_finds_and_removes_first_exact_match() {
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
    stack.push(
        "amp".into(),
        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );

    stack.remove(
        "amp",
        &DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );

    assert_eq!(stack.len(), 2);

    let entries: Vec<_> = stack.iter().collect();
    assert_eq!(entries[0].0, "feedback_loop");
    assert_eq!(entries[0].1.multiplier, OrderedFloat(1.5));
    assert_eq!(entries[1].0, "amp");
    assert_eq!(entries[1].1.multiplier, OrderedFloat(2.0));
}

#[test]
fn remove_second_call_removes_remaining_match() {
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
    stack.push(
        "amp".into(),
        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );

    let target = DamageBoostConfig {
        multiplier: OrderedFloat(2.0),
    };
    stack.remove("amp", &target);
    stack.remove("amp", &target);

    assert_eq!(stack.len(), 1);
    let entries: Vec<_> = stack.iter().collect();
    assert_eq!(entries[0].0, "feedback_loop");
}

#[test]
fn remove_does_nothing_when_source_matches_but_config_differs() {
    let mut stack = EffectStack::<DamageBoostConfig>::default();
    stack.push(
        "amp".into(),
        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );

    stack.remove(
        "amp",
        &DamageBoostConfig {
            multiplier: OrderedFloat(3.0),
        },
    );

    assert_eq!(stack.len(), 1);
}

#[test]
fn remove_does_nothing_when_config_matches_but_source_differs() {
    let mut stack = EffectStack::<DamageBoostConfig>::default();
    stack.push(
        "amp".into(),
        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );

    stack.remove(
        "overclock",
        &DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );

    assert_eq!(stack.len(), 1);
}

#[test]
fn remove_on_empty_stack_is_no_op() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    stack.remove(
        "anything",
        &SpeedBoostConfig {
            multiplier: OrderedFloat(1.0),
        },
    );
    assert_eq!(stack.len(), 0);
    assert!(stack.is_empty());
}

#[test]
fn remove_only_removes_first_match_when_multiple_identical_entries_exist() {
    let mut stack = EffectStack::<PiercingConfig>::default();
    let config = PiercingConfig { charges: 2 };
    stack.push("amp".into(), config.clone());
    stack.push("amp".into(), config.clone());
    stack.push("amp".into(), config);

    stack.remove("amp", &PiercingConfig { charges: 2 });

    assert_eq!(stack.len(), 2);
}

// ---------------------------------------------------------------
// is_empty / len / Default tests (behaviors 9-10)
// ---------------------------------------------------------------

#[test]
fn default_stack_is_empty() {
    let stack = EffectStack::<SpeedBoostConfig>::default();
    assert!(stack.is_empty());
    assert_eq!(stack.len(), 0);
}

#[test]
fn is_empty_returns_false_after_push_true_after_removing_all() {
    let mut stack = EffectStack::<PiercingConfig>::default();
    stack.push("splinter".into(), PiercingConfig { charges: 1 });
    assert!(!stack.is_empty());
    assert_eq!(stack.len(), 1);

    stack.remove("splinter", &PiercingConfig { charges: 1 });
    assert!(stack.is_empty());
    assert_eq!(stack.len(), 0);
}

// ---------------------------------------------------------------
// Iteration tests (behavior 11)
// ---------------------------------------------------------------

#[test]
fn iter_yields_references_in_insertion_order() {
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

    let entries: Vec<_> = stack.iter().collect();
    assert_eq!(entries.len(), 2);
    assert_eq!(
        entries[0],
        &(
            "overclock".to_string(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }
        )
    );
    assert_eq!(
        entries[1],
        &(
            "feedback_loop".to_string(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(2.0),
            }
        )
    );
}

#[test]
fn iter_on_empty_stack_yields_zero_items() {
    let stack = EffectStack::<SpeedBoostConfig>::default();
    assert_eq!(stack.iter().count(), 0);
}
