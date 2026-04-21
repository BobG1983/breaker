//! Scenario input injection systems.

use bevy::prelude::*;
use breaker::input::resources::InputActions;

use super::types::{ChaosInputLog, ScenarioConfig, ScenarioInputDriver};
use crate::{
    invariants::{ScenarioFrame, ScenarioStats},
    types::{GameAction as ScenarioGameAction, InputStrategy, ScriptedFrame},
};

/// Reads [`ScenarioConfig`] and inserts a [`ScenarioInputDriver`] into the world.
///
/// Runs once at scenario startup.
pub fn init_scenario_input(config: Res<ScenarioConfig>, mut commands: Commands) {
    let seed = config.definition.seed.unwrap_or(0);
    let driver = crate::input::InputDriver::from_strategy(&config.definition.input, seed);
    commands.insert_resource(ScenarioInputDriver(driver));
}

/// Maps a scenario-crate [`ScenarioGameAction`] to the game-crate
/// [`breaker::input::resources::GameAction`].
#[must_use]
pub const fn map_action(action: ScenarioGameAction) -> breaker::input::resources::GameAction {
    match action {
        ScenarioGameAction::MoveLeft => breaker::input::resources::GameAction::MoveLeft,
        ScenarioGameAction::MoveRight => breaker::input::resources::GameAction::MoveRight,
        ScenarioGameAction::Bump => breaker::input::resources::GameAction::Bump,
        ScenarioGameAction::DashLeft => breaker::input::resources::GameAction::DashLeft,
        ScenarioGameAction::DashRight => breaker::input::resources::GameAction::DashRight,
        ScenarioGameAction::MenuUp => breaker::input::resources::GameAction::MenuUp,
        ScenarioGameAction::MenuDown => breaker::input::resources::GameAction::MenuDown,
        ScenarioGameAction::MenuLeft => breaker::input::resources::GameAction::MenuLeft,
        ScenarioGameAction::MenuRight => breaker::input::resources::GameAction::MenuRight,
        ScenarioGameAction::MenuConfirm => breaker::input::resources::GameAction::MenuConfirm,
        ScenarioGameAction::TogglePause => breaker::input::resources::GameAction::TogglePause,
    }
}

/// Injects scenario-controlled actions into [`InputActions`] each fixed-update tick.
///
/// Reads [`ScenarioInputDriver`], queries the current [`ScenarioFrame`], maps the
/// scenario-crate [`crate::types::GameAction`] values to the game crate's
/// [`breaker::input::resources::GameAction`], and writes to [`InputActions`].
///
/// Uses `Option<ResMut<ScenarioInputDriver>>` so it does not panic if the resource
/// has not yet been inserted.
///
/// When the active [`crate::types::InputStrategy`] is `Chaos` or `Hybrid` and the
/// driver emitted at least one action this frame, the actions are cloned into
/// [`ChaosInputLog`] so the runner can dump a replayable scripted scenario on
/// failure. Recording is a passive side effect — the driver is only invoked
/// once, preserving RNG determinism.
pub fn inject_scenario_input(
    mut driver: Option<ResMut<ScenarioInputDriver>>,
    frame: Res<ScenarioFrame>,
    mut actions: ResMut<InputActions>,
    mut stats: Option<ResMut<ScenarioStats>>,
    config: Option<Res<ScenarioConfig>>,
    chaos_log: Option<ResMut<ChaosInputLog>>,
) {
    let Some(ref mut driver) = driver else {
        return;
    };

    let scenario_actions = driver.0.actions_for_frame(frame.0, true);
    let action_count = u32::try_from(scenario_actions.len()).unwrap_or(u32::MAX);

    if !scenario_actions.is_empty()
        && let Some(cfg) = config.as_ref()
        && matches!(
            cfg.definition.input,
            InputStrategy::Chaos(_) | InputStrategy::Hybrid(_)
        )
        && let Some(mut log) = chaos_log
    {
        log.0.push(ScriptedFrame {
            frame:   frame.0,
            actions: scenario_actions.clone(),
        });
    }

    for action in scenario_actions {
        actions.0.push(map_action(action));
    }

    if let Some(ref mut s) = stats {
        s.actions_injected += action_count;
    }
}
