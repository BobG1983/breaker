use super::super::drivers::*;
use crate::types::{ChaosParams, GameAction};

// -------------------------------------------------------------------------
// ChaosDriver — statistical probability
// -------------------------------------------------------------------------

/// `ChaosDriver` with `action_prob` 0.3 fires on roughly 30% of frames.
///
/// With seed 0 and 1000 frames, the count of non-empty frames must fall
/// between 230 and 370 (30% +/- 10%).
#[test]
fn chaos_driver_fires_at_correct_statistical_probability() {
    let params = ChaosParams { action_prob: 0.3 };
    let mut driver = ChaosDriver::new(0, &params);

    let fired_count = (0_u32..1000)
        .filter(|&frame| !driver.actions_for_frame(frame, true).is_empty())
        .count();

    assert!(
        (230..=370).contains(&fired_count),
        "expected 230-370 non-empty frames (30% +/- 10%), got {fired_count}"
    );
}

// -------------------------------------------------------------------------
// ChaosDriver — reproducibility
// -------------------------------------------------------------------------

/// Two `ChaosDriver` instances seeded identically produce identical output sequences.
#[test]
fn chaos_driver_produces_reproducible_output_for_same_seed() {
    let params = ChaosParams { action_prob: 0.5 };

    let mut driver_a = ChaosDriver::new(42, &params);
    let mut driver_b = ChaosDriver::new(42, &params);

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
    let params = ChaosParams { action_prob: 1.0 };
    let mut driver = ChaosDriver::new(0, &params);

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
    let params = ChaosParams { action_prob: 1.0 };
    let mut driver = ChaosDriver::new(1, &params);

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
// ChaosDriver — new takes seed as separate argument
// -------------------------------------------------------------------------

/// `ChaosDriver::new(42, &ChaosParams { action_prob: 0.3 })` produces
/// deterministic output from frame 0.
#[test]
fn chaos_driver_new_takes_seed() {
    let params = ChaosParams { action_prob: 0.3 };
    let mut driver = ChaosDriver::new(42, &params);

    // Must produce at least one deterministic frame result without panic
    let result = driver.actions_for_frame(0, true);
    // Result is deterministic but we only assert it's a valid Vec
    assert!(
        result.len() <= 1,
        "ChaosDriver should produce at most one action per frame, got {result:?}"
    );
}

/// Two `ChaosDriver` instances with the same explicit seed produce identical output.
#[test]
fn chaos_driver_same_seed_same_output() {
    let params = ChaosParams { action_prob: 0.5 };

    let mut driver_a = ChaosDriver::new(42, &params);
    let mut driver_b = ChaosDriver::new(42, &params);

    for frame in 0_u32..50 {
        let a = driver_a.actions_for_frame(frame, true);
        let b = driver_b.actions_for_frame(frame, true);
        assert_eq!(
            a, b,
            "frame {frame}: same seed must produce identical output"
        );
    }
}
