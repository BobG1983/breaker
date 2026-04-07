//! Tests for `StreamingPool` — a pure count-based state machine for managing
//! concurrent scenario dispatch.

use crate::runner::streaming::StreamingPool;

// =========================================================================
// Construction — behaviors 1-3
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 1: new pool with valid parameters reports correct initial state
// -------------------------------------------------------------------------

#[test]
fn new_pool_reports_zero_active_and_completed() {
    let pool = StreamingPool::new(4, 10);
    assert_eq!(pool.active_count(), 0);
    assert_eq!(pool.completed_count(), 0);
}

#[test]
fn new_pool_reports_total_as_remaining() {
    let pool = StreamingPool::new(4, 10);
    assert_eq!(pool.remaining_count(), 10);
}

#[test]
fn new_pool_can_start_is_true() {
    let pool = StreamingPool::new(4, 10);
    assert!(
        pool.can_start(),
        "pool with capacity and items should allow starting"
    );
}

#[test]
fn new_pool_is_not_done() {
    let pool = StreamingPool::new(4, 10);
    assert!(
        !pool.is_done(),
        "pool with remaining items should not be done"
    );
}

/// Edge case: single-slot, single-item pool.
#[test]
fn new_pool_single_slot_single_item_reports_correct_state() {
    let pool = StreamingPool::new(1, 1);
    assert_eq!(pool.active_count(), 0);
    assert_eq!(pool.completed_count(), 0);
    assert_eq!(pool.remaining_count(), 1);
    assert!(pool.can_start());
    assert!(!pool.is_done());
}

// -------------------------------------------------------------------------
// Behavior 2: new pool clamps max_concurrent of zero to one
// -------------------------------------------------------------------------

#[test]
fn new_pool_clamps_zero_max_concurrent_to_one_allows_start() {
    let pool = StreamingPool::new(0, 5);
    assert!(
        pool.can_start(),
        "max_concurrent=0 should be clamped to 1, allowing one start"
    );
    assert_eq!(pool.remaining_count(), 5);
    assert_eq!(pool.active_count(), 0);
}

/// Edge case: after one start, the clamped single slot is full.
#[test]
fn new_pool_clamped_zero_blocks_after_one_start() {
    let mut pool = StreamingPool::new(0, 5);
    pool.start_next();
    assert!(
        !pool.can_start(),
        "clamped max_concurrent=1 should block after one active item"
    );
}

// -------------------------------------------------------------------------
// Behavior 3: new pool with zero total is immediately done
// -------------------------------------------------------------------------

#[test]
fn new_pool_zero_total_is_immediately_done() {
    let pool = StreamingPool::new(3, 0);
    assert!(
        pool.is_done(),
        "pool with zero total should be done immediately"
    );
    assert!(
        !pool.can_start(),
        "pool with zero total should not allow starts"
    );
    assert_eq!(pool.active_count(), 0);
    assert_eq!(pool.completed_count(), 0);
    assert_eq!(pool.remaining_count(), 0);
}

/// Edge case: both `max_concurrent` and total are zero.
#[test]
fn new_pool_both_zero_is_immediately_done() {
    let pool = StreamingPool::new(0, 0);
    assert!(
        pool.is_done(),
        "pool with zero total should be done immediately"
    );
    assert!(
        !pool.can_start(),
        "pool with zero total should not allow starts"
    );
}

// =========================================================================
// Starting — behaviors 4-7
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 4: start_next returns sequential indices starting from zero
// -------------------------------------------------------------------------

#[test]
fn start_next_returns_sequential_indices() {
    let mut pool = StreamingPool::new(4, 10);
    assert_eq!(pool.start_next(), 0);
    assert_eq!(pool.start_next(), 1);
    assert_eq!(pool.start_next(), 2);
}

/// Edge case: serial pool (`max_concurrent=1`) returns sequential indices
/// across start/complete cycles.
#[test]
fn start_next_sequential_across_complete_cycles() {
    let mut pool = StreamingPool::new(1, 3);
    assert_eq!(pool.start_next(), 0, "first start_next should return 0");
    pool.mark_complete();
    assert_eq!(pool.start_next(), 1, "second start_next should return 1");
}

