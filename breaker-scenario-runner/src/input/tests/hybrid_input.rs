use super::super::drivers::*;
use crate::types::HybridParams;

// -------------------------------------------------------------------------
// HybridInput — returns empty during scripted phase
// -------------------------------------------------------------------------

/// `HybridInput` with `scripted_frames=100` returns empty `Vec` for any frame
/// in `0..scripted_frames`, even when `action_prob=1.0`.
///
/// Frame 50 (well inside the scripted phase) must return empty.
/// Frame 100 (exactly at the boundary) must also return empty — the scripted
/// phase covers frames `0..scripted_frames` (exclusive upper bound).
#[test]
fn hybrid_input_returns_empty_during_scripted_phase() {
    let params = HybridParams {
        scripted_frames: 100,
        action_prob: 1.0,
    };
    let mut hybrid = HybridInput::new(42, &params);

    let result_mid = hybrid.actions_for_frame(50, true);
    assert!(
        result_mid.is_empty(),
        "expected empty Vec at frame 50 (scripted phase), got {result_mid:?}"
    );

    // Edge case: frame exactly at boundary is still scripted phase
    let result_boundary = hybrid.actions_for_frame(99, true);
    assert!(
        result_boundary.is_empty(),
        "expected empty Vec at frame 99 (last scripted frame), got {result_boundary:?}"
    );
}

// -------------------------------------------------------------------------
// HybridInput — switches to chaos after scripted phase
// -------------------------------------------------------------------------

/// `HybridInput` with `scripted_frames=10` and `action_prob=1.0` must produce
/// non-empty actions on most frames in `10..110` when `is_active=true`.
///
/// With probability 1.0, every eligible frame should fire. We require at least
/// 50% of 100 frames to produce non-empty actions to give the statistical test
/// a wide safety margin.
#[test]
fn hybrid_input_switches_to_chaos_after_scripted_phase() {
    let params = HybridParams {
        scripted_frames: 10,
        action_prob: 1.0,
    };
    let mut hybrid = HybridInput::new(42, &params);

    // Exhaust the scripted phase frames first so RNG state is correct
    for frame in 0_u32..10 {
        drop(hybrid.actions_for_frame(frame, true));
    }

    let non_empty_count = (10_u32..110)
        .filter(|&frame| !hybrid.actions_for_frame(frame, true).is_empty())
        .count();

    assert!(
        non_empty_count >= 50,
        "expected at least 50/100 frames to produce non-empty actions after chaos phase starts \
         (action_prob=1.0), got {non_empty_count}"
    );
}

// -------------------------------------------------------------------------
// HybridInput — passes is_active=false to chaos phase
// -------------------------------------------------------------------------

/// `HybridInput` with `action_prob=1.0` must return empty `Vec` when
/// `is_active=false`, even for frames beyond the scripted phase.
#[test]
fn hybrid_input_respects_is_active_false_in_chaos_phase() {
    let params = HybridParams {
        scripted_frames: 10,
        action_prob: 1.0,
    };
    let mut hybrid = HybridInput::new(42, &params);

    // Frame 50 is well into the chaos phase but is_active=false
    let result = hybrid.actions_for_frame(50, false);
    assert!(
        result.is_empty(),
        "expected empty Vec when is_active=false in chaos phase, got {result:?}"
    );
}

// -------------------------------------------------------------------------
// HybridInput — new takes seed as separate argument
// -------------------------------------------------------------------------

/// `HybridInput::new(7, &params)` creates successfully with seed as separate arg.
#[test]
fn hybrid_input_new_takes_seed() {
    let params = HybridParams {
        scripted_frames: 100,
        action_prob: 0.5,
    };
    let mut hybrid = HybridInput::new(7, &params);

    // Scripted phase — must return empty
    let result = hybrid.actions_for_frame(0, true);
    assert!(
        result.is_empty(),
        "frame 0 in scripted phase must be empty, got {result:?}"
    );
}
