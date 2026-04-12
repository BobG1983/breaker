//! Input strategy types for scenario definitions.

use serde::Deserialize;

/// All gameplay actions that can be injected by an input strategy.
///
/// Mirrors `breaker::input::resources::GameAction` but lives here so it can
/// derive [`Deserialize`] independently (the game crate does not expose that
/// derive on the original type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum GameAction {
    /// Continuous horizontal movement left.
    MoveLeft,
    /// Continuous horizontal movement right.
    MoveRight,
    /// Bump activation (also launches serving bolt).
    Bump,
    /// Dash left (double-tap detected).
    DashLeft,
    /// Dash right (double-tap detected).
    DashRight,
    /// Menu navigate up.
    MenuUp,
    /// Menu navigate down.
    MenuDown,
    /// Menu navigate left (horizontal menus).
    MenuLeft,
    /// Menu navigate right (horizontal menus).
    MenuRight,
    /// Menu confirm selection.
    MenuConfirm,
    /// Toggle pause state.
    TogglePause,
}

/// Bump timing mode for the [`InputStrategy::Perfect`] variant.
///
/// Controls how the `PerfectDriver` times its bump actions relative to the
/// bolt's proximity to the breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum BumpMode {
    /// Always produces a perfectly-timed bump.
    AlwaysPerfect,
    /// Always produces an early bump.
    AlwaysEarly,
    /// Always produces a late bump.
    AlwaysLate,
    /// Always produces a whiff (miss).
    AlwaysWhiff,
    /// Never bumps at all.
    NeverBump,
    /// Randomly chooses a bump timing per frame.
    Random,
}

/// A single scripted frame entry -- a frame index and the actions to inject.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ScriptedFrame {
    /// The fixed-update frame on which these actions are injected.
    pub frame:   u32,
    /// Actions to inject on that frame.
    pub actions: Vec<GameAction>,
}

/// Parameters for the [`InputStrategy::Chaos`] variant.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ChaosParams {
    /// Probability (0.0--1.0) of injecting any action on a given frame.
    pub action_prob: f32,
}

/// Parameters for the [`InputStrategy::Scripted`] variant.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ScriptedParams {
    /// Ordered list of frame-action pairs.
    pub actions: Vec<ScriptedFrame>,
}

/// Parameters for the [`InputStrategy::Hybrid`] variant.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct HybridParams {
    /// Number of frames to play back scripted actions before switching to chaos.
    pub scripted_frames: u32,
    /// Probability (0.0--1.0) for the random phase.
    pub action_prob:     f32,
}

/// Input injection strategy for a scenario run.
///
/// RON newtype-variant syntax: `Chaos((action_prob: 0.3))`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum InputStrategy {
    /// Randomised action injection driven by an RNG seed and probability.
    Chaos(ChaosParams),
    /// Fully deterministic sequence of frame-action pairs.
    Scripted(ScriptedParams),
    /// Scripted actions up to `scripted_frames`, then random chaos afterwards.
    Hybrid(HybridParams),
    /// Perfect-timing input driven by a [`BumpMode`] strategy.
    Perfect(BumpMode),
}