// -------------------------------------------------------------------------
// Behavior 5: can_start returns false after filling all concurrent slots
// -------------------------------------------------------------------------

#[test]
fn can_start_false_after_filling_all_slots() {
    let mut pool = StreamingPool::new(3, 10);
    pool.start_next();
    pool.start_next();
    pool.start_next();
    assert!(
        !pool.can_start(),
        "all 3 slots filled — can_start should be false"
    );
    assert_eq!(pool.active_count(), 3);
    assert_eq!(pool.remaining_count(), 7);
}

/// Edge case: single-slot pool blocks after one start.
#[test]
fn can_start_false_after_filling_single_slot() {
    let mut pool = StreamingPool::new(1, 5);
    pool.start_next();
    assert!(
        !pool.can_start(),
        "single slot filled — can_start should be false"
    );
}

// -------------------------------------------------------------------------
// Behavior 6: can_start returns false when all items dispatched even with
//             free slots
// -------------------------------------------------------------------------

#[test]
fn can_start_false_when_all_items_dispatched_with_free_slots() {
    let mut pool = StreamingPool::new(5, 3);
    pool.start_next();
    pool.start_next();
    pool.start_next();
    assert!(
        !pool.can_start(),
        "all 3 items dispatched — can_start should be false despite 2 free slots"
    );
    assert_eq!(pool.active_count(), 3);
    assert_eq!(pool.remaining_count(), 0);
}

/// Edge case: 100 slots but only 1 item.
#[test]
fn can_start_false_after_single_item_dispatched_with_many_slots() {
    let mut pool = StreamingPool::new(100, 1);
    pool.start_next();
    assert!(
        !pool.can_start(),
        "single item dispatched — can_start should be false despite 99 free slots"
    );
}

// -------------------------------------------------------------------------
// Behavior 7: start_next increments active_count by one each call
// -------------------------------------------------------------------------

#[test]
fn start_next_increments_active_count() {
    let mut pool = StreamingPool::new(4, 10);
    pool.start_next();
    assert_eq!(
        pool.active_count(),
        1,
        "active_count should be 1 after one start"
    );
    pool.start_next();
    assert_eq!(
        pool.active_count(),
        2,
        "active_count should be 2 after two starts"
    );
}

// =========================================================================
// Completing — behaviors 8-10
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 8: mark_complete decrements active and increments completed
// -------------------------------------------------------------------------

#[test]
fn mark_complete_decrements_active_increments_completed() {
    let mut pool = StreamingPool::new(4, 10);
    pool.start_next();
    pool.start_next();
    pool.start_next();
    pool.mark_complete();
    assert_eq!(pool.active_count(), 2);
    assert_eq!(pool.completed_count(), 1);
}

/// Edge case: start 1, complete 1 — active goes to 0, completed to 1.
#[test]
fn mark_complete_single_item_returns_to_zero_active() {
    let mut pool = StreamingPool::new(4, 10);
    pool.start_next();
    pool.mark_complete();
    assert_eq!(pool.active_count(), 0);
    assert_eq!(pool.completed_count(), 1);
}

// -------------------------------------------------------------------------
// Behavior 9: mark_complete re-opens a slot for can_start when items remain
// -------------------------------------------------------------------------

#[test]
fn mark_complete_reopens_slot_when_items_remain() {
    let mut pool = StreamingPool::new(2, 5);
    pool.start_next();
    pool.start_next();
    assert!(!pool.can_start(), "both slots should be full");
    pool.mark_complete();
    assert!(
        pool.can_start(),
        "one slot freed — can_start should be true"
    );
    assert_eq!(pool.active_count(), 1);
    assert_eq!(pool.remaining_count(), 3);
}

