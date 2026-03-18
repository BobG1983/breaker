//! Input strategies — [`ChaosDriver`], [`ScriptedInput`], [`HybridInput`], and [`InputDriver`].
//!
//! All strategies are pure Rust — no Bevy types. They produce `Vec<GameAction>`
//! for a given frame number. The Bevy integration (injecting into `InputActions`)
//! lives in the lifecycle module.

use rand::{Rng, SeedableRng, prelude::IndexedRandom, rngs::SmallRng};

use crate::types::{
    ChaosParams, GameAction, HybridParams, InputStrategy, ScriptedFrame, ScriptedParams,
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
    /// Create a new [`ChaosDriver`] from the given [`ChaosParams`].
    #[must_use]
    pub fn new(params: &ChaosParams) -> Self {
        Self {
            rng: SmallRng::seed_from_u64(params.seed),
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
    /// Creates a new [`HybridInput`] from the given [`HybridParams`].
    #[must_use]
    pub fn new(params: &HybridParams) -> Self {
        Self {
            scripted_frames: params.scripted_frames,
            chaos: ChaosDriver::new(&ChaosParams {
                seed: params.seed,
                action_prob: params.action_prob,
            }),
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
}

impl InputDriver {
    /// Constructs an [`InputDriver`] from any [`InputStrategy`] variant.
    #[must_use]
    pub fn from_strategy(strategy: &InputStrategy) -> Self {
        match strategy {
            InputStrategy::Chaos(params) => Self::Chaos(ChaosDriver::new(params)),
            InputStrategy::Scripted(params) => Self::Scripted(ScriptedInput::new(params)),
            InputStrategy::Hybrid(params) => Self::Hybrid(HybridInput::new(params)),
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
        }
    }
}

#[cfg(test)]
mod tests {
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
}
