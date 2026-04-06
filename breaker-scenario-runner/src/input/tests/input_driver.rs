use crate::{
    input::drivers::*,
    types::{
        BumpMode, ChaosParams, GameAction, HybridParams, InputStrategy, ScriptedFrame,
        ScriptedParams,
    },
};

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
    let strategy = InputStrategy::Chaos(ChaosParams { action_prob: 0.5 });
    let mut driver = InputDriver::from_strategy(&strategy, 42);

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
    let mut driver = InputDriver::from_strategy(&strategy, 0);

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
        action_prob: 1.0,
    });
    let mut driver = InputDriver::from_strategy(&strategy, 42);

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

// -------------------------------------------------------------------------
// InputDriver — constructs from Perfect strategy
// -------------------------------------------------------------------------

/// `InputDriver::from_strategy(&Perfect(AlwaysPerfect), 99)` creates a Perfect variant.
#[test]
fn input_driver_from_perfect() {
    let strategy = InputStrategy::Perfect(BumpMode::AlwaysPerfect);
    let driver = InputDriver::from_strategy(&strategy, 99);

    assert!(
        matches!(driver, InputDriver::Perfect(_)),
        "expected InputDriver::Perfect variant"
    );
}

// -------------------------------------------------------------------------
// PerfectDriver — stub returns empty actions
// -------------------------------------------------------------------------

/// `PerfectDriver` stub returns empty `Vec` from `actions_for_frame`.
#[test]
fn perfect_driver_returns_empty_actions() {
    let mut driver = PerfectDriver::new(42, BumpMode::AlwaysPerfect);
    let result = driver.actions_for_frame(0, true);
    assert!(
        result.is_empty(),
        "PerfectDriver stub must return empty Vec, got {result:?}"
    );
}