/// Edge case: serial pool start/complete/start cycle returns correct index.
#[test]
fn mark_complete_reopens_single_slot_returns_next_index() {
    let mut pool = StreamingPool::new(1, 2);
    pool.start_next(); // index 0
    assert!(!pool.can_start());
    pool.mark_complete();
    assert!(pool.can_start());
    let idx = pool.start_next();
    assert_eq!(idx, 1, "second start_next should return index 1");
}

// -------------------------------------------------------------------------
// Behavior 10: mark_complete does not re-open slots when all items already
//              dispatched
// -------------------------------------------------------------------------

#[test]
fn mark_complete_does_not_reopen_when_all_dispatched() {
    let mut pool = StreamingPool::new(3, 3);
    pool.start_next();
    pool.start_next();
    pool.start_next();
    pool.mark_complete();
    assert!(
        !pool.can_start(),
        "all items dispatched — completing should not re-open can_start"
    );
    assert_eq!(pool.active_count(), 2);
    assert_eq!(pool.completed_count(), 1);
}

/// Edge case: complete all 3 — `is_done` is true, `can_start` still false.
#[test]
fn mark_complete_all_dispatched_reaches_done() {
    let mut pool = StreamingPool::new(3, 3);
    pool.start_next();
    pool.start_next();
    pool.start_next();
    pool.mark_complete();
    pool.mark_complete();
    pool.mark_complete();
    assert!(!pool.can_start(), "all done — can_start should be false");
    assert!(pool.is_done(), "all 3 completed — is_done should be true");
}

// =========================================================================
// Full drain — behaviors 11-13
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 11: start all then complete all reaches is_done
// -------------------------------------------------------------------------

#[test]
fn start_all_then_complete_all_reaches_done() {
    let mut pool = StreamingPool::new(4, 4);
    for _ in 0..4 {
        pool.start_next();
    }
    for _ in 0..4 {
        pool.mark_complete();
    }
    assert!(pool.is_done());
    assert_eq!(pool.active_count(), 0);
    assert_eq!(pool.completed_count(), 4);
    assert_eq!(pool.remaining_count(), 0);
    assert!(!pool.can_start());
}

/// Edge case: single item pool — start 1, complete 1.
#[test]
fn single_item_pool_start_and_complete_reaches_done() {
    let mut pool = StreamingPool::new(1, 1);
    pool.start_next();
    pool.mark_complete();
    assert!(pool.is_done());
}

// -------------------------------------------------------------------------
// Behavior 12: interleaved start and complete maintains correct counts
// -------------------------------------------------------------------------

#[test]
fn interleaved_start_complete_maintains_correct_counts() {
    let mut pool = StreamingPool::new(2, 4);

    // start(0), start(1)
    let idx0 = pool.start_next();
    let idx1 = pool.start_next();
    assert_eq!(idx0, 0);
    assert_eq!(idx1, 1);

    // Intermediate checkpoint: active=2, completed=0, remaining=2
    assert_eq!(pool.active_count(), 2);
    assert_eq!(pool.completed_count(), 0);
    assert_eq!(pool.remaining_count(), 2);

    // complete one
    pool.mark_complete();

    // Intermediate checkpoint: active=1, completed=1, remaining=2, can_start=true
    assert_eq!(pool.active_count(), 1);
    assert_eq!(pool.completed_count(), 1);
    assert_eq!(pool.remaining_count(), 2);
    assert!(pool.can_start());

    // start(2)
    let idx2 = pool.start_next();
    assert_eq!(idx2, 2);

    // complete two more
    pool.mark_complete();
    pool.mark_complete();

    // start(3)
    let idx3 = pool.start_next();
    assert_eq!(idx3, 3);

    // complete last
    pool.mark_complete();

    assert_eq!(pool.active_count(), 0);
    assert_eq!(pool.completed_count(), 4);
    assert!(pool.is_done());
}

// -------------------------------------------------------------------------
// Behavior 13: interleaved start/complete with max_concurrent=1 (serial)
// -------------------------------------------------------------------------

