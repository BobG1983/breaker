//! Group B — `CellConstraint` enum (#4–10)

use crate::state::run::generation::*;

// ── Behavior #4: CellConstraint variants and derives ─────────────────────────

#[test]
fn cell_constraint_has_four_variants_with_correct_derives() {
    // Construct all variants to confirm they compile; exercise PartialEq on each
    let must_include =
        CellConstraint::MustInclude(vec![BehaviorKind::Armored, BehaviorKind::Volatile]);
    let must_not_include = CellConstraint::MustNotInclude(vec![BehaviorKind::Portal]);

    assert_eq!(
        must_include,
        CellConstraint::MustInclude(vec![BehaviorKind::Armored, BehaviorKind::Volatile])
    );
    assert_eq!(
        must_not_include,
        CellConstraint::MustNotInclude(vec![BehaviorKind::Portal])
    );
    assert_eq!(CellConstraint::Any, CellConstraint::Any);
    assert_ne!(CellConstraint::Any, CellConstraint::PlainCell);

    // empty MustInclude vec is structurally valid
    assert_eq!(
        CellConstraint::MustInclude(vec![]),
        CellConstraint::MustInclude(vec![]),
    );
}

// ── Behavior #5: CellConstraint::Any deserializes from bare token ─────────────

#[test]
fn cell_constraint_any_deserializes_from_bare_token() {
    let result: CellConstraint = ron::de::from_str("Any").expect("Any should parse");
    assert_eq!(result, CellConstraint::Any);

    // Surrounding whitespace handled by ron
    let result2: CellConstraint =
        ron::de::from_str("  Any  ").expect("Any with whitespace should parse");
    assert_eq!(result2, CellConstraint::Any);
}

// ── Behavior #6: MustInclude deserializes from list of BehaviorKind tokens ────

#[test]
fn cell_constraint_must_include_deserializes_from_list() {
    let result: CellConstraint = ron::de::from_str("MustInclude([Armored, Volatile])")
        .expect("MustInclude list should parse");
    assert_eq!(
        result,
        CellConstraint::MustInclude(vec![BehaviorKind::Armored, BehaviorKind::Volatile]),
    );

    // Single element
    let single: CellConstraint =
        ron::de::from_str("MustInclude([Armored])").expect("single-element list should parse");
    assert_eq!(
        single,
        CellConstraint::MustInclude(vec![BehaviorKind::Armored]),
    );

    // Empty list
    let empty: CellConstraint =
        ron::de::from_str("MustInclude([])").expect("empty list should parse");
    assert_eq!(empty, CellConstraint::MustInclude(vec![]));
}

// ── Behavior #7: MustNotInclude (list form) deserializes from list ────────────

#[test]
fn cell_constraint_must_not_include_list_form_deserializes() {
    let result: CellConstraint = ron::de::from_str("MustNotInclude([Portal, Magnetic])")
        .expect("MustNotInclude list should parse");
    assert_eq!(
        result,
        CellConstraint::MustNotInclude(vec![BehaviorKind::Portal, BehaviorKind::Magnetic]),
    );

    // Single element
    let single: CellConstraint =
        ron::de::from_str("MustNotInclude([Phantom])").expect("single-element should parse");
    assert_eq!(
        single,
        CellConstraint::MustNotInclude(vec![BehaviorKind::Phantom]),
    );
}

// ── Behavior #8: PlainCell deserializes from bare token ───────────────────────

#[test]
fn cell_constraint_plain_cell_deserializes_from_bare_token() {
    let result: CellConstraint = ron::de::from_str("PlainCell").expect("PlainCell should parse");
    assert_eq!(result, CellConstraint::PlainCell);
}

// ── Behavior #9: MustNotInclude(Any) shorthand deserializes to PlainCell ──────

#[test]
fn must_not_include_any_shorthand_deserializes_to_plain_cell() {
    let result: CellConstraint =
        ron::de::from_str("MustNotInclude(Any)").expect("MustNotInclude(Any) should parse");
    assert_eq!(
        result,
        CellConstraint::PlainCell,
        "MustNotInclude(Any) must deserialize to PlainCell, not MustNotInclude(vec![])",
    );

    // Equivalence with bare PlainCell form
    let plain: CellConstraint = ron::de::from_str("PlainCell").expect("PlainCell should parse");
    assert_eq!(
        result, plain,
        "MustNotInclude(Any) and PlainCell must parse to equal values",
    );
}

// ── Behavior #10: Unknown BehaviorKind inside MustInclude is rejected ─────────

#[test]
fn must_include_with_unknown_behavior_kind_is_rejected() {
    let err = ron::de::from_str::<CellConstraint>("MustInclude([Bogus])");
    assert!(
        err.is_err(),
        "MustInclude([Bogus]) with unknown variant must be rejected"
    );

    // Case-sensitive — lowercase variant name rejected
    let err2 = ron::de::from_str::<CellConstraint>("MustInclude([armored])");
    assert!(
        err2.is_err(),
        "MustInclude([armored]) with lowercase variant must be rejected (serde is case-sensitive)",
    );
}
