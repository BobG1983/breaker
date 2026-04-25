use ordered_float::OrderedFloat;

use super::super::component::*;
use crate::{
    chips::definition::Rarity,
    effect_v3::effects::{PiercingConfig, SpeedBoostConfig},
    prelude::{SourceId, SourceIdExt},
};

// Builder-format `SourceId` fixtures — replace arbitrary string literals with
// canonical `chip:<template>:<rarity>` keys so tests demonstrate the shape
// production code uses.

fn overclock_source() -> SourceId {
    SourceId::chip("Overclock").rarity(Rarity::Common).build()
}

fn feedback_loop_source() -> SourceId {
    SourceId::chip("FeedbackLoop")
        .rarity(Rarity::Common)
        .build()
}

fn amp_source() -> SourceId {
    SourceId::chip("Amp").rarity(Rarity::Common).build()
}

fn anything_source() -> SourceId {
    SourceId::chip("Anything").rarity(Rarity::Common).build()
}

fn splinter_source() -> SourceId {
    SourceId::chip("Splinter").rarity(Rarity::Common).build()
}

// ---------------------------------------------------------------
// Push tests (behaviors 1-3)
// ---------------------------------------------------------------

#[test]
fn push_adds_single_entry_to_empty_stack() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    stack.push(
        overclock_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    assert_eq!(stack.len(), 1);
    assert!(!stack.is_empty());
}

#[test]
fn push_succeeds_with_empty_source_name() {
    // The empty-source case stays as `SourceId::from("")` because the builder
    // does not produce empty `SourceId` strings — this test pins the
    // mechanics edge case (empty-string source is permissible).
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    stack.push(
        SourceId::from(""),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    assert_eq!(stack.len(), 1);
}

#[test]
fn push_appends_multiple_entries_preserving_insertion_order() {
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
    stack.push(
        amp_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );
    assert_eq!(stack.len(), 3);

    let entries: Vec<_> = stack.iter().collect();
    assert_eq!(entries[0].0, amp_source());
    assert_eq!(entries[0].1.multiplier, OrderedFloat(2.0));
    assert_eq!(entries[1].0, feedback_loop_source());
    assert_eq!(entries[1].1.multiplier, OrderedFloat(1.5));
    assert_eq!(entries[2].0, amp_source());
    assert_eq!(entries[2].1.multiplier, OrderedFloat(2.0));
}

#[test]
fn push_allows_duplicate_source_config_pairs() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    let config = SpeedBoostConfig {
        multiplier: OrderedFloat(2.0),
    };
    stack.push(amp_source(), config.clone());
    stack.push(amp_source(), config);
    assert_eq!(stack.len(), 2);
}

#[test]
fn push_allows_same_source_with_different_configs() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    stack.push(
        overclock_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.2),
        },
    );
    stack.push(
        overclock_source(),
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
    stack.push(
        amp_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );

    stack.remove(
        &amp_source(),
        &SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );

    assert_eq!(stack.len(), 2);

    let entries: Vec<_> = stack.iter().collect();
    assert_eq!(entries[0].0, feedback_loop_source());
    assert_eq!(entries[0].1.multiplier, OrderedFloat(1.5));
    assert_eq!(entries[1].0, amp_source());
    assert_eq!(entries[1].1.multiplier, OrderedFloat(2.0));
}

#[test]
fn remove_second_call_removes_remaining_match() {
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
    stack.push(
        amp_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );

    let target = SpeedBoostConfig {
        multiplier: OrderedFloat(2.0),
    };
    stack.remove(&amp_source(), &target);
    stack.remove(&amp_source(), &target);

    assert_eq!(stack.len(), 1);
    let entries: Vec<_> = stack.iter().collect();
    assert_eq!(entries[0].0, feedback_loop_source());
}

#[test]
fn remove_does_nothing_when_source_matches_but_config_differs() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    stack.push(
        amp_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );

    stack.remove(
        &amp_source(),
        &SpeedBoostConfig {
            multiplier: OrderedFloat(3.0),
        },
    );

    assert_eq!(stack.len(), 1);
}

#[test]
fn remove_does_nothing_when_config_matches_but_source_differs() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    stack.push(
        amp_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );

    stack.remove(
        &overclock_source(),
        &SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );

    assert_eq!(stack.len(), 1);
}

#[test]
fn remove_on_empty_stack_is_no_op() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    stack.remove(
        &anything_source(),
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
    stack.push(amp_source(), config.clone());
    stack.push(amp_source(), config.clone());
    stack.push(amp_source(), config);

    stack.remove(&amp_source(), &PiercingConfig { charges: 2 });

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
    stack.push(splinter_source(), PiercingConfig { charges: 1 });
    assert!(!stack.is_empty());
    assert_eq!(stack.len(), 1);

    stack.remove(&splinter_source(), &PiercingConfig { charges: 1 });
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

    let entries: Vec<_> = stack.iter().collect();
    assert_eq!(entries.len(), 2);
    assert_eq!(
        entries[0],
        &(
            overclock_source(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }
        )
    );
    assert_eq!(
        entries[1],
        &(
            feedback_loop_source(),
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