#[test]
fn serial_execution_interleaved_start_complete() {
    let mut pool = StreamingPool::new(1, 3);

    let idx0 = pool.start_next();
    assert_eq!(idx0, 0);
    assert_eq!(pool.active_count(), 1);
    pool.mark_complete();

    let idx1 = pool.start_next();
    assert_eq!(idx1, 1);
    assert_eq!(pool.active_count(), 1);
    pool.mark_complete();

    let idx2 = pool.start_next();
    assert_eq!(idx2, 2);
    assert_eq!(pool.active_count(), 1);
    pool.mark_complete();

    assert!(pool.is_done());
    assert_eq!(pool.active_count(), 0);
    assert_eq!(pool.completed_count(), 3);
}

/// Edge case: `active_count` never exceeds 1 during serial execution.
#[test]
fn serial_execution_active_count_never_exceeds_one() {
    let mut pool = StreamingPool::new(1, 3);
    let mut max_active: usize = 0;

    for _ in 0..3 {
        pool.start_next();
        max_active = max_active.max(pool.active_count());
        pool.mark_complete();
    }

    assert!(
        max_active <= 1,
        "active_count exceeded 1 during serial execution: max was {max_active}"
    );
}

// =========================================================================
// Edge cases — behaviors 14-15
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 14: max_concurrent >= total means all start immediately
// -------------------------------------------------------------------------

#[test]
fn all_items_start_immediately_when_slots_exceed_total() {
    let mut pool = StreamingPool::new(10, 3);
    pool.start_next();
    pool.start_next();
    pool.start_next();
    assert!(
        !pool.can_start(),
        "all 3 dispatched — can_start should be false"
    );
    assert_eq!(pool.active_count(), 3);
    assert_eq!(pool.remaining_count(), 0);
}

/// Edge case: `usize::MAX` slots, 2 items.
#[test]
fn usize_max_slots_both_items_start() {
    let mut pool = StreamingPool::new(usize::MAX, 2);
    pool.start_next();
    pool.start_next();
    assert!(
        !pool.can_start(),
        "all 2 dispatched — can_start should be false"
    );
}

// -------------------------------------------------------------------------
// Behavior 15: remaining_count decreases with start_next, not mark_complete
// -------------------------------------------------------------------------

#[test]
fn remaining_count_decreases_on_start_not_complete() {
    let mut pool = StreamingPool::new(4, 6);
    assert_eq!(pool.remaining_count(), 6);

    pool.start_next();
    assert_eq!(
        pool.remaining_count(),
        5,
        "remaining should decrease after start_next"
    );

    pool.start_next();
    assert_eq!(
        pool.remaining_count(),
        4,
        "remaining should decrease after second start_next"
    );

    pool.mark_complete();
    assert_eq!(
        pool.remaining_count(),
        4,
        "remaining should NOT change after mark_complete"
    );
}

/// Edge case: start all 6, complete all 6 — remaining stays 0 during
/// completion phase.
#[test]
fn remaining_count_stays_zero_during_completion_phase() {
    let mut pool = StreamingPool::new(6, 6);
    for _ in 0..6 {
        pool.start_next();
    }
    assert_eq!(pool.remaining_count(), 0);

    for _ in 0..6 {
        pool.mark_complete();
        assert_eq!(
            pool.remaining_count(),
            0,
            "remaining should stay 0 during completion phase"
        );
    }
}

// =========================================================================
// Debug assertions — behaviors 16-17
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 16: debug assert fires in start_next when can_start is false
// -------------------------------------------------------------------------

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "can_start")]
fn start_next_panics_when_cannot_start() {
    let mut pool = StreamingPool::new(2, 2);
    pool.start_next();
    pool.start_next();
    // All slots full and all items dispatched — this should panic.
    pool.start_next();
}

// -------------------------------------------------------------------------
// Behavior 17: debug assert fires in mark_complete when active_count is zero
// -------------------------------------------------------------------------

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "active_count")]
fn mark_complete_panics_when_no_active_items() {
    let mut pool = StreamingPool::new(2, 5);
    // No items started — active_count is 0 — this should panic.
    pool.mark_complete();
}
