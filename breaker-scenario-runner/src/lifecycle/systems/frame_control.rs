//! Frame counting, frame limit checking, exit/restart, and game-state mapping.

use bevy::prelude::*;
use breaker::state::{
    run::node::messages::SpawnNodeComplete,
    types::{GameState, RunState},
};

use super::types::ScenarioConfig;
use crate::{
    invariants::{ScenarioFrame, ScenarioStats},
    types::ForcedGameState,
};

/// Increments [`ScenarioFrame`] by 1 each fixed-update tick.
///
/// Also updates [`ScenarioStats::max_frame`] when the stats resource is present.
pub fn tick_scenario_frame(
    mut frame: ResMut<ScenarioFrame>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    frame.0 += 1;
    if let Some(ref mut s) = stats {
        s.max_frame = frame.0;
    }
}

/// Sends [`AppExit::Success`] when [`ScenarioFrame`] reaches `max_frames`.
pub fn check_frame_limit(
    frame: Res<ScenarioFrame>,
    config: Res<ScenarioConfig>,
    mut exits: MessageWriter<AppExit>,
) {
    if frame.0 >= config.definition.max_frames {
        exits.write(AppExit::Success);
    }
}

/// Sends [`AppExit::Success`] when the run ends naturally.
///
/// Runs every frame while in `RunEnd` (not as a one-shot `OnEnter`) so that
/// the Winit event loop reliably sees the exit message on macOS.
pub fn exit_on_run_end(mut exits: MessageWriter<AppExit>) {
    exits.write(AppExit::Success);
}

/// Redirects `RunEnd` back to `Menu` (which `bypass_menu_to_playing`
/// sends to `Run`). Used when `allow_early_end` is false so the
/// scenario runs for the full `max_frames` frame budget.
///
/// Navigates through the teardown chain: `RunState::Teardown` triggers
/// routing back to `GameState::Menu`, which then fires
/// `bypass_menu_to_playing` on `OnEnter(MenuState::Main)`.
pub fn restart_run_on_end(
    mut next_run_phase: ResMut<NextState<RunState>>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    if let Some(ref mut stats) = stats {
        stats.entered_playing = false;
    }
    next_run_phase.set(RunState::Teardown);
}

/// Maps a [`ForcedGameState`] to the game crate's [`GameState`].
///
/// Used by [`super::debug_setup::apply_debug_setup`] to translate the RON-serializable enum
/// into the Bevy state enum. Since the game now uses a hierarchical state
/// machine, many old variants map to `GameState::Run` or `GameState::Menu`.
#[must_use]
pub const fn map_forced_game_state(forced: ForcedGameState) -> GameState {
    match forced {
        ForcedGameState::Loading => GameState::Loading,
        ForcedGameState::MainMenu | ForcedGameState::MetaProgression => GameState::Menu,
        ForcedGameState::RunSetup
        | ForcedGameState::Playing
        | ForcedGameState::TransitionOut
        | ForcedGameState::ChipSelect
        | ForcedGameState::TransitionIn
        | ForcedGameState::RunEnd => GameState::Run,
    }
}

/// Sets [`ScenarioStats::entered_playing`] to `true` when [`SpawnNodeComplete`]
/// fires, indicating all game entities are spawned and ready.
///
/// Frame counting and invariant checking are gated on `entered_playing`, so
/// no scenario frames advance until the node is fully loaded and spawned.
pub fn mark_entered_playing_on_spawn_complete(
    mut spawn_reader: MessageReader<SpawnNodeComplete>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    let spawned = spawn_reader.read().next().is_some();
    if spawned && let Some(ref mut s) = stats {
        s.entered_playing = true;
    }
}

/// Run condition: returns `true` when [`ScenarioStats::entered_playing`] is `true`.
///
/// Used as a `run_if` guard to prevent frame counting and frame-limit
/// checking from running before the game has entered `Playing`.
#[must_use]
pub fn entered_playing(stats: Option<Res<ScenarioStats>>) -> bool {
    stats.is_some_and(|s| s.entered_playing)
}
