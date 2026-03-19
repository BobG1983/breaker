use super::*;
use crate::types::{ChaosParams, HybridParams, ScriptedFrame, ScriptedParams};

// -------------------------------------------------------------------------
// ChaosDriver — statistical probability
// -------------------------------------------------------------------------

/// `ChaosDriver` with `action_prob` 0.3 fires on roughly 30% of frames.
///
/// With seed 0 and 1000 frames, the count of non-empty frames must fall
/// between 230 and 370 (30% ± 10%).
#[test]
fn chaos_driver_fires_at_correct_statistical_probability() {
    let params = ChaosParams {
        seed: 0,
        action_prob: 0.3,
    };
    let mut driver = ChaosDriver::new(&params);

    let fired_count = (0_u32..1000)
        .filter(|&frame| !driver.actions_for_frame(frame, true).is_empty())
        .count();

    assert!(
        (230..=370).contains(&fired_count),
        "expected 230–370 non-empty frames (30% ± 10%), got {fired_count}"
    );
}

// -------------------------------------------------------------------------
// ChaosDriver — reproducibility
// -------------------------------------------------------------------------

/// Two `ChaosDriver` instances seeded identically produce identical output sequences.
#[test]
fn chaos_driver_produces_reproducible_output_for_same_seed() {
    let params = ChaosParams {
        seed: 42,
        action_prob: 0.5,
    };

    let mut driver_a = ChaosDriver::new(&params);
    let mut driver_b = ChaosDriver::new(&params);

    let frame0_a = driver_a.actions_for_frame(0, true);
    let frame1_a = driver_a.actions_for_frame(1, true);
    let frame0_b = driver_b.actions_for_frame(0, true);
    let frame1_b = driver_b.actions_for_frame(1, true);

    assert_eq!(
        frame0_a, frame0_b,
        "frame 0 output must match for identical seeds"
    );
    assert_eq!(
        frame1_a, frame1_b,
        "frame 1 output must match for identical seeds"
    );
}

// -------------------------------------------------------------------------
// ChaosDriver — inactive returns empty
// -------------------------------------------------------------------------

/// `ChaosDriver` with `action_prob` 1.0 (always fires when active) returns
/// empty `Vec` when `is_active` is false.
#[test]
fn chaos_driver_does_not_fire_when_inactive() {
    let params = ChaosParams {
        seed: 0,
        action_prob: 1.0,
    };
    let mut driver = ChaosDriver::new(&params);

    let result = driver.actions_for_frame(0, false);

    assert!(
        result.is_empty(),
        "expected empty Vec when is_active=false, got {result:?}"
    );
}

// -------------------------------------------------------------------------
// ChaosDriver — only gameplay actions
// -------------------------------------------------------------------------

/// `ChaosDriver` only produces gameplay actions, never menu actions.
///
/// Over 200 frames with `action_prob` 1.0, every returned action must be one
/// of `[MoveLeft, MoveRight, Bump, DashLeft, DashRight]`.
#[test]
fn chaos_driver_only_picks_gameplay_actions() {
    let params = ChaosParams {
        seed: 1,
        action_prob: 1.0,
    };
    let mut driver = ChaosDriver::new(&params);

    let menu_actions = [
        GameAction::MenuUp,
        GameAction::MenuDown,
        GameAction::MenuLeft,
        GameAction::MenuRight,
        GameAction::MenuConfirm,
    ];

    for frame in 0_u32..200 {
        let actions = driver.actions_for_frame(frame, true);
        for action in &actions {
            assert!(
                !menu_actions.contains(action),
                "frame {frame}: got menu action {action:?} — ChaosDriver must only produce gameplay actions"
            );
            assert!(
                GAMEPLAY_ACTIONS.contains(action),
                "frame {frame}: got unexpected action {action:?}"
            );
        }
    }
}

// -------------------------------------------------------------------------
// ScriptedInput — fires at exact frame numbers
// -------------------------------------------------------------------------

/// `ScriptedInput` fires the configured actions only on the matching frames.
///
/// Frame 5 returns `[MoveLeft]`; frame 10 returns `[Bump, MoveRight]`;
/// all other frames 0..15 return empty `Vec`.
#[test]
fn scripted_input_fires_at_exact_frame_numbers() {
    let params = ScriptedParams {
        actions: vec![
            ScriptedFrame {
                frame: 5,
                actions: vec![GameAction::MoveLeft],
            },
            ScriptedFrame {
                frame: 10,
                actions: vec![GameAction::Bump, GameAction::MoveRight],
            },
        ],
    };
    let scripted = ScriptedInput::new(&params);

    assert_eq!(
        scripted.actions_for_frame(5),
        vec![GameAction::MoveLeft],
        "frame 5 must return [MoveLeft]"
    );
    assert_eq!(
        scripted.actions_for_frame(10),
        vec![GameAction::Bump, GameAction::MoveRight],
        "frame 10 must return [Bump, MoveRight]"
    );

    // All frames except 5 and 10 must return empty.
    for frame in 0_u32..15 {
        if frame == 5 || frame == 10 {
            continue;
        }
        let result = scripted.actions_for_frame(frame);
        assert!(
            result.is_empty(),
            "frame {frame} should return empty Vec, got {result:?}"
        );
    }
}

