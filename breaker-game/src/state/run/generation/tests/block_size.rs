//! Group C — `BlockSize` enum (#11–15)

use std::collections::HashSet;

use crate::state::run::generation::*;

// ── Behavior #11: BlockSize has 8 unit variants with derives ──────────────────

#[test]
fn block_size_has_eight_unit_variants_with_correct_derives() {
    use BlockSize::*;
    let all = [S4x3, S4x4, S6x4, S6x6, S8x4, S8x6, S10x5, S10x8];
    assert_eq!(all.len(), 8);

    // Eq
    assert_eq!(BlockSize::S4x3, BlockSize::S4x3);
    assert_ne!(BlockSize::S4x3, BlockSize::S6x4);

    // Hash + Eq: all 8 variants are distinct
    let set: HashSet<BlockSize> = all.iter().copied().collect();
    assert_eq!(
        set.len(),
        8,
        "all BlockSize variants must be distinct under Hash+Eq"
    );
}

// ── Behavior #12: BlockSize::cols() returns correct column count ──────────────

/// Proves `cols()` is callable in a const context — compile-time check for behavior #12.
const COLS_CONST_PROOF: u32 = BlockSize::S4x3.cols();

#[test]
fn block_size_cols_returns_correct_value_for_every_variant() {
    // Verify the const is accessible (proves constness at compile time)
    assert_eq!(COLS_CONST_PROOF, BlockSize::S4x3.cols());

    assert_eq!(BlockSize::S4x3.cols(), 4);
    assert_eq!(BlockSize::S4x4.cols(), 4);
    assert_eq!(BlockSize::S6x4.cols(), 6);
    assert_eq!(BlockSize::S6x6.cols(), 6);
    assert_eq!(BlockSize::S8x4.cols(), 8);
    assert_eq!(BlockSize::S8x6.cols(), 8);
    assert_eq!(BlockSize::S10x5.cols(), 10);
    assert_eq!(BlockSize::S10x8.cols(), 10);
}

// ── Behavior #13: BlockSize::rows() returns correct row count ────────────────

/// Proves `rows()` is callable in a const context — compile-time check for behavior #13.
const ROWS_CONST_PROOF: u32 = BlockSize::S4x3.rows();

#[test]
fn block_size_rows_returns_correct_value_for_every_variant() {
    // Verify the const is accessible (proves constness at compile time)
    assert_eq!(ROWS_CONST_PROOF, BlockSize::S4x3.rows());

    assert_eq!(BlockSize::S4x3.rows(), 3);
    assert_eq!(BlockSize::S4x4.rows(), 4);
    assert_eq!(BlockSize::S6x4.rows(), 4);
    assert_eq!(BlockSize::S6x6.rows(), 6);
    assert_eq!(BlockSize::S8x4.rows(), 4);
    assert_eq!(BlockSize::S8x6.rows(), 6);
    assert_eq!(BlockSize::S10x5.rows(), 5);
    assert_eq!(BlockSize::S10x8.rows(), 8);
}

// ── Behavior #14: BlockSize::ALL lists every variant exactly once ─────────────

#[test]
fn block_size_all_lists_every_variant_in_declaration_order() {
    use BlockSize::*;
    assert_eq!(
        BlockSize::ALL,
        &[S4x3, S4x4, S6x4, S6x6, S8x4, S8x6, S10x5, S10x8],
    );
    assert_eq!(BlockSize::ALL.len(), 8);

    // No duplicates
    let set: HashSet<BlockSize> = BlockSize::ALL.iter().copied().collect();
    assert_eq!(set.len(), 8, "BlockSize::ALL must not contain duplicates");
}

// ── Behavior #15: BlockSize deserializes from bare RON tokens ────────────────

#[test]
fn block_size_deserializes_from_bare_ron_tokens() {
    let s4x3: BlockSize = ron::de::from_str("S4x3").expect("S4x3 should parse");
    assert_eq!(s4x3, BlockSize::S4x3);

    let s10x8: BlockSize = ron::de::from_str("S10x8").expect("S10x8 should parse");
    assert_eq!(s10x8, BlockSize::S10x8);

    // Unknown size rejected
    let err = ron::de::from_str::<BlockSize>("S5x5");
    assert!(err.is_err(), "S5x5 (unknown size) must be rejected");

    // Lowercase rejected (case-sensitive)
    let err2 = ron::de::from_str::<BlockSize>("s4x3");
    assert!(err2.is_err(), "s4x3 (lowercase) must be rejected");
}
