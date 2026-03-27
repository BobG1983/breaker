//! Input strategies — [`ChaosDriver`], [`ScriptedInput`], [`HybridInput`], and [`InputDriver`].
//!
//! All strategies are pure Rust — no Bevy types. They produce `Vec<GameAction>`
//! for a given frame number. The Bevy integration (injecting into `InputActions`)
//! lives in the lifecycle module.

#[cfg(test)]
mod tests;

use rand::{Rng, SeedableRng, prelude::IndexedRandom, rngs::SmallRng};

use crate::types::{
    BumpMode, ChaosParams, GameAction, HybridParams, InputStrategy, ScriptedFrame, ScriptedParams,
};

/// Gameplay-only actions that [`ChaosDriver`] may inject.
///
/// Kept separate so menu actions are never produced by automated chaos input.
const GAMEPLAY_ACTIONS: &[GameAction] = &[
    GameAction::MoveLeft,
    GameAction::MoveRight,
    GameAction::Bump,
    GameAction::DashLeft,
    GameAction::DashRight,
    GameAction::TogglePause,
];

/// Randomised input strategy driven by a seeded RNG.
///
/// On each frame there is an `action_prob` chance of injecting a single
/// randomly-chosen gameplay action. The sequence is fully deterministic given
/// the same `seed`.
pub struct ChaosDriver {
    rng: SmallRng,
    action_prob: f32,
}

impl ChaosDriver {
    /// Create a new [`ChaosDriver`] from the given seed and [`ChaosParams`].
    #[must_use]
    pub fn new(seed: u64, params: &ChaosParams) -> Self {
        Self {
            rng: SmallRng::seed_from_u64(seed),
            action_prob: params.action_prob,
        }
    }

    /// Returns actions for this frame.
    ///
    /// Returns an empty `Vec` when `is_active` is `false` or when the RNG roll
    /// does not reach `action_prob`. Otherwise returns a `Vec` containing one
    /// randomly-chosen gameplay action.
    ///
    /// The `_frame` parameter is reserved for future rate-limiting use.
    pub fn actions_for_frame(&mut self, _frame: u32, is_active: bool) -> Vec<GameAction> {
        if !is_active {
            return vec![];
        }
        let roll: f32 = self.rng.random();
        if roll >= self.action_prob {
            return vec![];
        }
        // GAMEPLAY_ACTIONS is non-empty; choose() only returns None on empty slices
        GAMEPLAY_ACTIONS
            .choose(&mut self.rng)
            .copied()
            .map(|action| vec![action])
            .unwrap_or_default()
    }
}

/// Deterministic scripted input strategy.
///
/// Fires fixed action sequences at specific frame numbers. All other frames
/// return an empty `Vec`.
pub struct ScriptedInput {
    actions: Vec<ScriptedFrame>,
}

impl ScriptedInput {
    /// Create a new [`ScriptedInput`] from the given [`ScriptedParams`].
    #[must_use]
    pub fn new(params: &ScriptedParams) -> Self {
        Self {
            actions: params.actions.clone(),
        }
    }

    /// Returns the actions scheduled for `frame`, or an empty `Vec` if none.
    #[must_use]
    pub fn actions_for_frame(&self, frame: u32) -> Vec<GameAction> {
        self.actions
            .iter()
            .find(|entry| entry.frame == frame)
            .map(|entry| entry.actions.clone())
            .unwrap_or_default()
    }
}

/// Hybrid input strategy: scripted for the first `scripted_frames` frames, then chaos.
///
/// Frames `0..scripted_frames` always return an empty `Vec` (no scripted entries are
/// stored — the scripted phase acts as a silent warmup). Frame `scripted_frames` is
/// the first frame in the chaos phase; all subsequent frames also delegate to the
/// inner [`ChaosDriver`].
pub struct HybridInput {
    /// Number of frames in the scripted (silent) phase.
    pub scripted_frames: u32,
    /// Inner chaos driver used after the scripted phase.
    pub chaos: ChaosDriver,
}