// -------------------------------------------------------------------------
// ScriptedInput — empty entries returns nothing
// -------------------------------------------------------------------------

/// `ScriptedInput` with no entries always returns an empty `Vec`.
#[test]
fn scripted_input_with_empty_entries_returns_nothing() {
    let params = ScriptedParams { actions: vec![] };
    let scripted = ScriptedInput::new(&params);

    for frame in [0_u32, 1, 100, u32::MAX / 2] {
        let result = scripted.actions_for_frame(frame);
        assert!(
            result.is_empty(),
            "frame {frame}: expected empty Vec from empty ScriptedInput, got {result:?}"
        );
    }
}

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
        seed: 42,
        action_prob: 1.0,
    };
    let mut hybrid = HybridInput::new(&params);

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
        seed: 42,
        action_prob: 1.0,
    };
    let mut hybrid = HybridInput::new(&params);

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
        seed: 42,
        action_prob: 1.0,
    };
    let mut hybrid = HybridInput::new(&params);

    // Frame 50 is well into the chaos phase but is_active=false
    let result = hybrid.actions_for_frame(50, false);
    assert!(
        result.is_empty(),
        "expected empty Vec when is_active=false in chaos phase, got {result:?}"
    );
}

// -------------------------------------------------------------------------
// InputDriver — constructs from Chaos strategy
// -------------------------------------------------------------------------

/// `InputDriver::from_strategy` with `InputStrategy::Chaos` must not panic
/// and must produce a callable driver.
///
/// `actions_for_frame(0, true)` on the resulting driver must return a `Vec`
/// (empty or not — exact content depends on the RNG seed).
#[test]
fn input_driver_from_chaos_strategy_constructs_without_panic() {
    let strategy = InputStrategy::Chaos(ChaosParams {
        seed: 42,
        action_prob: 0.5,
    });
    let mut driver = InputDriver::from_strategy(&strategy);

    // Must not panic — return type is Vec<GameAction>
    drop(driver.actions_for_frame(0, true));
}

// -------------------------------------------------------------------------
// InputDriver — constructs from Scripted strategy
// -------------------------------------------------------------------------

/// `InputDriver::from_strategy` with `InputStrategy::Scripted` must inject
/// exactly the configured actions at the specified frame and nothing elsewhere.
///
/// Frame 5 must return `[Bump]`; frame 0 must return `[]`.
#[test]
fn input_driver_from_scripted_strategy_fires_at_configured_frame() {
    let strategy = InputStrategy::Scripted(ScriptedParams {
        actions: vec![ScriptedFrame {
            frame: 5,
            actions: vec![GameAction::Bump],
        }],
    });
    let mut driver = InputDriver::from_strategy(&strategy);

    let result_frame_5 = driver.actions_for_frame(5, true);
    assert_eq!(
        result_frame_5,
        vec![GameAction::Bump],
        "expected [Bump] at frame 5 from Scripted driver"
    );

    let result_frame_0 = driver.actions_for_frame(0, true);
    assert!(
        result_frame_0.is_empty(),
        "expected empty Vec at frame 0 from Scripted driver, got {result_frame_0:?}"
    );
}

// -------------------------------------------------------------------------
// InputDriver — constructs from Hybrid strategy
// -------------------------------------------------------------------------

/// `InputDriver::from_strategy` with `InputStrategy::Hybrid` must:
/// - return `[]` at frame 5 (inside scripted phase of 10 frames)
/// - return non-empty at frame 50 when `action_prob=1.0` (chaos phase)
#[test]
fn input_driver_from_hybrid_strategy_respects_phase_boundary() {
    let strategy = InputStrategy::Hybrid(HybridParams {
        scripted_frames: 10,
        seed: 42,
        action_prob: 1.0,
    });
    let mut driver = InputDriver::from_strategy(&strategy);

    // Scripted phase — frame 5 must be empty
    let result_scripted = driver.actions_for_frame(5, true);
    assert!(
        result_scripted.is_empty(),
        "expected empty Vec at frame 5 (scripted phase), got {result_scripted:?}"
    );

    // Chaos phase — frame 50 with action_prob=1.0 must produce actions
    let result_chaos = driver.actions_for_frame(50, true);
    assert!(
        !result_chaos.is_empty(),
        "expected non-empty Vec at frame 50 (chaos phase, action_prob=1.0), got {result_chaos:?}"
    );
}
