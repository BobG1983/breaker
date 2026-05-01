use super::system::{active_invariant_kinds, is_fixed_update_checker};
use crate::types::{InvariantKind, ScenarioDefinition};

// -----------------------------------------------------------------
// is_fixed_update_checker — classification function
// -----------------------------------------------------------------

/// Behavior 1: `is_fixed_update_checker` returns `true` for FixedUpdate-batch
/// variants. Test a representative from each of the three original batches.
#[test]
fn is_fixed_update_checker_returns_true_for_fixed_update_kinds() {
    // checkers_a representative
    assert!(
        is_fixed_update_checker(InvariantKind::BoltInBounds),
        "BoltInBounds should be a FixedUpdate checker"
    );
    // checkers_b representative
    assert!(
        is_fixed_update_checker(InvariantKind::NoEntityLeaks),
        "NoEntityLeaks should be a FixedUpdate checker"
    );
    // checkers_c representative
    assert!(
        is_fixed_update_checker(InvariantKind::AabbMatchesEntityDimensions),
        "AabbMatchesEntityDimensions should be a FixedUpdate checker"
    );
}

/// Behavior 2: `is_fixed_update_checker` returns `false` for `ChipOfferExpected`.
#[test]
fn is_fixed_update_checker_returns_false_for_chip_offer_expected() {
    assert!(
        !is_fixed_update_checker(InvariantKind::ChipOfferExpected),
        "ChipOfferExpected runs on Update, not FixedUpdate"
    );
}

/// Behavior 3: Exhaustive coverage — exactly 32 variants return `true`,
/// exactly 1 returns `false` (`ChipOfferExpected`), total = 33 = `ALL.len()`.
#[test]
fn is_fixed_update_checker_covers_every_invariant_kind_variant() {
    let total = InvariantKind::ALL.len();
    assert_eq!(total, 33, "expected 33 InvariantKind variants in ALL");

    let fixed_update_count = InvariantKind::ALL
        .iter()
        .filter(|k| is_fixed_update_checker(**k))
        .count();
    let non_fixed_update_count = total - fixed_update_count;

    assert_eq!(
        fixed_update_count, 32,
        "expected exactly 32 FixedUpdate checker kinds, got {fixed_update_count}"
    );
    assert_eq!(
        non_fixed_update_count, 1,
        "expected exactly 1 non-FixedUpdate kind, got {non_fixed_update_count}"
    );

    // The sole non-FixedUpdate variant must be ChipOfferExpected
    let non_fixed: Vec<_> = InvariantKind::ALL
        .iter()
        .filter(|k| !is_fixed_update_checker(**k))
        .collect();
    assert_eq!(
        non_fixed,
        vec![&InvariantKind::ChipOfferExpected],
        "the only non-FixedUpdate kind should be ChipOfferExpected"
    );
}

// -----------------------------------------------------------------
// active_invariant_kinds — set computation
// -----------------------------------------------------------------

/// Behavior 4: Empty `disallowed_failures` and None `allowed_failures` returns
/// all 32 `FixedUpdate` kinds.
#[test]
fn active_invariant_kinds_returns_all_32_when_both_lists_empty() {
    let def = ScenarioDefinition {
        disallowed_failures: vec![],
        allowed_failures: None,
        ..Default::default()
    };
    let active = active_invariant_kinds(&def);
    assert_eq!(
        active.len(),
        32,
        "expected 32 active kinds when both lists empty, got {}",
        active.len()
    );
    assert!(
        !active.contains(&InvariantKind::ChipOfferExpected),
        "fallback set must not contain ChipOfferExpected"
    );
}

/// Behavior 4 edge case: Empty vec with Some(vec![]) also returns all 32.
#[test]
fn active_invariant_kinds_returns_all_32_when_allowed_is_empty_some() {
    let def = ScenarioDefinition {
        disallowed_failures: vec![],
        allowed_failures: Some(vec![]),
        ..Default::default()
    };
    let active = active_invariant_kinds(&def);
    assert_eq!(
        active.len(),
        32,
        "expected 32 active kinds when both lists effectively empty, got {}",
        active.len()
    );
    assert!(
        !active.contains(&InvariantKind::ChipOfferExpected),
        "fallback set must not contain ChipOfferExpected"
    );
}