impl HybridInput {
    /// Creates a new [`HybridInput`] from the given seed and [`HybridParams`].
    #[must_use]
    pub fn new(seed: u64, params: &HybridParams) -> Self {
        Self {
            scripted_frames: params.scripted_frames,
            chaos: ChaosDriver::new(
                seed,
                &ChaosParams {
                    action_prob: params.action_prob,
                },
            ),
        }
    }

    /// Returns actions for `frame`.
    ///
    /// Returns empty `Vec` while `frame < scripted_frames`.
    /// Delegates to [`ChaosDriver`] when `frame >= scripted_frames` and `is_active` is `true`.
    #[must_use]
    pub fn actions_for_frame(&mut self, frame: u32, is_active: bool) -> Vec<GameAction> {
        if frame < self.scripted_frames {
            return vec![];
        }
        self.chaos.actions_for_frame(frame, is_active)
    }
}

/// Perfect-timing input strategy driven by a [`BumpMode`].
///
/// Observes the bolt's proximity to the breaker each frame and times bump
/// actions according to the configured mode.
pub struct PerfectDriver {
    /// Which bump timing mode to use.
    pub bump_mode: BumpMode,
    /// Seeded RNG for the `Random` mode.
    pub rng: SmallRng,
}

impl PerfectDriver {
    /// Create a new [`PerfectDriver`] with the given seed and [`BumpMode`].
    #[must_use]
    pub fn new(seed: u64, mode: BumpMode) -> Self {
        Self {
            bump_mode: mode,
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    /// Returns actions for this frame.
    ///
    /// The `_frame` parameter is reserved for future use.
    /// The `_is_active` parameter indicates whether gameplay input is active.
    pub const fn actions_for_frame(&mut self, _frame: u32, _is_active: bool) -> Vec<GameAction> {
        vec![]
    }
}

/// Unified input driver that wraps any [`InputStrategy`] variant.
///
/// Created via [`InputDriver::from_strategy`] and queried per-frame via
/// [`InputDriver::actions_for_frame`].
pub enum InputDriver {
    /// Randomised chaos input.
    Chaos(ChaosDriver),
    /// Fully scripted deterministic input.
    Scripted(ScriptedInput),
    /// Scripted warmup then chaos.
    Hybrid(HybridInput),
    /// Perfect-timing input.
    Perfect(PerfectDriver),
}

impl InputDriver {
    /// Constructs an [`InputDriver`] from any [`InputStrategy`] variant and a seed.
    #[must_use]
    pub fn from_strategy(strategy: &InputStrategy, seed: u64) -> Self {
        match strategy {
            InputStrategy::Chaos(params) => Self::Chaos(ChaosDriver::new(seed, params)),
            InputStrategy::Scripted(params) => Self::Scripted(ScriptedInput::new(params)),
            InputStrategy::Hybrid(params) => Self::Hybrid(HybridInput::new(seed, params)),
            InputStrategy::Perfect(mode) => Self::Perfect(PerfectDriver::new(seed, *mode)),
        }
    }

    /// Returns actions for `frame`, delegating to the inner strategy.
    ///
    /// The `is_active` flag is passed through to strategies that support it.
    /// For [`InputDriver::Scripted`], `is_active` is ignored — scripted actions
    /// always fire at their configured frames.
    pub fn actions_for_frame(&mut self, frame: u32, is_active: bool) -> Vec<GameAction> {
        match self {
            Self::Chaos(driver) => driver.actions_for_frame(frame, is_active),
            Self::Scripted(scripted) => scripted.actions_for_frame(frame),
            Self::Hybrid(hybrid) => hybrid.actions_for_frame(frame, is_active),
            Self::Perfect(perfect) => perfect.actions_for_frame(frame, is_active),
        }
    }
}
