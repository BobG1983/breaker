//! Input domain resources — action types and key bindings.

use bevy::prelude::*;
use brickbreaker_derive::GameConfig;
use serde::Deserialize;

/// Tracks timestamps for double-tap dash detection.
#[derive(Resource, Debug)]
pub struct DoubleTapState {
    /// Wall-clock time of the last left-direction press.
    pub last_left_tap: f64,
    /// Wall-clock time of the last right-direction press.
    pub last_right_tap: f64,
}

impl Default for DoubleTapState {
    fn default() -> Self {
        Self {
            last_left_tap: f64::NEG_INFINITY,
            last_right_tap: f64::NEG_INFINITY,
        }
    }
}

/// All player actions the game understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// Menu confirm selection.
    MenuConfirm,
}

/// Active actions for the current frame.
///
/// Populated each frame by `read_input_actions`. Consumed by gameplay systems.
#[derive(Resource, Debug, Default)]
pub struct InputActions(pub Vec<GameAction>);

impl InputActions {
    /// Returns `true` if the given action is active this frame.
    #[must_use]
    pub fn active(&self, action: GameAction) -> bool {
        self.0.contains(&action)
    }
}

/// Input defaults loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug, GameConfig)]
#[game_config(name = "InputConfig")]
pub struct InputDefaults {
    /// Keys that move the breaker left.
    pub move_left: Vec<KeyCode>,
    /// Keys that move the breaker right.
    pub move_right: Vec<KeyCode>,
    /// Keys that activate bump / launch bolt.
    pub bump: Vec<KeyCode>,
    /// Keys that navigate up in menus.
    pub menu_up: Vec<KeyCode>,
    /// Keys that navigate down in menus.
    pub menu_down: Vec<KeyCode>,
    /// Keys that confirm the current menu selection.
    pub menu_confirm: Vec<KeyCode>,
    /// Time window for double-tap dash detection (seconds).
    pub double_tap_window: f32,
}

impl Default for InputDefaults {
    fn default() -> Self {
        Self {
            move_left: vec![KeyCode::ArrowLeft, KeyCode::KeyA],
            move_right: vec![KeyCode::ArrowRight, KeyCode::KeyD],
            bump: vec![KeyCode::ArrowUp, KeyCode::KeyW],
            menu_up: vec![KeyCode::ArrowUp, KeyCode::KeyW],
            menu_down: vec![KeyCode::ArrowDown, KeyCode::KeyS],
            menu_confirm: vec![KeyCode::Enter, KeyCode::Space],
            double_tap_window: 0.25,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_actions_default_is_empty() {
        let actions = InputActions::default();
        assert!(actions.0.is_empty());
    }

    #[test]
    fn active_returns_true_for_present_action() {
        let actions = InputActions(vec![GameAction::Bump]);
        assert!(actions.active(GameAction::Bump));
    }

    #[test]
    fn active_returns_false_for_absent_action() {
        let actions = InputActions(vec![GameAction::MoveLeft]);
        assert!(!actions.active(GameAction::Bump));
    }

    #[test]
    fn ron_parses() {
        let ron_str = include_str!("../../assets/config/defaults.input.ron");
        let result: InputDefaults = ron::de::from_str(ron_str).expect("input RON should parse");
        assert!(!result.move_left.is_empty());
        assert!(result.double_tap_window > 0.0);
    }

    #[test]
    fn default_config_has_bindings() {
        let config = InputConfig::default();
        assert!(!config.move_left.is_empty());
        assert!(!config.move_right.is_empty());
        assert!(!config.bump.is_empty());
        assert!(!config.menu_up.is_empty());
        assert!(!config.menu_down.is_empty());
        assert!(!config.menu_confirm.is_empty());
        assert!(config.double_tap_window > 0.0);
    }
}