/// Behavior 5: Single `disallowed_failures` entry returns only that kind.
#[test]
fn active_invariant_kinds_single_disallowed_returns_only_that_kind() {
    let def = ScenarioDefinition {
        disallowed_failures: vec![InvariantKind::NoNaN],
        allowed_failures: None,
        ..Default::default()
    };
    let active = active_invariant_kinds(&def);
    assert_eq!(
        active.len(),
        1,
        "expected 1 active kind, got {}",
        active.len()
    );
    assert!(
        active.contains(&InvariantKind::NoNaN),
        "active set must contain NoNaN"
    );
}

/// Behavior 6: Multiple `disallowed_failures` entries return their union.
#[test]
fn active_invariant_kinds_multiple_disallowed_returns_union() {
    let def = ScenarioDefinition {
        disallowed_failures: vec![
            InvariantKind::BoltInBounds,
            InvariantKind::BreakerInBounds,
            InvariantKind::NoNaN,
        ],
        allowed_failures: None,
        ..Default::default()
    };
    let active = active_invariant_kinds(&def);
    assert_eq!(
        active.len(),
        3,
        "expected 3 active kinds, got {}",
        active.len()
    );
    assert!(active.contains(&InvariantKind::BoltInBounds));
    assert!(active.contains(&InvariantKind::BreakerInBounds));
    assert!(active.contains(&InvariantKind::NoNaN));
}

/// Behavior 7: `allowed_failures` entries are included in active set.
/// Edge case: same entry in both lists produces no duplication.
#[test]
fn active_invariant_kinds_allowed_failures_included_no_duplication() {
    let def = ScenarioDefinition {
        disallowed_failures: vec![InvariantKind::BoltInBounds],
        allowed_failures: Some(vec![InvariantKind::BoltInBounds]),
        ..Default::default()
    };
    let active = active_invariant_kinds(&def);
    assert_eq!(
        active.len(),
        1,
        "expected 1 active kind (no duplication), got {}",
        active.len()
    );
    assert!(active.contains(&InvariantKind::BoltInBounds));
}

/// Behavior 8: Union of `disallowed_failures` and `allowed_failures`.
#[test]
fn active_invariant_kinds_union_of_disallowed_and_allowed() {
    let def = ScenarioDefinition {
        disallowed_failures: vec![InvariantKind::BoltInBounds],
        allowed_failures: Some(vec![InvariantKind::NoNaN]),
        ..Default::default()
    };
    let active = active_invariant_kinds(&def);
    assert_eq!(
        active.len(),
        2,
        "expected 2 active kinds, got {}",
        active.len()
    );
    assert!(active.contains(&InvariantKind::BoltInBounds));
    assert!(active.contains(&InvariantKind::NoNaN));
}

/// Behavior 9: `ChipOfferExpected` in `disallowed_failures` is filtered out;
/// only non-`ChipOfferExpected` kinds remain.
#[test]
fn active_invariant_kinds_filters_out_chip_offer_expected() {
    let def = ScenarioDefinition {
        disallowed_failures: vec![
            InvariantKind::ChipOfferExpected,
            InvariantKind::BoltInBounds,
        ],
        allowed_failures: None,
        ..Default::default()
    };
    let active = active_invariant_kinds(&def);
    assert_eq!(
        active.len(),
        1,
        "expected 1 active kind after filtering ChipOfferExpected, got {}",
        active.len()
    );
    assert!(active.contains(&InvariantKind::BoltInBounds));
    assert!(
        !active.contains(&InvariantKind::ChipOfferExpected),
        "ChipOfferExpected must be filtered out"
    );
}

/// Behavior 9 edge case: `ChipOfferExpected` as the only entry triggers
/// fallback to all 32 `FixedUpdate` kinds.
#[test]
fn active_invariant_kinds_chip_offer_expected_only_triggers_fallback() {
    let def = ScenarioDefinition {
        disallowed_failures: vec![InvariantKind::ChipOfferExpected],
        allowed_failures: None,
        ..Default::default()
    };
    let active = active_invariant_kinds(&def);
    assert_eq!(
        active.len(),
        32,
        "expected 32 active kinds (fallback) when only ChipOfferExpected is listed, got {}",
        active.len()
    );
    assert!(
        !active.contains(&InvariantKind::ChipOfferExpected),
        "fallback set must not contain ChipOfferExpected"
    );
}
