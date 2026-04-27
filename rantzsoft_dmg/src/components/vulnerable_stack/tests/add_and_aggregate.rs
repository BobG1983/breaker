use super::{super::system::VulnerableStack, helpers::assert_f32_eq};
use crate::SourceId;

// ── Behavior 68: `add` appends a single `(SourceId, f32)` entry ──

#[test]
fn add_appends_single_entry_to_persistent() {
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("mark:fragility"), 1.5);
    assert_f32_eq(stack.aggregate_persistent(), 1.5);
    assert!(!stack.is_empty());
}

#[test]
fn add_on_default_stack_does_not_panic() {
    // Edge case for Behavior 68: calling `add` on a default-constructed
    // stack must succeed without panicking.
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("mark:fragility"), 1.0);
    assert!(!stack.is_empty());
}

// ── Behavior 69: same source added five times produces five entries ──

#[test]
fn same_source_added_five_times_aggregates_to_mult_pow_five() {
    let mut stack = VulnerableStack::default();
    for _ in 0..5 {
        stack.add(SourceId::from("mark:fragility"), 3.0);
    }
    // Pins the Vec semantic: if a future implementer swaps to
    // `HashMap<SourceId, f32>`, this aggregate would collapse to 3.0.
    // 3.0_f32.powi(5) == 243.0.
    assert_f32_eq(stack.aggregate_persistent(), 243.0);
}

#[test]
fn interleaved_sources_produce_product_of_all_entries() {
    // Edge case for Behavior 69: interleaved sources do not cross-collapse.
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("m:a"), 2.0);
    stack.add(SourceId::from("m:b"), 1.25);
    stack.add(SourceId::from("m:a"), 2.0);
    assert_f32_eq(stack.aggregate_persistent(), 5.0);
}

// ── Behavior 70: mixed sources and mixed multipliers multiply all entries ──

#[test]
fn mixed_sources_and_multipliers_multiply_all_entries() {
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("m:a"), 1.5);
    stack.add(SourceId::from("m:b"), 2.0);
    stack.add(SourceId::from("m:c"), 4.0);
    // 1.5 * 2.0 * 4.0 = 12.0.
    assert_f32_eq(stack.aggregate_persistent(), 12.0);
}

#[test]
fn aggregate_is_order_independent() {
    // Edge case for Behavior 70: order of insertion does not change the
    // aggregate (multiplication is commutative).
    let mut forward = VulnerableStack::default();
    forward.add(SourceId::from("m:a"), 1.5);
    forward.add(SourceId::from("m:b"), 2.0);
    forward.add(SourceId::from("m:c"), 4.0);

    let mut reverse = VulnerableStack::default();
    reverse.add(SourceId::from("m:c"), 4.0);
    reverse.add(SourceId::from("m:b"), 2.0);
    reverse.add(SourceId::from("m:a"), 1.5);

    // Assert against a concrete expected value (1.5 * 2.0 * 4.0 = 12.0)
    // on BOTH sides independently. Comparing forward vs reverse directly
    // would pass trivially against the RED stub (both return 0.0), so we
    // pin each side to 12.0 to keep the RED-gate signal meaningful.
    assert_f32_eq(forward.aggregate_persistent(), 12.0);
    assert_f32_eq(reverse.aggregate_persistent(), 12.0);
}
