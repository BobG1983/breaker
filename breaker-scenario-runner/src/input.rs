//! Input strategies — `ChaosMonkey` and `ScriptedInput`.
//!
//! Both strategies are pure Rust — no Bevy types. They produce `Vec<GameAction>`
//! for a given frame number. The Bevy integration (injecting into `InputActions`)
//! lives in the lifecycle module.

use rand::{Rng, SeedableRng, prelude::IndexedRandom, rngs::SmallRng};

use crate::types::{ChaosParams, GameAction, ScriptedFrame, ScriptedParams};

/// Gameplay-only actions that `ChaosMonkey` may inject.
///
/// Kept separate so menu actions are never produced by automated chaos input.
const GAMEPLAY_ACTIONS: &[GameAction] = &[
    GameAction::MoveLeft,
    GameAction::MoveRight,
    GameAction::Bump,
    GameAction::DashLeft,
    GameAction::DashRight,
];

/// Randomised input strategy driven by a seeded RNG.
///
/// On each frame there is an `action_prob` chance of injecting a single
/// randomly-chosen gameplay action. The sequence is fully deterministic given
/// the same `seed`.
pub struct ChaosMonkey {
    rng: SmallRng,
    action_prob: f32,
}

impl ChaosMonkey {
    /// Create a new [`ChaosMonkey`] from the given [`ChaosParams`].
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChaosParams, ScriptedFrame, ScriptedParams};

    // -------------------------------------------------------------------------
    // ChaosMonkey — statistical probability
    // -------------------------------------------------------------------------

    /// `ChaosMonkey` with `action_prob` 0.3 fires on roughly 30% of frames.
    ///
    /// With seed 0 and 1000 frames, the count of non-empty frames must fall
    /// between 230 and 370 (30% ± 10%).
    #[test]
    fn chaos_monkey_fires_at_correct_statistical_probability() {
        let params = ChaosParams {
            seed: 0,
            action_prob: 0.3,
        };
        let mut monkey = ChaosMonkey::new(&params);

        let fired_count = (0_u32..1000)
            .filter(|&frame| !monkey.actions_for_frame(frame, true).is_empty())
            .count();

        assert!(
            (230..=370).contains(&fired_count),
            "expected 230–370 non-empty frames (30% ± 10%), got {fired_count}"
        );
    }

    // -------------------------------------------------------------------------
    // ChaosMonkey — reproducibility
    // -------------------------------------------------------------------------

    /// Two `ChaosMonkey` instances seeded identically produce identical output sequences.
    #[test]
    fn chaos_monkey_produces_reproducible_output_for_same_seed() {
        let params = ChaosParams {
            seed: 42,
            action_prob: 0.5,
        };

        let mut monkey_a = ChaosMonkey::new(&params);
        let mut monkey_b = ChaosMonkey::new(&params);

        let frame0_a = monkey_a.actions_for_frame(0, true);
        let frame1_a = monkey_a.actions_for_frame(1, true);
        let frame0_b = monkey_b.actions_for_frame(0, true);
        let frame1_b = monkey_b.actions_for_frame(1, true);

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
    // ChaosMonkey — inactive returns empty
    // -------------------------------------------------------------------------

    /// `ChaosMonkey` with `action_prob` 1.0 (always fires when active) returns
    /// empty `Vec` when `is_active` is false.
    #[test]
    fn chaos_monkey_does_not_fire_when_inactive() {
        let params = ChaosParams {
            seed: 0,
            action_prob: 1.0,
        };
        let mut monkey = ChaosMonkey::new(&params);

        let result = monkey.actions_for_frame(0, false);

        assert!(
            result.is_empty(),
            "expected empty Vec when is_active=false, got {result:?}"
        );
    }

    // -------------------------------------------------------------------------
    // ChaosMonkey — only gameplay actions
    // -------------------------------------------------------------------------

    /// `ChaosMonkey` only produces gameplay actions, never menu actions.
    ///
    /// Over 200 frames with `action_prob` 1.0, every returned action must be one
    /// of `[MoveLeft, MoveRight, Bump, DashLeft, DashRight]`.
    #[test]
    fn chaos_monkey_only_picks_gameplay_actions() {
        let params = ChaosParams {
            seed: 1,
            action_prob: 1.0,
        };
        let mut monkey = ChaosMonkey::new(&params);

        let menu_actions = [
            GameAction::MenuUp,
            GameAction::MenuDown,
            GameAction::MenuLeft,
            GameAction::MenuRight,
            GameAction::MenuConfirm,
        ];

        for frame in 0_u32..200 {
            let actions = monkey.actions_for_frame(frame, true);
            for action in &actions {
                assert!(
                    !menu_actions.contains(action),
                    "frame {frame}: got menu action {action:?} — ChaosMonkey must only produce gameplay actions"
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
}
